use core::task::{Context, Poll, Waker};
use framework_auth::AuthenticationError;
use std::sync::{Mutex, MutexGuard};

/// One native result and the future's current interest in it
pub(crate) struct CompletionCell {
    state: Mutex<State>,
}

struct State {
    completed: bool,
    detached: bool,
    result: Option<Result<(), AuthenticationError>>,
    waker: Option<Waker>,
}

impl CompletionCell {
    /// Creates an incomplete cell with no future interest
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

    /// Publishes one result and wakes the future after lock release
    pub(crate) fn complete(&self, result: Result<(), AuthenticationError>) -> bool {
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
            waker.wake();
        }
        true
    }

    /// Returns one result or saves the task waker
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Result<(), AuthenticationError>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
            state.detached = true;
            state.waker = None;
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

    /// Drops future interest and clears any stored result or waker
    pub(crate) fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
