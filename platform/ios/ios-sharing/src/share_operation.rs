use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_sharing::{ShareError, ShareOutcome, ShareRequest};

type ShareCallback = Box<dyn FnOnce(Result<ShareOutcome, ShareError>)>;

/// A synchronous share-start failure that returns the callback to its caller.
pub struct ShareStartError<F> {
    /// The error that prevented request acceptance.
    pub error: ShareError,
    /// The callback that remains owned by the caller after rejection.
    pub callback: F,
}

impl<F> core::fmt::Debug for ShareStartError<F> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ShareStartError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

pub(crate) fn rejected_start<F>(error: ShareError, callback: F) -> ShareStartError<F> {
    ShareStartError { error, callback }
}

pub(crate) fn accept_preflight<T, F>(
    preflight: Result<T, ShareError>,
    callback: F,
) -> Result<(T, F), ShareStartError<F>> {
    match preflight {
        Ok(value) => Ok((value, callback)),
        Err(error) => Err(rejected_start(error, callback)),
    }
}

struct CompletionState {
    active: bool,
    completed: bool,
    result: Option<Result<ShareOutcome, ShareError>>,
    waker: Option<Waker>,
    callback: Option<ShareCallback>,
}

#[derive(Clone)]
pub(crate) struct ShareCompletion {
    state: Rc<RefCell<CompletionState>>,
}

impl ShareCompletion {
    pub(crate) fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(CompletionState {
                active: true,
                completed: false,
                result: None,
                waker: None,
                callback: None,
            })),
        }
    }

    pub(crate) fn with_callback(
        callback: impl FnOnce(Result<ShareOutcome, ShareError>) + 'static,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(CompletionState {
                active: true,
                completed: false,
                result: None,
                waker: None,
                callback: Some(Box::new(callback)),
            })),
        }
    }

    pub(crate) fn complete(&self, result: Result<ShareOutcome, ShareError>) -> bool {
        self.complete_with(result, || {})
    }

    pub(crate) fn complete_with(
        &self,
        result: Result<ShareOutcome, ShareError>,
        before_notify: impl FnOnce(),
    ) -> bool {
        let (waker, callback) = {
            let mut state = self.state.borrow_mut();
            if !state.active || state.completed {
                return false;
            }
            state.completed = true;
            let callback = state.callback.take();
            if callback.is_some() {
                state.active = false;
                state.result = None;
                state.waker = None;
                (None, callback)
            } else {
                state.result = Some(result);
                (state.waker.take(), None)
            }
        };
        before_notify();
        if let Some(callback) = callback {
            callback(result);
        }
        if let Some(waker) = waker {
            waker.wake();
        }
        true
    }

    pub(crate) fn is_active(&self) -> bool {
        let state = self.state.borrow();
        state.active && !state.completed
    }

    fn poll_result(&self, context: &Context<'_>) -> Poll<Result<ShareOutcome, ShareError>> {
        let mut state = self.state.borrow_mut();
        if let Some(result) = state.result.take() {
            state.active = false;
            state.waker = None;
            return Poll::Ready(result);
        }
        if state.active
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    pub(crate) fn detach(&self) {
        let mut state = self.state.borrow_mut();
        state.active = false;
        state.result = None;
        state.waker = None;
        state.callback = None;
    }
}

pub(crate) trait ShareAccess {
    fn present(
        &mut self,
        request: ShareRequest,
        completion: ShareCompletion,
    ) -> Result<(), ShareError>;

    fn detach(&mut self);
}

pub(crate) struct ShareOperation<'a, A: ShareAccess> {
    access: &'a mut A,
    request: Option<ShareRequest>,
    completion: ShareCompletion,
    started: bool,
    finished: bool,
    _not_send: PhantomData<Rc<()>>,
}

impl<'a, A: ShareAccess> ShareOperation<'a, A> {
    pub(crate) fn new(access: &'a mut A, request: ShareRequest) -> Self {
        Self {
            access,
            request: Some(request),
            completion: ShareCompletion::new(),
            started: false,
            finished: false,
            _not_send: PhantomData,
        }
    }
}

impl<A: ShareAccess> Future for ShareOperation<'_, A> {
    type Output = Result<ShareOutcome, ShareError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        if !this.started {
            this.started = true;
            let request = this.request.take().expect("share request starts once");
            if let Err(error) = this.access.present(request, this.completion.clone()) {
                this.completion.complete(Err(error));
            }
        }
        match this.completion.poll_result(context) {
            Poll::Ready(result) => {
                this.finished = true;
                this.access.detach();
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<A: ShareAccess> Drop for ShareOperation<'_, A> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.completion.detach();
            self.access.detach();
        }
    }
}

