use alloc::rc::Rc;
use core::cell::RefCell;
use core::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};

use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL};
use objc2_ui_kit::{
    UIDocumentPickerDelegate, UIDocumentPickerViewController, UIView, UIViewController,
};

use crate::document_export_state::{DocumentExportCompletion, accept_start, is_local_file_url};
pub use crate::document_export_state::{
    DocumentExportError, DocumentExportResult, DocumentExportStartError,
};

const EXPORT_AS_COPY: bool = true;

/// A main-thread-bound session for user-mediated export of one local file.
///
/// Keep the borrowed presenter alive for the session lifetime and call every method on the main
/// thread. UIKit receives a copy-export request, so it does not move or consume the source file.
/// The source `NSURL` is retained until a success/cancel event or session teardown. Dropping or
/// cancelling detaches the callback but does not dismiss the picker.
pub struct IosDocumentExportSession<'ctx> {
    presenter: &'ctx UIViewController,
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
    active_operation: Option<DocumentExportOperation>,
}

struct DocumentExportOperation {
    picker: Retained<UIDocumentPickerViewController>,
    completion: DocumentExportCompletion,
    _delegate: Retained<DocumentExportDelegate>,
}

impl Drop for DocumentExportOperation {
    fn drop(&mut self) {
        self.completion.detach();
        self.picker.setDelegate(None);
    }
}

struct DocumentExportDelegateIvars {
    completion: DocumentExportCompletion,
    source_url: RefCell<Option<Retained<NSURL>>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The object is main-thread-only and contains only Rc-backed, main-thread state.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DocumentExportDelegateIvars]
    struct DocumentExportDelegate;

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for DocumentExportDelegate {}

    // SAFETY: UIKit calls this delegate on the main thread. Completion state is one-shot, and
    // callback panics are contained before returning through the Objective-C boundary.
    #[allow(non_snake_case)]
    unsafe impl UIDocumentPickerDelegate for DocumentExportDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn documentPicker_didPickDocumentsAtURLs(
            &self,
            picker: &UIDocumentPickerViewController,
            _urls: &NSArray<NSURL>,
        ) {
            self.finish(picker, DocumentExportResult::Exported);
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        fn documentPickerWasCancelled(&self, picker: &UIDocumentPickerViewController) {
            self.finish(picker, DocumentExportResult::Cancelled);
        }
    }
);

impl DocumentExportDelegate {
    fn new(
        marker: MainThreadMarker,
        completion: DocumentExportCompletion,
        source_url: Retained<NSURL>,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(DocumentExportDelegateIvars {
            completion,
            source_url: RefCell::new(Some(source_url)),
        });
        // SAFETY: The class derives from NSObject and its ivars are set before NSObject init.
        unsafe { msg_send![super(this), init] }
    }

    fn finish(&self, picker: &UIDocumentPickerViewController, result: DocumentExportResult) {
        // SAFETY: This retain keeps the delegate ivars alive if the completion drops the session.
        let receiver = unsafe {
            Retained::retain(self as *const Self as *mut Self)
                .expect("live document-picker delegate cannot be nil")
        };
        let source_url = self.ivars().source_url.borrow_mut().take();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            picker.setDelegate(None);
            self.ivars().completion.complete(result);
        }));
        drop(source_url);
        drop(receiver);
    }
}

impl<'ctx> IosDocumentExportSession<'ctx> {
    /// Creates a session for a caller-owned presenter.
    ///
    /// The presenter must remain alive and attached to a window when [`Self::start`] is called.
    /// The returned session is neither `Send` nor `Sync` and must be dropped on the main thread.
    pub fn new(main_thread: MainThread, presenter: &'ctx UIViewController) -> Self {
        Self {
            presenter,
            _main_thread: main_thread,
            _not_send: PhantomData,
            active_operation: None,
        }
    }

    /// Requests export of one caller-owned local file URL using copy semantics.
    ///
    /// The caller must keep the source file readable and stable until `completion` runs. A
    /// successful return means UIKit presentation was requested; it does not prove visible UI,
    /// provider durability, physical persistence, or save completion beyond UIKit's success
    /// delegate event. Input/presenter/active-operation failures return synchronously with the
    /// callback unconsumed. A callback panic is caught and does not cross Objective-C.
    pub fn start<F>(
        &mut self,
        source_url: &NSURL,
        completion: F,
    ) -> Result<(), DocumentExportStartError<F>>
    where
        F: FnOnce(DocumentExportResult) + 'static,
    {
        let local_path_prefix = NSString::from_str("/");
        let source_is_local = is_local_file_url(
            source_url.isFileURL(),
            source_url.host().is_some_and(|host| host.length() != 0),
            source_url
                .path()
                .is_some_and(|path| path.hasPrefix(&local_path_prefix)),
        );
        let is_active = self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active());
        let completion = accept_start(source_is_local, is_active, || self.preflight(), completion)?;
        let Some(marker) = MainThreadMarker::new() else {
            return Err(DocumentExportStartError {
                error: DocumentExportError::MainThreadUnavailable,
                completion,
            });
        };

        drop(self.active_operation.take());
        // SAFETY: `source_url` is a live borrowed Objective-C object. Retain keeps the exact NSURL
        // alive through the terminal callback; the source file remains caller-owned and untouched.
        let retained_source = unsafe {
            Retained::retain(source_url as *const NSURL as *mut NSURL)
                .expect("borrowed source NSURL cannot be nil")
        };
        let urls = NSArray::from_slice(&[retained_source.as_ref()]);
        let callback = DocumentExportCompletion::new(completion);
        let delegate = DocumentExportDelegate::new(marker, callback.clone(), retained_source);
        let picker = UIDocumentPickerViewController::initForExportingURLs_asCopy(
            UIDocumentPickerViewController::alloc(marker),
            &urls,
            EXPORT_AS_COPY,
        );
        let protocol_delegate = ProtocolObject::from_ref(&*delegate);
        picker.setDelegate(Some(protocol_delegate));
        self.active_operation = Some(DocumentExportOperation {
            picker: picker.clone(),
            completion: callback,
            _delegate: delegate,
        });
        self.presenter
            .presentViewController_animated_completion(&picker, true, None);
        Ok(())
    }

    /// Detaches the pending callback without dismissing UIKit's document picker.
    ///
    /// Returns `true` only if a callback was still pending. UIKit may continue to display the
    /// picker; a new start is rejected by presenter preflight until that controller is dismissed.
    pub fn cancel(&mut self) -> bool {
        let active = self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active());
        drop(self.active_operation.take());
        active
    }

    fn preflight(&self) -> Result<(), DocumentExportError> {
        if MainThreadMarker::new().is_none() {
            return Err(DocumentExportError::MainThreadUnavailable);
        }
        if self.presenter.isBeingPresented()
            || self.presenter.isBeingDismissed()
            || self.presenter.presentedViewController().is_some()
        {
            return Err(DocumentExportError::InvalidPresenter);
        }
        let view: Retained<UIView> = self
            .presenter
            .viewIfLoaded()
            .ok_or(DocumentExportError::InvalidPresenter)?;
        if view.window().is_none() {
            return Err(DocumentExportError::InvalidPresenter);
        }
        Ok(())
    }
}
