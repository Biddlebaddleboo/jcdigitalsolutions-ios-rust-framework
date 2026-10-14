use alloc::rc::Rc;
use core::marker::PhantomData;

use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_foundation::{
    NSArray, NSObject, NSObjectProtocol, NSOperatingSystemVersion, NSProcessInfo, NSString, NSURL,
};
use objc2_ui_kit::{
    UIDocumentPickerDelegate, UIDocumentPickerViewController, UIView, UIViewController,
};
use objc2_uniform_type_identifiers::UTType;

use crate::document_import_state::{
    DocumentImportCompletion, accept_start, validate_selection_count,
};
pub use crate::document_import_state::{DocumentImportError, DocumentImportStartError};

/// A terminal event from UIKit's document-open picker.
pub enum DocumentImportResult {
    /// The user selected one original, security-scoped document URL.
    Selected(Retained<NSURL>),
    /// The user cancelled or dismissed the picker without selecting a document.
    Cancelled,
    /// UIKit reported a selection count other than one.
    Failed(DocumentImportError),
}

/// A main-thread-bound session for user-mediated open-in-place document selection.
///
/// The caller retains the selected URL and owns security-scope start/stop and all file I/O. This
/// session does not read, copy, persist, or coordinate the selected document.
pub struct IosDocumentImportSession<'ctx> {
    presenter: &'ctx UIViewController,
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
    active_operation: Option<DocumentImportOperation>,
}

struct DocumentImportOperation {
    picker: Retained<UIDocumentPickerViewController>,
    completion: DocumentImportCompletion<DocumentImportResult>,
    _delegate: Retained<DocumentImportDelegate>,
}

impl Drop for DocumentImportOperation {
    fn drop(&mut self) {
        self.completion.detach();
        self.picker.setDelegate(None);
    }
}

struct DocumentImportDelegateIvars {
    completion: DocumentImportCompletion<DocumentImportResult>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - This object is main-thread-only and contains only Rc-backed callback state.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DocumentImportDelegateIvars]
    struct DocumentImportDelegate;

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for DocumentImportDelegate {}

    // SAFETY: UIKit calls this delegate on the main thread. Completion is one-shot and contains
    // callback panics before returning through the Objective-C boundary.
    #[allow(non_snake_case)]
    unsafe impl UIDocumentPickerDelegate for DocumentImportDelegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        fn documentPicker_didPickDocumentsAtURLs(
            &self,
            picker: &UIDocumentPickerViewController,
            urls: &NSArray<NSURL>,
        ) {
            match validate_selection_count(urls.len()) {
                Ok(()) => self.finish(
                    picker,
                    DocumentImportResult::Selected(urls.objectAtIndex(0)),
                ),
                Err(error) => self.finish(picker, DocumentImportResult::Failed(error)),
            }
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        fn documentPickerWasCancelled(&self, picker: &UIDocumentPickerViewController) {
            self.finish(picker, DocumentImportResult::Cancelled);
        }
    }
);

impl DocumentImportDelegate {
    fn new(
        marker: MainThreadMarker,
        completion: DocumentImportCompletion<DocumentImportResult>,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(DocumentImportDelegateIvars { completion });
        // SAFETY: The class derives from NSObject and its ivars are set before NSObject init.
        unsafe { msg_send![super(this), init] }
    }

    fn finish(&self, picker: &UIDocumentPickerViewController, result: DocumentImportResult) {
        // SAFETY: This retain keeps delegate state alive if the callback drops the session.
        let receiver = unsafe {
            Retained::retain(self as *const Self as *mut Self)
                .expect("live document-import delegate cannot be nil")
        };
        picker.setDelegate(None);
        self.ivars().completion.complete(result);
        drop(receiver);
    }
}

impl<'ctx> IosDocumentImportSession<'ctx> {
    /// Creates a document-import session for a caller-owned presenter.
    ///
    /// Keep the presenter alive and attached to a window before [`Self::start`]. The session is
    /// neither `Send` nor `Sync` and must be used and dropped on UIKit's main thread.
    pub fn new(main_thread: MainThread, presenter: &'ctx UIViewController) -> Self {
        Self {
            presenter,
            _main_thread: main_thread,
            _not_send: PhantomData,
            active_operation: None,
        }
    }

