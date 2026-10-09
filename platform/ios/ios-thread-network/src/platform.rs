use core::future::Future;
use core::marker::PhantomPinned;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::Bool;
use objc2_thread_network::THClient;

use self::completion::Completion;

/// A one-shot, non-`Send` future for preferred Thread network availability.
///
/// The native operation starts when the request function runs. The future retains its
/// `THClient` on the originating thread until it is dropped. Dropping the future detaches the Rust
/// result and waker; it does not guarantee that ThreadNetwork cancels its asynchronous operation.
pub struct PreferredThreadNetworkAvailabilityFuture {
    completion: Arc<Completion>,
    _client: Retained<THClient>,
    finished: bool,
    _pin: PhantomPinned,
}

impl Future for PreferredThreadNetworkAvailabilityFuture {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: Poll updates only `finished` and the Arc-backed completion state. It does not
        // move the retained `THClient` or any field that could be structurally pinned.
        let this = unsafe { self.get_unchecked_mut() };
        if this.finished {
            return Poll::Pending;
        }
        let result = this.completion.poll(context);
        if result.is_ready() {
            this.finished = true;
        }
        result
    }
}

impl Drop for PreferredThreadNetworkAvailabilityFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

pub(crate) fn start() -> PreferredThreadNetworkAvailabilityFuture {
    let completion = Arc::new(Completion::new());
    let callback_completion = Arc::clone(&completion);
    let handler = RcBlock::new(move |available: Bool| {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            callback_completion.complete(available.as_bool());
        }));
    });

    let client = autoreleasepool(|_| {
        // SAFETY: `THClient` is a public Objective-C class and the retained result remains in the
        // future, so it is created, used, and released on one Rust thread.
        let client = unsafe { THClient::new() };
        // SAFETY: iOS 16.4 is the declared availability floor. The async Objective-C method
        // retains its owned heap block for completion. The block captures only `Arc<Completion>`,
        // which is synchronized and contains no Objective-C handle or borrowed Rust data; Apple
        // does not document a callback queue, so it is safe for the block to run on any queue.
        unsafe { client.isPreferredNetworkAvailableWithCompletion(&handler) };
        client
    });

    PreferredThreadNetworkAvailabilityFuture {
        completion,
        _client: client,
        finished: false,
        _pin: PhantomPinned,
    }
}

mod completion {
    use core::task::{Context, Poll, Waker};
    use std::sync::{Mutex, MutexGuard};

    /// One request's owned completion state shared with its native callback.
    pub(super) struct Completion {
        state: Mutex<State>,
    }

    struct State {
        completed: bool,
        detached: bool,
        result: Option<bool>,
        waker: Option<Waker>,
    }

    impl Completion {
        /// Creates a pending query state.
        pub(super) fn new() -> Self {
            Self {
                state: Mutex::new(State {
                    completed: false,
                    detached: false,
                    result: None,
                    waker: None,
                }),
            }
        }

        /// Publishes the returned scalar and wakes the current task.
        pub(super) fn complete(&self, result: bool) {
            let waker = {
                let mut state = self.lock();
                if state.completed {
                    return;
                }
                state.completed = true;
                if state.detached {
                    return;
                }
                state.result = Some(result);
                state.waker.take()
            };
            if let Some(waker) = waker {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
            }
        }

        /// Reads a result or stores the current task waker.
        pub(super) fn poll(&self, context: &mut Context<'_>) -> Poll<bool> {
            let mut state = self.lock();
            if let Some(result) = state.result.take() {
                return Poll::Ready(result);
            }
            if !state.detached
                && state
                    .waker
                    .as_ref()
                    .is_none_or(|waker| !waker.will_wake(context.waker()))
            {
                state.waker = Some(context.waker().clone());
            }
            Poll::Pending
        }

        /// Drops Rust interest while leaving the callback state valid until native completion.
        pub(super) fn detach(&self) {
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
}
