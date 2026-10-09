#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable informational network path snapshots with static backend selection."]

use core::future::Future;
use framework_core::{Error, ErrorKind, PlatformErrorCode};

/// The status of one backend-observed local network path.
///
/// These values describe path state only. They do not establish Internet access, DNS success,
/// captive-portal clearance, endpoint reachability, or the outcome of a request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NetworkPathStatus {
    /// The backend has no valid status to report.
    ///
    /// This is not a claim that the network is down.
    Unknown,
    /// The backend reports a current path that can establish a connection attempt.
    ///
    /// This does not guarantee that a particular endpoint or request will succeed.
    Satisfied,
    /// The backend reports no currently usable path.
    Unsatisfied,
    /// The backend reports that a connection attempt may activate a path.
    ///
    /// This is distinct from [`Self::Satisfied`].
    Satisfiable,
}

/// One copyable snapshot of the backend-observed local network path status.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NetworkPathSnapshot {
    status: NetworkPathStatus,
}

impl NetworkPathSnapshot {
    /// Creates a snapshot with one framework-owned path status.
    pub const fn new(status: NetworkPathStatus) -> Self {
        Self { status }
    }

    /// Returns the observed status by value.
    pub const fn status(self) -> NetworkPathStatus {
        self.status
    }
}

/// A stable connectivity error that preserves an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NetworkPathError {
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl NetworkPathError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional signed backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for one observed network-path snapshot.
///
/// `current_path` must return a lazy future: native work begins no earlier than that future's
/// first poll. The first non-null native path callback completes the request, including a
/// callback whose invalid native status maps to [`NetworkPathStatus::Unknown`]; there is no
/// completion deadline. A backend that starts a continuous native monitor must stop it after
/// that first callback.
///
/// The associated future owns callback state, detachment/invalidation, and native cancellation.
/// Dropping the facade future drops this backend future; it must detach or invalidate callback
/// state before requesting native cancellation when supported. A generic
/// `framework_async::OperationFuture` only unregisters its waker and is not sufficient by itself
/// to own this native lifecycle. Callback state must remain valid until native callbacks can no
/// longer run, but it must not retain or access the dropped future. Late or duplicate callbacks
/// must not publish another result or wake a detached task. No executor or `Send` requirement is
/// imposed.
pub trait NetworkPathBackend {
    /// The concrete future for one current-path snapshot, selected without trait-object boxing.
    type CurrentPathFuture<'a>: Future<Output = Result<NetworkPathSnapshot, NetworkPathError>> + 'a
    where
        Self: 'a;

    /// Requests one snapshot without starting native work before the returned future is polled.
    fn current_path<'a>(&'a mut self) -> Self::CurrentPathFuture<'a>;
}

/// A thin facade over caller-owned, statically selected connectivity-backend state.
pub struct Connectivity<B> {
    backend: B,
}