    /// Presents the iOS 14+ picker for one user-selected document that keeps its original URL.
    ///
    /// `content_type_identifiers` contains exact Uniform Type Identifier strings, such as
    /// `public.plain-text` or a caller-declared type. UIKit returns an original security-scoped
    /// URL, not an app-owned copy. Start `ios_files::IosSecurityScopedAccess` before file access;
    /// use `ios_files::IosFileCoordinator` when coordinated provider access is required. A
    /// successful return means only that UIKit's presentation request was issued. Start failures
    /// return the exact uncalled callback; cancel/drop detaches it but does not dismiss the picker.
    pub fn start<F>(
        &mut self,
        content_type_identifiers: &[&str],
        completion: F,
    ) -> Result<(), DocumentImportStartError<F>>
    where
        F: FnOnce(DocumentImportResult) + 'static,
    {
        let main_thread_marker = MainThreadMarker::new();
        let has_active_operation = self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active());
        let completion = accept_start(
            !content_type_identifiers.is_empty(),
            has_active_operation,
            || {
                if main_thread_marker.is_none() {
                    return Err(DocumentImportError::MainThreadUnavailable);
                }
                if !document_import_api_available() {
                    return Err(DocumentImportError::UnsupportedOsVersion);
                }
                self.preflight()
            },
            completion,
        )?;
        let Some(marker) = MainThreadMarker::new() else {
            return Err(DocumentImportStartError {
                error: DocumentImportError::MainThreadUnavailable,
                completion,
            });
        };

        let mut native_types = Vec::with_capacity(content_type_identifiers.len());
        for identifier in content_type_identifiers {
            let identifier = NSString::from_str(identifier);
            let Some(content_type) = UTType::typeWithIdentifier(&identifier) else {
                return Err(DocumentImportStartError {
                    error: DocumentImportError::InvalidContentTypeIdentifier,
                    completion,
                });
            };
            native_types.push(content_type);
        }
        let native_type_refs = native_types
            .iter()
            .map(|content_type| &**content_type)
            .collect::<Vec<_>>();
        let native_types = NSArray::from_slice(&native_type_refs);

        drop(self.active_operation.take());
        let completion = DocumentImportCompletion::new(completion);
        let delegate = DocumentImportDelegate::new(marker, completion.clone());
        let picker = UIDocumentPickerViewController::initForOpeningContentTypes_asCopy(
            UIDocumentPickerViewController::alloc(marker),
            &native_types,
            false,
        );
        picker.setAllowsMultipleSelection(false);
        picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        self.active_operation = Some(DocumentImportOperation {
            picker: picker.clone(),
            completion,
            _delegate: delegate,
        });
        self.presenter
            .presentViewController_animated_completion(&picker, true, None);
        Ok(())
    }

    /// Detaches the pending callback without dismissing UIKit's picker.
    ///
    /// Returns `true` only if a callback was still pending. A new start remains subject to the
    /// presenter's native presentation preflight until UIKit dismisses the picker.
    pub fn cancel(&mut self) -> bool {
        let active = self
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.completion.is_active());
        drop(self.active_operation.take());
        active
    }

    fn preflight(&self) -> Result<(), DocumentImportError> {
        if self.presenter.isBeingPresented()
            || self.presenter.isBeingDismissed()
            || self.presenter.presentedViewController().is_some()
        {
            return Err(DocumentImportError::InvalidPresenter);
        }
        let view: Retained<UIView> = self
            .presenter
            .viewIfLoaded()
            .ok_or(DocumentImportError::InvalidPresenter)?;
        if view.window().is_none() {
            return Err(DocumentImportError::InvalidPresenter);
        }
        Ok(())
    }
}

fn document_import_api_available() -> bool {
    NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(NSOperatingSystemVersion {
        majorVersion: 14,
        minorVersion: 0,
        patchVersion: 0,
    })
}
