use core::cell::{Cell, RefCell};
use core::ffi::c_char;
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

use block2::RcBlock;
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, extern_class,
    extern_methods, extern_protocol, msg_send,
};
use objc2_foundation::{
    NSArray, NSError, NSItemProvider, NSObjectProtocol, NSProgress, NSString, NSTemporaryDirectory,
    NSURL,
};
use objc2_photos_ui::{
    PHPickerConfiguration, PHPickerConfigurationAssetRepresentationMode, PHPickerFilter,
    PHPickerResult,
};
use objc2_ui_kit::UIViewController;

use crate::copy::{CopyError, copy_provider_file};
use crate::session_policy::{PickerCompletion, SelectionDecision, selection_decision};
use crate::{IosPhotoPickerError, IosPhotoPickerEvent};

/// A main-thread-bound, one-image PhotosUI session.
pub struct IosPhotoPickerSession {
    picker: Retained<NativePHPickerViewController>,
    delegate: Retained<PhotoPickerDelegate>,
    _main_thread: MainThreadMarker,
    _not_send: PhantomData<*mut ()>,
}

impl IosPhotoPickerSession {
    /// Presents the system image picker and copies one provider file under `max_bytes`.
    ///
    /// `max_bytes` must be nonzero. The callback is `Send` because the provider may invoke its
    /// completion on a background queue. The selected file contains exactly the bytes returned
    /// by the provider's `public.image` file representation, which may be a compatible rather than
    /// original library representation. This method requests no Photos authorization and does not
    /// enumerate the library. It confirms only that UIKit presentation was requested; it does not
    /// prove that the picker appeared or a user selected an image.
    pub fn present<F>(
        main_thread: MainThread,
        presenter: &UIViewController,
        max_bytes: u64,
        callback: F,
    ) -> Result<Self, IosPhotoPickerError>
    where
        F: FnOnce(IosPhotoPickerEvent) + Send + 'static,
    {
        if max_bytes == 0 {
            return Err(IosPhotoPickerError::InvalidByteLimit);
        }
        let marker = main_thread.into_objc2();
        // SAFETY: These public PhotosUI constructors and setters are called on the UIKit main
        // thread. Selection is limited to one image and the current representation is preferred.
        let configuration = unsafe { PHPickerConfiguration::new() };
        unsafe {
            configuration.setSelectionLimit(1);
            configuration.setPreferredAssetRepresentationMode(
                PHPickerConfigurationAssetRepresentationMode::Current,
            );
        }
        // SAFETY: `imagesFilter` is a public PhotosUI class property available on iOS 14.
        let image_filter = unsafe { PHPickerFilter::imagesFilter() };
        // SAFETY: The configuration copies this retained public filter object.
        unsafe { configuration.setFilter(Some(&image_filter)) };

        // SAFETY: This local Objective-C binding mirrors the public iOS initializer declared in
        // the Xcode PhotosUI SDK header and retains the created controller.
        let picker = unsafe {
            NativePHPickerViewController::initWithConfiguration(
                NativePHPickerViewController::alloc(marker),
                &configuration,
            )
        };
        let completion = PickerCompletion::new(callback);
        let delegate = PhotoPickerDelegate::new(marker, completion, max_bytes);
        // SAFETY: PhotosUI stores a weak delegate; `Self` retains the delegate for the session.
        unsafe { picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };
        presenter.presentViewController_animated_completion(&picker, true, None);
        Ok(Self {
            picker,
            delegate,
            _main_thread: marker,
            _not_send: PhantomData,
        })
    }

    /// Cancels a pending selection or provider transfer and settles the callback once as cancelled.
    pub fn cancel(&mut self) {
        self.delegate.cancel();
        self.delegate.dismiss_if_presented(&self.picker);
    }
}

impl Drop for IosPhotoPickerSession {
    fn drop(&mut self) {
        self.delegate.detach();
        self.delegate.dismiss_if_presented(&self.picker);
    }
}

struct PhotoPickerDelegateIvars {
    completion: PickerCompletion,
    progress: RefCell<Option<Retained<NSProgress>>>,
    is_presented: Cell<bool>,
    max_bytes: u64,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - Ivars are used only on the main thread except for the separately synchronized completion.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosPhotoPickerDelegate"]
    #[ivars = PhotoPickerDelegateIvars]
    struct PhotoPickerDelegate;

    // SAFETY: NSObjectProtocol adds no required selectors.
    unsafe impl NSObjectProtocol for PhotoPickerDelegate {}