impl<B: NetworkPathBackend> Connectivity<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Requests one advisory snapshot of the currently observed local network path.
    ///
    /// Backend work begins when this returned future is first polled. The snapshot may become
    /// stale immediately and does not prove that a particular operation can succeed; callers
    /// must attempt their operation and handle its actual result instead of gating it on this
    /// value. The backend may wait indefinitely for its first update.
    pub async fn current_path(&mut self) -> Result<NetworkPathSnapshot, NetworkPathError> {
        self.backend.current_path().await
    }

    /// Borrows the selected backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the selected backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::Wake;

    #[derive(Default)]
    struct Operation {
        backend_calls: usize,
        start_count: usize,
        callback_attached: bool,
        callback_count: usize,
        completed: bool,
        result: Option<Result<NetworkPathSnapshot, NetworkPathError>>,
        waker: Option<Waker>,
        cancel_requests: usize,
        detached_before_cancel: bool,
    }

    struct FakeBackend {
        operation: Rc<RefCell<Operation>>,
        result_on_start: Option<Result<NetworkPathSnapshot, NetworkPathError>>,
    }

    impl FakeBackend {
        fn pending() -> (Self, Rc<RefCell<Operation>>) {
            let operation = Rc::new(RefCell::new(Operation::default()));
            (
                Self {
                    operation: Rc::clone(&operation),
                    result_on_start: None,
                },
                operation,
            )
        }

        fn completes_with(
            result: Result<NetworkPathSnapshot, NetworkPathError>,
        ) -> (Self, Rc<RefCell<Operation>>) {
            let operation = Rc::new(RefCell::new(Operation::default()));
            (
                Self {
                    operation: Rc::clone(&operation),
                    result_on_start: Some(result),
                },
                operation,
            )
        }
    }

    struct FakeFuture {
        operation: Rc<RefCell<Operation>>,
        result_on_start: Option<Result<NetworkPathSnapshot, NetworkPathError>>,
        finished: bool,
    }

    impl Future for FakeFuture {
        type Output = Result<NetworkPathSnapshot, NetworkPathError>;

        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let operation = Rc::clone(&this.operation);
            let mut state = operation.borrow_mut();
            if state.start_count == 0 {
                state.start_count = 1;
                state.callback_attached = true;
                if let Some(result) = this.result_on_start.take() {
                    state.completed = true;
                    state.callback_attached = false;
                    state.result = Some(result);
                }
            }
            if let Some(result) = state.result.take() {
                this.finished = true;
                return Poll::Ready(result);
            }
            state.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }

    impl Drop for FakeFuture {
        fn drop(&mut self) {
            if self.finished {
                return;
            }
            let mut state = self.operation.borrow_mut();
            state.result = None;
            state.waker = None;
            if state.start_count != 0 && !state.completed {
                state.callback_attached = false;
                state.detached_before_cancel = !state.callback_attached && state.waker.is_none();
                state.cancel_requests += 1;
            }
        }
    }

    impl NetworkPathBackend for FakeBackend {
        type CurrentPathFuture<'a>
            = FakeFuture
        where
            Self: 'a;

        fn current_path<'a>(&'a mut self) -> Self::CurrentPathFuture<'a> {
            self.operation.borrow_mut().backend_calls += 1;
            FakeFuture {
                operation: Rc::clone(&self.operation),
                result_on_start: self.result_on_start.take(),
                finished: false,
            }
        }
    }

    fn deliver(
        operation: &Rc<RefCell<Operation>>,
        result: Result<NetworkPathSnapshot, NetworkPathError>,
    ) -> bool {
        let waker = {
            let mut state = operation.borrow_mut();
            if !state.callback_attached || state.completed {
                return false;
            }
            state.callback_count += 1;
            state.completed = true;
            state.callback_attached = false;
            state.result = Some(result);
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
        true
    }

    struct WakeCounter(Arc<AtomicUsize>);

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn counting_waker(counter: Arc<AtomicUsize>) -> Waker {
        Waker::from(Arc::new(WakeCounter(counter)))
    }

    #[test]
    fn all_native_statuses_have_distinct_portable_values() {
        for status in [
            NetworkPathStatus::Unknown,
            NetworkPathStatus::Satisfied,
            NetworkPathStatus::Unsatisfied,
            NetworkPathStatus::Satisfiable,
        ] {
            let snapshot = NetworkPathSnapshot::new(status);
            assert_eq!(snapshot.status(), status);
            let copied = snapshot;
            assert_eq!(snapshot, copied);
        }
        assert_ne!(
            NetworkPathSnapshot::new(NetworkPathStatus::Satisfied),
            NetworkPathSnapshot::new(NetworkPathStatus::Satisfiable)
        );
    }

    #[test]
    fn fake_backend_returns_one_snapshot_by_value() {
        let expected = NetworkPathSnapshot::new(NetworkPathStatus::Unsatisfied);
        let (backend, _) = FakeBackend::completes_with(Ok(expected));
        let mut connectivity = Connectivity::new(backend);
        let mut future = Box::pin(connectivity.current_path());
        let mut context = Context::from_waker(Waker::noop());

        assert_eq!(
            future.as_mut().poll(&mut context),
            Poll::Ready(Ok(expected))
        );
    }

    #[test]
    fn backend_work_starts_only_when_the_facade_future_is_polled() {
        let (backend, operation) = FakeBackend::pending();
        let mut connectivity = Connectivity::new(backend);
        let mut future = Box::pin(connectivity.current_path());
        assert_eq!(operation.borrow().backend_calls, 0);
        assert_eq!(operation.borrow().start_count, 0);

        let mut context = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut context).is_pending());
        assert_eq!(operation.borrow().backend_calls, 1);
        assert_eq!(operation.borrow().start_count, 1);
        assert!(operation.borrow().callback_attached);
    }

    #[test]
    fn dropping_pending_request_detaches_before_requesting_cancel() {
        let (backend, operation) = FakeBackend::pending();
        let mut connectivity = Connectivity::new(backend);
        let wakes = Arc::new(AtomicUsize::new(0));
        let waker = counting_waker(Arc::clone(&wakes));
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(connectivity.current_path());
        assert!(future.as_mut().poll(&mut context).is_pending());
        assert!(operation.borrow().callback_attached);
        drop(future);

        let state = operation.borrow();
        assert!(!state.callback_attached);
        assert!(state.waker.is_none());
        assert_eq!(state.cancel_requests, 1);
        assert!(state.detached_before_cancel);
        drop(state);
        assert!(!deliver(
            &operation,
            Ok(NetworkPathSnapshot::new(NetworkPathStatus::Satisfied))
        ));
        assert_eq!(wakes.load(Ordering::Relaxed), 0);
        assert_eq!(operation.borrow().result, None);
    }

    #[test]
    fn first_callback_completes_once_and_late_duplicates_are_ignored() {
        let (backend, operation) = FakeBackend::pending();
        let mut connectivity = Connectivity::new(backend);
        let wakes = Arc::new(AtomicUsize::new(0));
        let waker = counting_waker(Arc::clone(&wakes));
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(connectivity.current_path());
        assert!(future.as_mut().poll(&mut context).is_pending());

        let first = NetworkPathSnapshot::new(NetworkPathStatus::Unknown);
        assert!(deliver(&operation, Ok(first)));
        assert!(!deliver(
            &operation,
            Ok(NetworkPathSnapshot::new(NetworkPathStatus::Satisfied))
        ));
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
        assert_eq!(operation.borrow().callback_count, 1);
        assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(Ok(first)));
    }

    #[test]
    fn backend_error_preserves_kind_and_optional_native_code() {
        let code = PlatformErrorCode::new(-71).unwrap();
        let error =
            NetworkPathError::Backend(Error::new(ErrorKind::Unavailable).with_platform_code(code));
        assert_eq!(error.kind(), ErrorKind::Unavailable);
        assert_eq!(error.platform_code(), Some(code));
        assert_eq!(
            NetworkPathError::Backend(Error::new(ErrorKind::Cancelled)).platform_code(),
            None
        );
    }
}
