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

    /// Drops future interest but leaves callback state valid until completion.
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