pub(crate) fn map_activity_result(
    completed: bool,
    has_error: bool,
    native_code: Option<i32>,
) -> Result<ShareOutcome, ShareError> {
    if has_error {
        let mut error = Error::new(ErrorKind::Platform);
        if let Some(code) = native_code.and_then(PlatformErrorCode::new) {
            error = error.with_platform_code(code);
        }
        Err(ShareError::Backend(error))
    } else if completed {
        Ok(ShareOutcome::Completed)
    } else {
        Ok(ShareOutcome::Dismissed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use alloc::string::ToString;
    use core::cell::Cell;
    use core::cell::RefCell;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    #[derive(Default)]
    struct FakeAccessState {
        present_count: u32,
        detach_count: u32,
        completion: Option<ShareCompletion>,
        immediate_result: Option<Result<ShareOutcome, ShareError>>,
        start_error: Option<ShareError>,
        request: Option<ShareRequest>,
    }

    #[derive(Clone, Default)]
    struct FakeAccess {
        state: Rc<RefCell<FakeAccessState>>,
    }

    impl ShareAccess for FakeAccess {
        fn present(
            &mut self,
            request: ShareRequest,
            completion: ShareCompletion,
        ) -> Result<(), ShareError> {
            let (immediate_result, start_error) = {
                let mut state = self.state.borrow_mut();
                state.present_count += 1;
                state.request = Some(request);
                state.completion = Some(completion.clone());
                (state.immediate_result, state.start_error)
            };
            if let Some(result) = immediate_result {
                completion.complete(result);
            }
            if let Some(error) = start_error {
                return Err(error);
            }
            Ok(())
        }

        fn detach(&mut self) {
            let mut state = self.state.borrow_mut();
            state.detach_count += 1;
            state.completion = None;
        }
    }

    fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    fn request() -> ShareRequest {
        ShareRequest::new(framework_sharing::ShareItem::text("hello".to_string()))
    }

    #[test]
    fn activity_result_maps_completion_dismissal_and_native_error() {
        assert_eq!(
            map_activity_result(true, false, None),
            Ok(ShareOutcome::Completed)
        );
        assert_eq!(
            map_activity_result(false, false, None),
            Ok(ShareOutcome::Dismissed)
        );
        let error = map_activity_result(false, true, Some(-41)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -41);
        assert_eq!(
            map_activity_result(false, true, Some(0))
                .unwrap_err()
                .platform_code(),
            None
        );
    }

    #[test]
    fn start_occurs_on_first_poll_and_terminal_completion_is_exactly_once() {
        let mut access = FakeAccess::default();
        let state = access.state.clone();
        let mut future = pin!(ShareOperation::new(&mut access, request()));
        assert!(poll(future.as_mut()).is_pending());
        assert_eq!(state.borrow().present_count, 1);
        assert_eq!(state.borrow().request.as_ref().unwrap().items().len(), 1);
        let completion = state.borrow().completion.as_ref().unwrap().clone();
        assert!(completion.complete(Ok(ShareOutcome::Completed)));
        assert!(!completion.complete(Ok(ShareOutcome::Dismissed)));
        assert_eq!(
            poll(future.as_mut()),
            Poll::Ready(Ok(ShareOutcome::Completed))
        );
        assert_eq!(state.borrow().detach_count, 1);
    }

    #[test]
    fn dropping_before_first_poll_starts_no_native_work() {
        let mut access = FakeAccess::default();
        let state = access.state.clone();
        drop(ShareOperation::new(&mut access, request()));
        assert_eq!(state.borrow().present_count, 0);
        assert_eq!(state.borrow().detach_count, 0);
    }

    #[test]
    fn dropping_after_start_detaches_callback_and_suppresses_late_result() {
        let mut access = FakeAccess::default();
        let state = access.state.clone();
        let completion;
        {
            let mut future = pin!(ShareOperation::new(&mut access, request()));
            assert!(poll(future.as_mut()).is_pending());
            completion = state.borrow().completion.as_ref().unwrap().clone();
        }
        assert_eq!(state.borrow().detach_count, 1);
        assert!(!completion.complete(Ok(ShareOutcome::Completed)));
    }

    #[test]
    fn start_error_completes_without_leaving_the_future_pending() {
        let error = ShareError::Backend(Error::new(ErrorKind::Unavailable));
        let mut access = FakeAccess::default();
        access.state.borrow_mut().start_error = Some(error);
        let state = access.state.clone();
        let mut future = pin!(ShareOperation::new(&mut access, request()));
        assert_eq!(poll(future.as_mut()), Poll::Ready(Err(error)));
        assert_eq!(state.borrow().present_count, 1);
        assert_eq!(state.borrow().detach_count, 1);
    }

    #[test]
    fn synchronous_preflight_rejection_returns_the_unconsumed_callback() {
        let error = ShareError::Backend(Error::new(ErrorKind::Unavailable));
        let callback_called = Rc::new(Cell::new(false));
        let callback_state = callback_called.clone();
        let callback = move |_: Result<ShareOutcome, ShareError>| callback_state.set(true);
        let rejected = accept_preflight::<(), _>(Err(error), callback);
        let rejection = match rejected {
            Err(rejection) => rejection,
            Ok(_) => panic!("failed preflight was accepted"),
        };

        assert_eq!(rejection.error, error);
        assert!(!callback_called.get());
        (rejection.callback)(Ok(ShareOutcome::Completed));
        assert!(callback_called.get());
    }

    #[test]
    fn callback_completion_detaches_before_notifying_exactly_once() {
        let callback_count = Rc::new(Cell::new(0));
        let callback_detached = Rc::new(Cell::new(false));
        let callback_count_state = callback_count.clone();
        let callback_detached_state = callback_detached.clone();
        let completion = ShareCompletion::with_callback(move |_| {
            assert!(callback_detached_state.get());
            callback_count_state.set(callback_count_state.get() + 1);
        });

        assert!(completion.complete_with(Ok(ShareOutcome::Completed), || {
            callback_detached.set(true);
        }));
        assert!(!completion.is_active());
        assert!(!completion.complete(Ok(ShareOutcome::Dismissed)));
        assert_eq!(callback_count.get(), 1);
    }

    #[test]
    fn detached_callback_completion_suppresses_late_result() {
        let callback_called = Rc::new(Cell::new(false));
        let callback_state = callback_called.clone();
        let completion = ShareCompletion::with_callback(move |_| callback_state.set(true));

        completion.detach();
        assert!(!completion.complete(Ok(ShareOutcome::Completed)));
        assert!(!callback_called.get());
    }
}
