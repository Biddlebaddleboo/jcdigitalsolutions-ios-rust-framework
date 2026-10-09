use core::task::{Context, Poll, Waker};
use std::sync::{Mutex, MutexGuard};

use crate::{FileProviderQueryError, RegisteredDomainPresence};

/// Per-request state shared only with its native callback
pub(crate) struct Completion {
    state: Mutex<State>,
}

struct State {
    completed: bool,
    detached: bool,
    result: Option<Result<RegisteredDomainPresence, FileProviderQueryError>>,
    waker: Option<Waker>,
}

impl Completion {
    /// Creates a pending request state
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

    /// Publishes one callback result and wakes the current caller task
    pub(crate) fn complete(
        &self,
        result: Result<RegisteredDomainPresence, FileProviderQueryError>,
    ) {
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

    /// Reads a completed result or stores the current task waker
    pub(crate) fn poll(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Result<RegisteredDomainPresence, FileProviderQueryError>> {
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

    /// Drops Rust interest while leaving callback state valid until the native callback ends
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
