use core::task::{Context, Poll, Waker};
use std::sync::{Mutex, MutexGuard};

use framework_vpn::PersonalVpnQueryError;

/// One request's owned completion state shared with its native callback.
pub(crate) struct Completion<T> {
    state: Mutex<State<T>>,
}

struct State<T> {
    completed: bool,
    detached: bool,
    result: Option<Result<T, PersonalVpnQueryError>>,
    waker: Option<Waker>,
}

impl<T: Send + 'static> Completion<T> {
    /// Creates a pending request state.
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(State {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    /// Publishes one callback result and wakes the current caller task.
    pub(crate) fn complete(&self, result: Result<T, PersonalVpnQueryError>) {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return;
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
    }

    /// Reads a completed result or stores the current task waker.
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Result<T, PersonalVpnQueryError>> {
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
}

impl<T> Completion<T> {
    /// Drops Rust interest while leaving callback state valid until the native callback ends.
    pub(crate) fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State<T>> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
