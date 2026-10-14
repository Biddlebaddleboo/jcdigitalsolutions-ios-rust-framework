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

use crate::copy::{CopyError, copy_provider_file, provider_file_len};
use crate::session_policy::{
    MultiPickerCompletion, PickerCompletion, SelectionDecision, selection_decision,
    validate_multi_limits,
};
use crate::{IosPhotoPickerError, IosPhotoPickerEvent};

/// A main-thread-bound PhotosUI image-selection session.
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
        Self::present_with_completion(
            main_thread,
            presenter,
            1,
            max_bytes,
            PickerCallback::Single(PickerCompletion::new(callback)),
        )
    }

    /// Presents the system image picker for a bounded selection of two or more images.
    ///
    /// `selection_limit` must be between [`crate::MIN_MULTI_IMAGE_SELECTION`] and
    /// [`crate::MAX_MULTI_IMAGE_SELECTION`], inclusive. `max_aggregate_bytes` must be nonzero and
    /// no greater than [`crate::MAX_MULTI_IMAGE_AGGREGATE_BYTES`]. Every selected provider file is copied
    /// byte-for-byte into app-temporary storage, and the total provider bytes are reserved before
    /// copying so concurrent transfers cannot exceed the aggregate ceiling. Results are returned
    /// in PhotosUI selection order. Any provider, file, copy, or aggregate-size failure fails the
    /// entire batch and drops all files already copied; other already-started provider requests
    /// may finish, but their late results are ignored. Cancellation emits `Cancelled` once and
    /// cancels all retained provider progress objects. Dropping the session cancels and detaches
    /// the callback, so late results are ignored and copied files are removed.
    pub fn present_multiple<F>(
        main_thread: MainThread,
        presenter: &UIViewController,
        selection_limit: u8,
        max_aggregate_bytes: u64,
        callback: F,
    ) -> Result<Self, IosPhotoPickerError>
    where
        F: FnOnce(IosPhotoPickerEvent) + Send + 'static,
    {
        validate_multi_limits(selection_limit, max_aggregate_bytes)?;
        Self::present_with_completion(
            main_thread,
            presenter,
            selection_limit,
            max_aggregate_bytes,
            PickerCallback::Multiple(MultiPickerCompletion::new(max_aggregate_bytes, callback)),
        )
    }

    fn present_with_completion(
        main_thread: MainThread,
        presenter: &UIViewController,
        selection_limit: u8,
        max_bytes: u64,
        completion: PickerCallback,
    ) -> Result<Self, IosPhotoPickerError> {
        let marker = main_thread.into_objc2();
        // SAFETY: These public PhotosUI constructors and setters are called on the UIKit main
        // thread. The configured selection limit is bounded and the current representation is preferred.
        let configuration = unsafe { PHPickerConfiguration::new() };
        unsafe {
            configuration.setSelectionLimit(isize::from(selection_limit));
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
        let delegate = PhotoPickerDelegate::new(marker, completion, selection_limit, max_bytes);
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
    completion: PickerCallback,
    progress: RefCell<Vec<Retained<NSProgress>>>,
    is_presented: Cell<bool>,
    selection_limit: u8,
    max_bytes: u64,
}

#[derive(Clone)]
enum PickerCallback {
    Single(PickerCompletion),
    Multiple(MultiPickerCompletion),
}

impl PickerCallback {
    fn clone_for_provider(&self) -> Self {
        self.clone()
    }

    fn begin_loading(&self, count: usize) -> bool {
        match self {
            Self::Single(completion) => count == 1 && completion.begin_loading(),
            Self::Multiple(completion) => completion.begin_loading(count),
        }
    }

    fn is_loading(&self) -> bool {
        match self {
            Self::Single(completion) => completion.is_loading(),
            Self::Multiple(completion) => completion.is_loading(),
        }
    }

    fn is_multiple(&self) -> bool {
        matches!(self, Self::Multiple(_))
    }

    fn reserve(&self, index: usize, byte_len: u64) -> bool {
        match self {
            Self::Multiple(completion) => completion.reserve(index, byte_len),
            Self::Single(_) => false,
        }
    }

    fn finish_item(&self, index: usize, image: crate::IosPickedImage) {
        match self {
            Self::Single(completion) if index == 0 => {
                completion.finish(IosPhotoPickerEvent::Selected(image));
            }
            Self::Single(completion) => {
                drop(image);
                completion.finish(IosPhotoPickerEvent::Failed(
                    IosPhotoPickerError::UnexpectedSelectionCount {
                        actual: u64::try_from(index.saturating_add(1)).unwrap_or(u64::MAX),
                    },
                ));
            }
            Self::Multiple(completion) => {
                completion.finish_item(index, image);
            }
        }
    }

    fn fail(&self, error: IosPhotoPickerError) {
        match self {
            Self::Single(completion) => {
                completion.finish(IosPhotoPickerEvent::Failed(error));
            }
            Self::Multiple(completion) => {
                completion.fail(error);
            }
        }
    }

    fn cancel(&self) {
        match self {
            Self::Single(completion) => {
                completion.finish(IosPhotoPickerEvent::Cancelled);
            }
            Self::Multiple(completion) => {
                completion.cancel();
            }
        }
    }

    fn detach(&self) {
        match self {
            Self::Single(completion) => {
                completion.detach();
            }
            Self::Multiple(completion) => {
                completion.detach();
            }
        }
    }
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
                self.ivars()
                    .completion
                    .fail(IosPhotoPickerError::CopyFailed {
                        message: String::from("Rust panic while handling PhotosUI selection"),
                    });
            }
        }
    }
);

