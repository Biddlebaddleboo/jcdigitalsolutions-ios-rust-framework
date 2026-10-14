use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::convert::TryFrom;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Why a document import request was rejected or ended without one selected URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DocumentImportError {
    /// No allowed Uniform Type Identifier was supplied.
    EmptyContentTypes,
    /// UIKit could not create a native `UTType` from one identifier.
    InvalidContentTypeIdentifier,
    /// The presenter is not attached to a window or cannot present another controller.
    InvalidPresenter,
    /// This session already has an operation awaiting a UIKit terminal event.
    AlreadyExists,
    /// The current call is not on UIKit's main thread.
    MainThreadUnavailable,
    /// The modern document-open initializer requires iOS 14.0 or newer.
    UnsupportedOsVersion,
    /// UIKit returned a selection count other than one.
    UnexpectedSelectionCount {
        /// The number of URLs returned by UIKit, saturated at `u32::MAX`.
        actual: u32,
    },
}

/// A synchronous document-import start failure that returns the completion callback.
pub struct DocumentImportStartError<F> {
    /// The reason UIKit presentation was not requested.
    pub error: DocumentImportError,
    /// The callback remains owned by the caller after rejection.
    pub completion: F,
}

impl<F> core::fmt::Debug for DocumentImportStartError<F> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DocumentImportStartError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl<F> DocumentImportStartError<F> {
    /// Returns the typed rejection and the exact callback that was not called.
    pub fn into_parts(self) -> (DocumentImportError, F) {
        (self.error, self.completion)
    }
}

pub(crate) fn accept_start<F>(
    has_content_types: bool,
    already_active: bool,
    preflight: impl FnOnce() -> Result<(), DocumentImportError>,
    completion: F,
) -> Result<F, DocumentImportStartError<F>> {
    let result = if !has_content_types {
        Err(DocumentImportError::EmptyContentTypes)
    } else if already_active {
        Err(DocumentImportError::AlreadyExists)
    } else {
        preflight()
    };
    match result {
        Ok(()) => Ok(completion),
        Err(error) => Err(DocumentImportStartError { error, completion }),
    }
}

pub(crate) fn validate_selection_count(actual: usize) -> Result<(), DocumentImportError> {
    if actual == 1 {
        Ok(())
    } else {
        Err(DocumentImportError::UnexpectedSelectionCount {
            actual: u32::try_from(actual).unwrap_or(u32::MAX),
        })
    }
}

type Callback<E> = Box<dyn FnOnce(E) + 'static>;

pub(crate) struct DocumentImportCompletion<E> {
    callback: Rc<RefCell<Option<Callback<E>>>>,
}

impl<E> Clone for DocumentImportCompletion<E> {
    fn clone(&self) -> Self {
        Self {
            callback: Rc::clone(&self.callback),
        }
    }
}

impl<E> DocumentImportCompletion<E> {
    pub(crate) fn new(completion: impl FnOnce(E) + 'static) -> Self {
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

    pub(crate) fn complete(&self, event: E) -> bool {
        let Some(callback) = self.callback.borrow_mut().take() else {
            return false;
        };
        let _ = catch_unwind(AssertUnwindSafe(|| callback(event)));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentImportCompletion, DocumentImportError, accept_start, validate_selection_count,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn empty_types_reject_before_preflight_and_return_callback_untouched() {
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
            move |_| callback_calls.set(callback_calls.get() + 1),
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("empty content type list was accepted"),
        };
        assert_eq!(rejected.error, DocumentImportError::EmptyContentTypes);
        assert_eq!(calls.get(), 0);
        assert!(!preflight_calls.get());
        (rejected.completion)(());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn active_operation_rejects_before_preflight_and_returns_callback_untouched() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let rejected = match accept_start(
            true,
            true,
            || panic!("active operation must not preflight"),
            move |_| callback_calls.set(callback_calls.get() + 1),
        ) {
            Err(rejected) => rejected,
            Ok(_) => panic!("second active import was accepted"),
        };
        assert_eq!(rejected.error, DocumentImportError::AlreadyExists);
        assert_eq!(calls.get(), 0);
        (rejected.completion)(());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn preflight_errors_return_callback_untouched() {
        for error in [
            DocumentImportError::MainThreadUnavailable,
            DocumentImportError::UnsupportedOsVersion,
            DocumentImportError::InvalidPresenter,
        ] {
            let calls = Rc::new(Cell::new(0));
            let callback_calls = Rc::clone(&calls);
            let rejected = match accept_start(
                true,
                false,
                || Err(error),
                move |_| callback_calls.set(callback_calls.get() + 1),
            ) {
                Err(rejected) => rejected,
                Ok(_) => panic!("invalid preflight was accepted"),
            };
            assert_eq!(rejected.error, error);
            assert_eq!(calls.get(), 0);
            (rejected.completion)(());
            assert_eq!(calls.get(), 1);
        }
    }

    #[test]
    fn terminal_event_is_one_shot_and_detach_suppresses_late_events() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let completion = DocumentImportCompletion::new(move |_| {
            callback_calls.set(callback_calls.get() + 1);
        });
        assert!(completion.is_active());
        assert!(completion.complete(()));
        assert!(!completion.is_active());
        assert!(!completion.complete(()));
        assert_eq!(calls.get(), 1);

        let late_calls = Rc::new(Cell::new(0));
        let callback_late_calls = Rc::clone(&late_calls);
        let detached = DocumentImportCompletion::new(move |_| {
            callback_late_calls.set(callback_late_calls.get() + 1);
        });
        assert!(detached.detach());
        assert!(!detached.complete(()));
        assert_eq!(late_calls.get(), 0);
    }

    #[test]
    fn unexpected_selection_counts_are_rejected_and_saturated() {
        assert_eq!(validate_selection_count(1), Ok(()));
        assert_eq!(
            validate_selection_count(0),
            Err(DocumentImportError::UnexpectedSelectionCount { actual: 0 })
        );
        assert_eq!(
            validate_selection_count(2),
            Err(DocumentImportError::UnexpectedSelectionCount { actual: 2 })
        );
        assert_eq!(
            validate_selection_count(usize::MAX),
            Err(DocumentImportError::UnexpectedSelectionCount { actual: u32::MAX })
        );
    }

    #[test]
    fn callback_panic_is_contained_and_state_detaches_first() {
        let completion = DocumentImportCompletion::new(|_| panic!("callback panic"));
        assert!(completion.complete(()));
        assert!(!completion.is_active());
        assert!(!completion.complete(()));
    }
}
