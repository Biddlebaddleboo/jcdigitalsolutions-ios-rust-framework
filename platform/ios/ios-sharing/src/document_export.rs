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

use crate::document_export_state::{
    DocumentExportCompletion, StagedExportFile, StagedExportLease, accept_start, is_local_file_url,
};
pub use crate::document_export_state::{
    DocumentExportError, DocumentExportFileError, DocumentExportResult, DocumentExportStartError,
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
    staged_source: Option<StagedExportLease>,
    _delegate: Retained<DocumentExportDelegate>,
}

impl Drop for DocumentExportOperation {
    fn drop(&mut self) {
        self.completion.detach();
        if let Some(staged_source) = &self.staged_source {
            staged_source.abandon();
        }
        self.picker.setDelegate(None);
    }
}

struct DocumentExportDelegateIvars {
    completion: DocumentExportCompletion,
    source_url: RefCell<Option<Retained<NSURL>>>,
    staged_source: Option<StagedExportLease>,
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
        staged_source: Option<StagedExportLease>,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(DocumentExportDelegateIvars {
            completion,
            source_url: RefCell::new(Some(source_url)),
            staged_source,
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
        if let Some(staged_source) = &self.ivars().staged_source {
            staged_source.finish();
        }
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
        self.start_source(source_url, None, completion)
    }

    /// Requests export of staged arbitrary bytes under a caller-selected file name.
    ///
    /// Construct [`IosDocumentExportFile`] with `from_bytes` before calling this method; staging
    /// is synchronous and may be done on a worker thread. The file name extension is the only
    /// document-type hint passed to UIKit. A rejected start returns the callback and removes the
    /// consumed staged file. Once accepted, the picker delegate owns the file through its terminal
    /// callback. If this session is cancelled or dropped first, the staged file is deliberately
    /// left in the app temporary directory because UIKit may still be reading it; the OS may purge
    /// it later. This does not prove provider durability or runtime save behavior.
    pub fn start_staged<F>(
        &mut self,
        source: IosDocumentExportFile,
        completion: F,
    ) -> Result<(), DocumentExportStartError<F>>
    where
        F: FnOnce(DocumentExportResult) + 'static,
    {
        let path = source
            .staged
            .path()
            .to_str()
            .expect("staged export paths are validated as UTF-8");
        let path = NSString::from_str(path);
        let source_url = NSURL::fileURLWithPath(&path);
        self.start_source(&source_url, Some(source.staged), completion)
    }

    fn start_source<F>(
        &mut self,
        source_url: &NSURL,
        staged_source: Option<StagedExportFile>,
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
        // alive through the terminal callback; staged-file ownership is held by the shared lease.
        let retained_source = unsafe {
            Retained::retain(source_url as *const NSURL as *mut NSURL)
                .expect("borrowed source NSURL cannot be nil")
        };
        let urls = NSArray::from_slice(&[retained_source.as_ref()]);
        let callback = DocumentExportCompletion::new(completion);
        let staged_source = staged_source.map(StagedExportLease::new);
        let delegate = DocumentExportDelegate::new(
            marker,
            callback.clone(),
            retained_source,
            staged_source.clone(),
        );
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
            staged_source,
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

/// A validated, app-temporary file staged from caller-owned bytes.
///
/// Construction copies the exact bytes synchronously and may run off the UIKit main thread.
/// Dropping an unsubmitted value removes its temporary file. After accepted picker presentation,
/// `start_staged` transfers cleanup ownership to the session and delegate.
pub struct IosDocumentExportFile {
    staged: StagedExportFile,
}

impl IosDocumentExportFile {
    /// Stages `bytes` under `file_name`, preserving every byte including embedded NUL values.
    ///
    /// `file_name` must be one nonempty path component other than `.` or `..`; separators, NUL,
    /// and control characters are rejected. The source is written to a unique directory under the
    /// app temporary directory with create-new semantics. This synchronous call may write a large
    /// payload, so callers may construct the value on a worker thread and pass it to the main-thread
    /// session later.
    pub fn from_bytes(file_name: &str, bytes: &[u8]) -> Result<Self, DocumentExportFileError> {
        Ok(Self {
            staged: StagedExportFile::from_bytes(file_name, bytes)?,
        })
    }
}