    // SAFETY: PhotosUI invokes this optional delegate on the main thread. The completion state is
    // mutex-protected for provider callbacks, which may execute on a background queue.
    unsafe impl NativePHPickerViewControllerDelegate for PhotoPickerDelegate {
        #[unsafe(method(picker:didFinishPicking:))]
        unsafe fn picker_did_finish_picking(
            &self,
            picker: &NativePHPickerViewController,
            results: &NSArray<PHPickerResult>,
        ) {
            let _keep_alive = self.retain();
            let result = catch_unwind(AssertUnwindSafe(|| self.handle_results(picker, results)));
            if result.is_err() {
                // SAFETY: We are still on the UIKit main thread and use public controller methods.
                self.dismiss(picker);
                self.ivars().completion.finish(IosPhotoPickerEvent::Failed(
                    IosPhotoPickerError::CopyFailed {
                        message: String::from("Rust panic while handling PhotosUI selection"),
                    },
                ));
            }
        }
    }
);

impl PhotoPickerDelegate {
    fn new(
        marker: MainThreadMarker,
        completion: PickerCompletion,
        max_bytes: u64,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(PhotoPickerDelegateIvars {
            completion,
            progress: RefCell::new(None),
            is_presented: Cell::new(true),
            max_bytes,
        });
        // SAFETY: NSObject is the direct superclass and all ivars are initialized before init.
        unsafe { msg_send![super(this), init] }
    }

    fn handle_results(
        &self,
        picker: &NativePHPickerViewController,
        results: &NSArray<PHPickerResult>,
    ) {
        // SAFETY: Delegate callbacks are main-thread-only; the result array is borrowed for this call.
        let count = results.count();
        match selection_decision(u64::try_from(count).unwrap_or(u64::MAX)) {
            SelectionDecision::Cancelled => {
                self.dismiss(picker);
                self.ivars()
                    .completion
                    .finish(IosPhotoPickerEvent::Cancelled);
                return;
            }
            SelectionDecision::UnexpectedCount(actual) => {
                self.dismiss(picker);
                self.ivars().completion.finish(IosPhotoPickerEvent::Failed(
                    IosPhotoPickerError::UnexpectedSelectionCount { actual },
                ));
                return;
            }
            SelectionDecision::Selected => {}
        }
        if !self.ivars().completion.begin_loading() {
            self.dismiss(picker);
            return;
        }
        // SAFETY: The configured selection limit is one and `count` was checked above.
        let result = results.objectAtIndex(0);
        // SAFETY: PHPickerResult.itemProvider is a public retained property.
        let provider = unsafe { result.itemProvider() };
        let type_identifier = NSString::from_str("public.image");
        if !provider.hasItemConformingToTypeIdentifier(&type_identifier) {
            self.dismiss(picker);
            self.ivars().completion.finish(IosPhotoPickerEvent::Failed(
                IosPhotoPickerError::ImageRepresentationUnavailable,
            ));
            return;
        }

        self.dismiss(picker);
        self.load_image(provider, type_identifier);
    }

    fn dismiss(&self, picker: &NativePHPickerViewController) {
        if self.ivars().is_presented.replace(false) {
            // SAFETY: PhotosUI calls the delegate on the main thread; detaching prevents duplicate events.
            unsafe { picker.setDelegate(None) };
            picker.dismissViewControllerAnimated_completion(true, None);
        }
    }

    fn dismiss_if_presented(&self, picker: &NativePHPickerViewController) {
        if self.ivars().is_presented.replace(false) {
            // SAFETY: `MainThreadMarker` and the non-Send marker confine session cleanup to main.
            unsafe { picker.setDelegate(None) };
            picker.dismissViewControllerAnimated_completion(true, None);
        }
    }