impl PhotoPickerDelegate {
    fn new(
        marker: MainThreadMarker,
        completion: PickerCallback,
        selection_limit: u8,
        max_bytes: u64,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(PhotoPickerDelegateIvars {
            completion,
            progress: RefCell::new(Vec::new()),
            is_presented: Cell::new(true),
            selection_limit,
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
        match selection_decision(
            u64::try_from(count).unwrap_or(u64::MAX),
            self.ivars().selection_limit,
        ) {
            SelectionDecision::Cancelled => {
                self.dismiss(picker);
                self.ivars().completion.cancel();
                return;
            }
            SelectionDecision::UnexpectedCount(actual) => {
                self.dismiss(picker);
                self.ivars()
                    .completion
                    .fail(IosPhotoPickerError::UnexpectedSelectionCount { actual });
                return;
            }
            SelectionDecision::Selected => {}
        }
        if !self.ivars().completion.begin_loading(count) {
            self.dismiss(picker);
            return;
        }
        let type_identifier = NSString::from_str("public.image");
        self.dismiss(picker);
        for index in 0..count {
            if !self.ivars().completion.is_loading() {
                return;
            }
            // SAFETY: The result index is within the borrowed PhotosUI result array.
            let result = results.objectAtIndex(index);
            // SAFETY: PHPickerResult.itemProvider is a public retained property.
            let provider = unsafe { result.itemProvider() };
            if !provider.hasItemConformingToTypeIdentifier(&type_identifier) {
                self.ivars()
                    .completion
                    .fail(IosPhotoPickerError::ImageRepresentationUnavailable);
                return;
            }
            self.load_image(index, provider, type_identifier.clone());
        }
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

    fn load_image(
        &self,
        index: usize,
        provider: Retained<NSItemProvider>,
        type_identifier: Retained<NSString>,
    ) {
        let completion = self.ivars().completion.clone_for_provider();
        let max_bytes = self.ivars().max_bytes;
        let handler = RcBlock::new(move |url: *mut NSURL, native_error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                if !completion.is_loading() {
                    return;
                }
                if let Some(error) = error_ptr(native_error).map(error_from_native) {
                    completion.fail(error);
                    return;
                }
                let Some(url) = NonNull::new(url) else {
                    completion.fail(IosPhotoPickerError::ImageRepresentationUnavailable);
                    return;
                };
                let Some(source) = provider_file_path(url) else {
                    completion.fail(IosPhotoPickerError::ImageRepresentationUnavailable);
                    return;
                };
                // SAFETY: Foundation returns the app's private temporary directory as an NSString.
                let temporary_root = PathBuf::from(NSTemporaryDirectory().to_string());
                let copy_limit = if completion.is_multiple() {
                    let byte_len = match provider_file_len(&source) {
                        Ok(byte_len) => byte_len,
                        Err(error) => {
                            completion.fail(copy_error(error));
                            return;
                        }
                    };
                    if !completion.reserve(index, byte_len) {
                        return;
                    }
                    byte_len
                } else {
                    max_bytes
                };
                match copy_provider_file(&source, &temporary_root, copy_limit, || {
                    completion.is_loading()
                }) {
                    Ok(image) => {
                        completion.finish_item(index, image);
                    }
                    Err(CopyError::Cancelled) => {}
                    Err(error) => {
                        completion.fail(copy_error(error));
                    }
                }
            }));
            if result.is_err() {
                completion.fail(IosPhotoPickerError::CopyFailed {
                    message: String::from("Rust panic while copying PhotosUI provider data"),
                });
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
        self.ivars().progress.borrow_mut().push(progress);
    }

    fn cancel(&self) {
        self.ivars().completion.cancel();
        for progress in self.ivars().progress.borrow_mut().drain(..) {
            progress.cancel();
        }
    }

    fn detach(&self) {
        self.ivars().completion.detach();
        for progress in self.ivars().progress.borrow_mut().drain(..) {
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
