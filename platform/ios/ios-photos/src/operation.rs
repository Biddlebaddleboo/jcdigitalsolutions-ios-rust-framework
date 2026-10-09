use core::task::{Context, Poll, Waker};
use framework_photos::PhotoLibraryAuthorizationStatus;
use std::sync::{Mutex, MutexGuard};

/// One native status result and the future's current interest in it.
pub(crate) struct CompletionCell {
    state: Mutex<State>,
}

struct State {
    completed: bool,
    detached: bool,
    status: Option<PhotoLibraryAuthorizationStatus>,
    waker: Option<Waker>,
}

impl CompletionCell {
    /// Creates an incomplete cell with no future interest.
    pub(crate) const fn new() -> Self {
        Self {
            state: Mutex::new(State {
                completed: false,
                detached: false,
                status: None,
                waker: None,
            }),
        }
    }

    /// Publishes one status and wakes the future after releasing the lock.
    pub(crate) fn complete(&self, status: PhotoLibraryAuthorizationStatus) -> bool {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return false;
            }
            state.completed = true;
            if state.detached {
                None
            } else {
                state.status = Some(status);
                state.waker.take()
            }
        };
        if let Some(waker) = waker {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
        }
        true
    }

    /// Returns one status or saves the current task's waker.
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<PhotoLibraryAuthorizationStatus> {
        let mut state = self.lock();
        if let Some(status) = state.status.take() {
            state.detached = true;
            state.waker = None;
            return Poll::Ready(status);
        }
        if !state.completed
            && !state.detached
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    /// Drops future interest and clears any stored status or waker.
    pub(crate) fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.status = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    struct WakeCount(AtomicUsize);

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn one_callback_status_wakes_and_wins() {
        let cell = CompletionCell::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        let mut context = Context::from_waker(&waker);
        assert!(cell.poll(&mut context).is_pending());
        assert!(cell.complete(PhotoLibraryAuthorizationStatus::Limited));
        assert!(!cell.complete(PhotoLibraryAuthorizationStatus::Authorized));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            cell.poll(&mut context),
            Poll::Ready(PhotoLibraryAuthorizationStatus::Limited)
        );
    }

    #[test]
    fn detached_future_discards_late_status_without_waking() {
        let cell = CompletionCell::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        let mut context = Context::from_waker(&waker);
        assert!(cell.poll(&mut context).is_pending());
        cell.detach();
        assert!(cell.complete(PhotoLibraryAuthorizationStatus::Limited));
        assert!(!cell.complete(PhotoLibraryAuthorizationStatus::Authorized));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
        assert!(cell.poll(&mut context).is_pending());
    }
}
