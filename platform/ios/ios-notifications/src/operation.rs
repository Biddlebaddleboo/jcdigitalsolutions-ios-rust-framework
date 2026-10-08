use core::task::{Context, Poll, Waker};
use std::sync::{Mutex, MutexGuard};

/// One native callback result and the future's current interest in that result.
pub(crate) struct CompletionCell<T, E> {
    state: Mutex<State<T, E>>,
}

struct State<T, E> {
    completed: bool,
    detached: bool,
    result: Option<Result<T, E>>,
    waker: Option<Waker>,
}

impl<T, E> CompletionCell<T, E> {
    /// Creates an incomplete cell with no callback or future interest.
    pub(crate) const fn new() -> Self {
        Self {
            state: Mutex::new(State {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    /// Publishes one terminal callback result and wakes an interested future.
    pub(crate) fn complete(&self, result: Result<T, E>) -> bool {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return false;
            }
            state.completed = true;
            if state.detached {
                None
            } else {
                state.result = Some(result);
                state.waker.take()
            }
        };
        if let Some(waker) = waker {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
        }
        true
    }

    /// Polls for one result or saves the current task's waker.
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Result<T, E>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
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

    /// Drops future interest but leaves native callback state valid until completion.
    pub(crate) fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State<T, E>> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
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

    fn context(waker: &Waker) -> Context<'_> {
        Context::from_waker(waker)
    }

    #[test]
    fn one_callback_result_wakes_and_wins() {
        let cell = CompletionCell::<u8, u8>::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut cx = context(&waker);
        assert!(cell.poll(&mut cx).is_pending());
        assert!(cell.complete(Ok(7)));
        assert!(!cell.complete(Ok(8)));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 1);
        assert!(matches!(cell.poll(&mut cx), Poll::Ready(Ok(7))));
    }

    #[test]
    fn detached_future_discards_late_callback_without_waking() {
        let cell = Arc::new(CompletionCell::<u8, u8>::new());
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut cx = context(&waker);
        assert!(cell.poll(&mut cx).is_pending());
        cell.detach();
        assert!(cell.complete(Ok(9)));
        assert!(!cell.complete(Ok(10)));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn completion_and_detach_race_has_one_terminal_result() {
        let cell = Arc::new(CompletionCell::<u8, u8>::new());
        let barrier = Arc::new(Barrier::new(2));
        let thread_cell = cell.clone();
        let thread_barrier = barrier.clone();
        let callback = std::thread::spawn(move || {
            thread_barrier.wait();
            thread_cell.complete(Ok(11))
        });
        barrier.wait();
        cell.detach();
        assert!(callback.join().unwrap());
        assert!(!cell.complete(Ok(12)));
    }
}
