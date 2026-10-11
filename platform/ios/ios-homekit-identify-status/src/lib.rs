#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Host-supplied HomeKit accessory identify support and operation for iOS"]

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
use core::task::{Context, Poll, Waker};
#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
use std::sync::{Mutex, MutexGuard};

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
/// An error while requesting a HomeKit accessory identify action
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitIdentifyError {
    /// The current iOS version does not provide the requested HomeKit API
    NativeApiUnavailable,
    /// HomeKit rejected the identify request with its original NSError domain and code
    NativeFailure {
        /// The original Foundation error domain
        domain: String,
        /// The original Foundation error code
        code: i64,
    },
    /// A Rust panic occurred while copying the native completion result
    CallbackPanicked,
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit accessory identify-support value
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitIdentifyStatusError {
    /// The iOS version does not provide `HMAccessory.supportsIdentify`
    NativeApiUnavailable,
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessory;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// Read whether a host-supplied accessory supports the HomeKit identify action
///
/// This reads only `HMAccessory.supportsIdentify` from a live accessory that the host already
/// obtained through its HomeKit lifecycle. `Ok(false)` means Apple documents that calls to
/// `identifyWithCompletionHandler:` return an error; `Ok(true)` does not guarantee that a later
/// identify call succeeds. This function never invokes identify, creates `HMHomeManager`, requests
/// permission, enumerates accessories, or reports HomeKit readiness, authorization, or reachability
///
/// The getter is available on iOS 11.3 and later. Older iOS versions return
/// [`HomeKitIdentifyStatusError::NativeApiUnavailable`]. The host owns the HomeKit capability,
/// `NSHomeKitUsageDescription`, prior HomeKit authorization, and the lifetime/source of the supplied
/// accessory
pub fn supports_identify(accessory: &HMAccessory) -> Result<bool, HomeKitIdentifyStatusError> {
    if !objc2::available!(ios = 11.3, ..) {
        return Err(HomeKitIdentifyStatusError::NativeApiUnavailable);
    }

    // SAFETY: the input is a borrowed live HMAccessory; Apple marks the class NS_SWIFT_SENDABLE,
    // and this scalar readonly Objective-C property omits nonatomic, so its access is atomic
    Ok(unsafe { accessory.supportsIdentify() })
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// A lazy HomeKit accessory identify request
///
/// The future retains the host-supplied accessory and starts the public HomeKit request on first
/// poll. Dropping this future detaches its result and waker but does not cancel the HomeKit
/// operation; callback-owned state and the accessory remain alive until HomeKit releases its
/// completion block. The host continues to own the HomeKit manager/lifecycle, capability,
/// entitlement, `NSHomeKitUsageDescription`, prior authorization, and accessory source
#[must_use = "futures do nothing unless polled"]
pub struct HomeKitIdentifyFuture {
    accessory: objc2::rc::Retained<HMAccessory>,
    completion: std::sync::Arc<CompletionCell<(), HomeKitIdentifyError>>,
    started: bool,
    finished: bool,
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
impl HomeKitIdentifyFuture {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        if !objc2::available!(ios = 8.0, ..) {
            self.completion
                .complete(Err(HomeKitIdentifyError::NativeApiUnavailable));
            return;
        }

        use block2::RcBlock;
        use objc2::Message;
        use objc2::rc::autoreleasepool;
        use objc2_foundation::NSError;
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let completion = std::sync::Arc::clone(&self.completion);
        let callback_accessory = self.accessory.clone();
        let handler = RcBlock::new(move |error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: HomeKit borrows NSError only for this completion invocation.
                    let native_error = unsafe { error.as_ref() }.map(|error| NativeFailure {
                        domain: error.domain().to_string(),
                        code: error.code() as i64,
                    });
                    result_from_native(native_error)
                })
            }))
            .unwrap_or(Err(HomeKitIdentifyError::CallbackPanicked));
            completion.complete(result);
            // Keep the borrowed accessory alive through any callback and future cancellation.
            let _ = &callback_accessory;
        });

        // SAFETY: `handler` is an owned escaping block for Apple's asynchronous completion API.
        // The block captures only thread-safe owned state and retains the supplied accessory.
        // HomeKit documents that it invokes this completion once the request is processed.
        unsafe { self.accessory.identifyWithCompletionHandler(&handler) };
    }
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
impl core::future::Future for HomeKitIdentifyFuture {
    type Output = Result<(), HomeKitIdentifyError>;

