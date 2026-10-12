use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::IosPhotoPickerEvent;

/// An error from PhotosUI selection or bounded provider-file copying.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IosPhotoPickerError {
    /// The caller supplied a zero-byte ceiling.
    InvalidByteLimit,
    /// The selected provider did not supply an image file representation.
    ImageRepresentationUnavailable,
    /// The selected result count violated the single-selection contract.
    UnexpectedSelectionCount {
        /// The reported number of selected results.
        actual: u64,
    },
    /// The provider returned a native error.
    Provider {
        /// The native NSError domain.
        domain: String,
        /// The native NSError code.
        code: i64,
    },
    /// The provider file was empty or was not a regular file.
    InvalidProviderFile,
    /// The copied provider file exceeded the caller's byte ceiling.
    ImageTooLarge {
        /// The caller-supplied maximum byte length.
        limit: u64,
    },
    /// The image file could not be copied into app-temporary storage.
    CopyFailed {
        /// A stable diagnostic from the Rust filesystem operation.
        message: String,
    },
}

type Callback = Box<dyn FnOnce(IosPhotoPickerEvent) + Send + 'static>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Pending,
    Loading,
    Terminal,
    Detached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectionDecision {
    Cancelled,
    Selected,
    UnexpectedCount(u64),
}

pub(crate) fn selection_decision(count: u64) -> SelectionDecision {
    match count {
        0 => SelectionDecision::Cancelled,
        1 => SelectionDecision::Selected,
        actual => SelectionDecision::UnexpectedCount(actual),
    }
}

struct Inner {
    phase: Phase,
    callback: Option<Callback>,
}

/// Shared terminal-state arbitration for picker and background provider callbacks.
pub(crate) struct PickerCompletion {
    inner: Arc<Mutex<Inner>>,
}

impl PickerCompletion {
    pub(crate) fn new(callback: impl FnOnce(IosPhotoPickerEvent) + Send + 'static) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                phase: Phase::Pending,
                callback: Some(Box::new(callback)),
            })),
        }
    }

    pub(crate) fn clone_for_provider(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn begin_loading(&self) -> bool {
        let mut inner = lock(&self.inner);
        if inner.phase != Phase::Pending {
            return false;
        }
        inner.phase = Phase::Loading;
        true
    }

    pub(crate) fn is_loading(&self) -> bool {
        lock(&self.inner).phase == Phase::Loading
    }

    pub(crate) fn finish(&self, event: IosPhotoPickerEvent) -> bool {
        let callback = {
            let mut inner = lock(&self.inner);
            if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
                return false;
            }
            inner.phase = Phase::Terminal;
            inner.callback.take()
        };
        if let Some(callback) = callback {
            let _ = catch_unwind(AssertUnwindSafe(|| callback(event)));
            true
        } else {
            false
        }
    }

    pub(crate) fn detach(&self) -> bool {
        let mut inner = lock(&self.inner);
        if !matches!(inner.phase, Phase::Pending | Phase::Loading) {
            return false;
        }
        inner.phase = Phase::Detached;
        inner.callback.take().is_some()
    }
}

fn lock(inner: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::{PickerCompletion, SelectionDecision, selection_decision};
    use crate::IosPhotoPickerEvent;
    use std::sync::{Arc, Mutex};

    #[test]
    fn cancellation_and_late_provider_result_settle_once() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion = PickerCompletion::new(move |event| captured.lock().unwrap().push(event));
        assert!(completion.begin_loading());
        assert!(completion.finish(IosPhotoPickerEvent::Cancelled));
        assert!(!completion.finish(IosPhotoPickerEvent::Cancelled));
        assert_eq!(events.lock().unwrap().len(), 1);
    }

    #[test]
    fn session_drop_detaches_and_ignores_late_provider_completion() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let completion = PickerCompletion::new(move |event| captured.lock().unwrap().push(event));
        let provider = completion.clone_for_provider();
        assert!(completion.begin_loading());
        assert!(completion.detach());
        assert!(!provider.is_loading());
        assert!(!provider.finish(IosPhotoPickerEvent::Cancelled));
        assert!(events.lock().unwrap().is_empty());
    }

    #[test]
    fn callback_panic_is_contained_after_terminal_state() {
        let completion = PickerCompletion::new(|_| panic!("callback failure"));
        assert!(completion.finish(IosPhotoPickerEvent::Cancelled));
        assert!(!completion.finish(IosPhotoPickerEvent::Cancelled));
    }

    #[test]
    fn empty_single_and_multiple_result_counts_are_distinct() {
        assert_eq!(selection_decision(0), SelectionDecision::Cancelled);
        assert_eq!(selection_decision(1), SelectionDecision::Selected);
        assert_eq!(selection_decision(2), SelectionDecision::UnexpectedCount(2));
    }
}