    fn load_image(&self, provider: Retained<NSItemProvider>, type_identifier: Retained<NSString>) {
        let completion = self.ivars().completion.clone_for_provider();
        let max_bytes = self.ivars().max_bytes;
        let handler = RcBlock::new(move |url: *mut NSURL, native_error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Some(error) = error_ptr(native_error).map(error_from_native) {
                    completion.finish(IosPhotoPickerEvent::Failed(error));
                    return;
                }
                let Some(url) = NonNull::new(url) else {
                    completion.finish(IosPhotoPickerEvent::Failed(
                        IosPhotoPickerError::ImageRepresentationUnavailable,
                    ));
                    return;
                };
                let Some(source) = provider_file_path(url) else {
                    completion.finish(IosPhotoPickerEvent::Failed(
                        IosPhotoPickerError::ImageRepresentationUnavailable,
                    ));
                    return;
                };
                // SAFETY: Foundation returns the app's private temporary directory as an NSString.
                let temporary_root = PathBuf::from(NSTemporaryDirectory().to_string());
                match copy_provider_file(&source, &temporary_root, max_bytes, || {
                    completion.is_loading()
                }) {
                    Ok(image) => {
                        completion.finish(IosPhotoPickerEvent::Selected(image));
                    }
                    Err(CopyError::Cancelled) => {}
                    Err(error) => {
                        completion.finish(IosPhotoPickerEvent::Failed(copy_error(error)));
                    }
                }
            }));
            if result.is_err() {
                completion.finish(IosPhotoPickerEvent::Failed(
                    IosPhotoPickerError::CopyFailed {
                        message: String::from("Rust panic while copying PhotosUI provider data"),
                    },
                ));
            }
        });
        // SAFETY: The escaping block captures only Arc-backed Send+Sync completion state and a
        // fixed byte ceiling. It contains panics and copies the provider URL before this block exits.
        let progress = unsafe {
            provider.loadFileRepresentationForTypeIdentifier_completionHandler(
                &type_identifier,
                &handler,
            )
        };
        *self.ivars().progress.borrow_mut() = Some(progress);
    }

    fn cancel(&self) {
        self.ivars()
            .completion
            .finish(IosPhotoPickerEvent::Cancelled);
        if let Some(progress) = self.ivars().progress.borrow_mut().take() {
            progress.cancel();
        }
    }

    fn detach(&self) {
        self.ivars().completion.detach();
        if let Some(progress) = self.ivars().progress.borrow_mut().take() {
            progress.cancel();
        }
    }
}

fn error_from_native(error: NonNull<NSError>) -> IosPhotoPickerError {
    // SAFETY: NSError is valid for the duration of NSItemProvider's completion callback.
    let error = unsafe { error.as_ref() };
    IosPhotoPickerError::Provider {
        domain: error.domain().to_string(),
        code: i64::try_from(error.code()).unwrap_or(i64::MAX),
    }
}

fn error_ptr(error: *mut NSError) -> Option<NonNull<NSError>> {
    NonNull::new(error)
}

fn provider_file_path(url: NonNull<NSURL>) -> Option<PathBuf> {
    // SAFETY: The NSURL pointer is valid throughout this synchronous provider callback.
    let url = unsafe { url.as_ref() };
    if !url.isFileURL() {
        return None;
    }
    // SAFETY: NSURL.fileSystemRepresentation returns a NUL-terminated filesystem path valid
    // while `url` remains alive; the bytes are copied into PathBuf before the callback returns.
    let representation: NonNull<c_char> = url.fileSystemRepresentation();
    let bytes = unsafe { CStr::from_ptr(representation.as_ptr()) }.to_bytes();
    Some(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
}

fn copy_error(error: CopyError) -> IosPhotoPickerError {
    match error {
        CopyError::InvalidByteLimit => IosPhotoPickerError::InvalidByteLimit,
        CopyError::NotRegularFile | CopyError::EmptyFile => {
            IosPhotoPickerError::InvalidProviderFile
        }
        CopyError::TooLarge { limit } => IosPhotoPickerError::ImageTooLarge { limit },
        CopyError::Cancelled => IosPhotoPickerError::InvalidProviderFile,
        CopyError::Io(message) => IosPhotoPickerError::CopyFailed { message },
    }
}

extern_protocol!(
    /// The private-to-this-crate Objective-C binding for the public PhotosUI delegate protocol.
    ///
    /// # Safety
    ///
    /// Implementations must preserve the selector and main-thread callback contract in Apple's SDK.
    #[allow(non_snake_case)]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait NativePHPickerViewControllerDelegate:
        NSObjectProtocol + MainThreadOnly
    {
        #[optional]
        #[unsafe(method(picker:didFinishPicking:))]
        unsafe fn picker_didFinishPicking(
            &self,
            picker: &NativePHPickerViewController,
            results: &NSArray<PHPickerResult>,
        );
    }
);

extern_class!(
    /// Private-to-this-crate binding for the public iOS PhotosUI picker controller.
    #[unsafe(super(UIViewController))]
    #[thread_kind = MainThreadOnly]
    #[name = "PHPickerViewController"]
    pub struct NativePHPickerViewController;
);

#[allow(non_snake_case)]
impl NativePHPickerViewController {
    extern_methods!(
        // SAFETY: Mirrors public `initWithConfiguration:` from the iOS PhotosUI SDK header.
        #[unsafe(method(initWithConfiguration:))]
        #[unsafe(method_family = init)]
        pub unsafe fn initWithConfiguration(
            this: Allocated<Self>,
            configuration: &PHPickerConfiguration,
        ) -> Retained<Self>;

        // SAFETY: Mirrors the public weak delegate property from the iOS PhotosUI SDK header.
        #[unsafe(method(setDelegate:))]
        pub unsafe fn setDelegate(
            &self,
            delegate: Option<&ProtocolObject<dyn NativePHPickerViewControllerDelegate>>,
        );
    );
}
