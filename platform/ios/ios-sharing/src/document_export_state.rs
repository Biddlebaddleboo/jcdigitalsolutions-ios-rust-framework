use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// A terminal event reported by UIKit's document picker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DocumentExportResult {
    /// UIKit called the document-picker success delegate method.
    Exported,
    /// UIKit called the document-picker cancellation delegate method.
    Cancelled,
}

/// Why a document export request was rejected before UIKit presentation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DocumentExportError {
    /// The source is not a local file URL.
    InvalidSourceUrl,
    /// The presenter is not attached to a window or cannot present another controller.
    InvalidPresenter,
    /// This session already has an operation awaiting a UIKit terminal event.
    AlreadyExists,
    /// The current call is not on UIKit's main thread.
    #[cfg(all(target_os = "ios", feature = "document-export"))]
    MainThreadUnavailable,
}

/// A synchronous document-export start failure that returns the completion callback.
pub struct DocumentExportStartError<F> {
    /// The reason UIKit presentation was not requested.
    pub error: DocumentExportError,
    /// The callback remains owned by the caller after rejection.
    pub completion: F,
}

impl<F> core::fmt::Debug for DocumentExportStartError<F> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DocumentExportStartError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

pub(crate) fn accept_start<F>(
    is_local_file_url: bool,
    already_active: bool,
    preflight: impl FnOnce() -> Result<(), DocumentExportError>,
    completion: F,
) -> Result<F, DocumentExportStartError<F>> {
    let result = if !is_local_file_url {
        Err(DocumentExportError::InvalidSourceUrl)
    } else if already_active {
        Err(DocumentExportError::AlreadyExists)
    } else {
        preflight()
    };
    match result {
        Ok(()) => Ok(completion),
        Err(error) => Err(DocumentExportStartError { error, completion }),
    }
}

pub(crate) const fn is_local_file_url(
    is_file_url: bool,
    has_nonempty_host: bool,
    has_absolute_path: bool,
) -> bool {
    is_file_url && !has_nonempty_host && has_absolute_path
}

type Callback = Box<dyn FnOnce(DocumentExportResult) + 'static>;

#[derive(Clone)]
pub(crate) struct DocumentExportCompletion {
    callback: Rc<RefCell<Option<Callback>>>,
}

impl DocumentExportCompletion {
    pub(crate) fn new(completion: impl FnOnce(DocumentExportResult) + 'static) -> Self {
        Self {
            callback: Rc::new(RefCell::new(Some(Box::new(completion)))),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.callback.borrow().is_some()
    }

    pub(crate) fn detach(&self) -> bool {
        self.callback.borrow_mut().take().is_some()
    }

    pub(crate) fn complete(&self, result: DocumentExportResult) -> bool {
        let Some(callback) = self.callback.borrow_mut().take() else {
            return false;
        };
        let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentExportCompletion, DocumentExportError, DocumentExportResult, accept_start,
        is_local_file_url,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn non_file_url_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let preflight_calls = Cell::new(false);
        let rejected = match accept_start(
            false,
            false,
            || {
                preflight_calls.set(true);
                Ok(())
            },
            move |_| {
                callback_calls.set(callback_calls.get() + 1);
            },
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("non-file URL was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::InvalidSourceUrl);
        assert_eq!(calls.get(), 0);
        assert!(!preflight_calls.get());
        (rejected.completion)(DocumentExportResult::Exported);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn remote_host_file_url_is_not_a_local_export_source() {
        assert!(!is_local_file_url(true, true, true));
        assert!(!is_local_file_url(true, false, false));
        assert!(is_local_file_url(true, false, true));
    }

    #[test]
    fn second_active_request_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let rejected = match accept_start(
            true,
            true,
            || panic!("active request must not preflight"),
            move |_| {
                callback_calls.set(callback_calls.get() + 1);
            },
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("second active export was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::AlreadyExists);
        assert_eq!(calls.get(), 0);
        (rejected.completion)(DocumentExportResult::Cancelled);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn invalid_presenter_rejection_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let rejected = match accept_start(
            true,
            false,
            || Err(DocumentExportError::InvalidPresenter),
            move |_| callback_calls.set(callback_calls.get() + 1),
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("invalid presenter was accepted"),
        };
        assert_eq!(rejected.error, DocumentExportError::InvalidPresenter);
        assert_eq!(calls.get(), 0);
        (rejected.completion)(DocumentExportResult::Exported);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn terminal_result_is_one_shot_and_detach_suppresses_late_events() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let completion = DocumentExportCompletion::new(move |_| {
            callback_calls.set(callback_calls.get() + 1);
        });
        assert!(completion.is_active());
        assert!(completion.complete(DocumentExportResult::Exported));
        assert!(!completion.is_active());
        assert!(!completion.complete(DocumentExportResult::Cancelled));
        assert_eq!(calls.get(), 1);

        let late_calls = Rc::new(Cell::new(0));
        let callback_late_calls = Rc::clone(&late_calls);
        let detached = DocumentExportCompletion::new(move |_| {
            callback_late_calls.set(callback_late_calls.get() + 1);
        });
        assert!(detached.detach());
        assert!(!detached.complete(DocumentExportResult::Cancelled));
        assert_eq!(late_calls.get(), 0);
    }

    #[test]
    fn callback_panic_is_contained_and_callback_cannot_run_twice() {
        let completion = DocumentExportCompletion::new(|_| panic!("callback panic"));
        assert!(completion.complete(DocumentExportResult::Cancelled));
        assert!(!completion.complete(DocumentExportResult::Exported));
    }
}