    fn poll(self: core::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.start();
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
impl Drop for HomeKitIdentifyFuture {
    fn drop(&mut self) {
        if !self.finished {
            self.completion.detach();
        }
    }
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// Request that a host-supplied live HomeKit accessory identify itself
///
/// The request begins when the returned future is first polled. A nil `NSError` yields `Ok(())`;
/// a non-nil error preserves its native domain and code. Success means only that HomeKit reports
/// the request processed; it does not prove the accessory was reachable or visibly identified
/// itself. This function never creates `HMHomeManager`, requests permission, enumerates
/// accessories, or manages the host's HomeKit lifecycle
///
/// `identifyWithCompletionHandler:` is inherited from `HMAccessory`, available on iOS 8.0 and
/// later. The separate `supports_identify` snapshot remains available only from iOS 11.3. The host
/// owns HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior
/// authorization, and the source/lifetime of the supplied accessory
pub fn identify(accessory: &HMAccessory) -> HomeKitIdentifyFuture {
    HomeKitIdentifyFuture {
        accessory: objc2::Message::retain(accessory),
        completion: std::sync::Arc::new(CompletionCell::new()),
        started: false,
        finished: false,
    }
}

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeFailure {
    domain: String,
    code: i64,
}

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
fn result_from_native(error: Option<NativeFailure>) -> Result<(), HomeKitIdentifyError> {
    error.map_or(Ok(()), |error| {
        Err(HomeKitIdentifyError::NativeFailure {
            domain: error.domain,
            code: error.code,
        })
    })
}

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
struct CompletionCell<T, E> {
    state: Mutex<CompletionState<T, E>>,
}

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
struct CompletionState<T, E> {
    completed: bool,
    detached: bool,
    result: Option<Result<T, E>>,
    waker: Option<Waker>,
}

#[cfg(any(all(target_os = "ios", not(target_abi = "macabi")), test))]
impl<T, E> CompletionCell<T, E> {
    const fn new() -> Self {
        Self {
            state: Mutex::new(CompletionState {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    fn complete(&self, result: Result<T, E>) -> bool {
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

    fn poll(&self, context: &mut Context<'_>) -> Poll<Result<T, E>> {
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

    fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, CompletionState<T, E>> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    struct CountWake(AtomicUsize);

    impl Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn nil_native_error_completes_successfully_once() {
        assert_eq!(result_from_native(None), Ok(()));
        let completion = CompletionCell::new();
        assert!(completion.complete(Ok::<_, HomeKitIdentifyError>(())));
        assert!(!completion.complete(Err(HomeKitIdentifyError::CallbackPanicked)));
        let waker = Waker::from(Arc::new(NoopWake));
        assert_eq!(
            completion.poll(&mut Context::from_waker(&waker)),
            Poll::Ready(Ok(()))
        );
    }

    #[test]
    fn native_completion_wakes_the_registered_task_once() {
        let completion = CompletionCell::new();
        let wake_count = Arc::new(CountWake(AtomicUsize::new(0)));
        let waker = Waker::from(Arc::clone(&wake_count));
        assert_eq!(
            completion.poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
        assert!(completion.complete(Ok::<_, HomeKitIdentifyError>(())));
        assert!(!completion.complete(Ok(())));
        assert_eq!(wake_count.0.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn native_error_preserves_domain_and_code() {
        let native = NativeFailure {
            domain: "HMErrorDomain".to_owned(),
            code: -70502,
        };
        assert_eq!(
            result_from_native(Some(native)),
            Err(HomeKitIdentifyError::NativeFailure {
                domain: "HMErrorDomain".to_owned(),
                code: -70502,
            })
        );
    }

    #[test]
    fn dropped_future_detaches_without_invalidating_callback_state() {
        let future_state = Arc::new(CompletionCell::<(), HomeKitIdentifyError>::new());
        let callback_state = Arc::clone(&future_state);
        future_state.detach();
        drop(future_state);

        assert!(callback_state.complete(result_from_native(None)));
        let waker = Waker::from(Arc::new(NoopWake));
        assert_eq!(
            callback_state.poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
    }

    #[test]
    fn callback_state_outlives_future_until_completion() {
        let future_state = Arc::new(CompletionCell::<(), HomeKitIdentifyError>::new());
        let callback_state = Arc::clone(&future_state);
        assert_eq!(Arc::strong_count(&callback_state), 2);
        drop(future_state);
        assert_eq!(Arc::strong_count(&callback_state), 1);
        assert!(callback_state.complete(result_from_native(None)));
        let waker = Waker::from(Arc::new(NoopWake));
        assert_eq!(
            callback_state.poll(&mut Context::from_waker(&waker)),
            Poll::Ready(Ok(()))
        );
    }
}
