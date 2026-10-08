//! Portable outgoing-share values and a statically selected backend contract.

use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// One owned item supported by the V1 outgoing-share contract.
///
/// URL text is retained as supplied by the caller. This portable layer does not validate, parse,
/// or normalize it; a backend may reject or transform it while creating a native share item.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ShareItem {
    /// UTF-8 text supplied by the caller.
    Text(String),
    /// URL text supplied by the caller, without portable validation or normalization.
    Url(String),
}

impl ShareItem {
    /// Takes ownership of a UTF-8 text item.
    pub const fn text(value: String) -> Self {
        Self::Text(value)
    }

    /// Takes ownership of URL text without validating or normalizing it.
    pub const fn url(value: String) -> Self {
        Self::Url(value)
    }

    /// Borrows the payload as UTF-8 text, regardless of its item kind.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text(value) | Self::Url(value) => value,
        }
    }
}

/// An owned outgoing-share request containing one or more text or URL-text items.
///
/// The item order is retained in this value and passed to the backend in that order. A native
/// platform may choose a different presentation order or representation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ShareRequest {
    items: Vec<ShareItem>,
}

impl ShareRequest {
    /// Creates a request with one item, taking ownership of its payload.
    pub fn new(item: ShareItem) -> Self {
        Self {
            items: alloc::vec![item],
        }
    }

    /// Appends an item while preserving the request's existing item order.
    pub fn push(&mut self, item: ShareItem) {
        self.items.push(item);
    }

    /// Returns the request's non-empty items in caller-supplied order.
    pub fn items(&self) -> &[ShareItem] {
        &self.items
    }

    /// Transfers the owned items to the caller in caller-supplied order.
    pub fn into_items(self) -> Vec<ShareItem> {
        self.items
    }
}

/// The terminal state reported by a completed system-share operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShareOutcome {
    /// The system-share operation reported completion; this does not prove recipient delivery.
    Completed,
    /// The user dismissed or cancelled the system-share operation.
    Dismissed,
}

/// A portable share error that preserves an optional backend-native error code.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShareError {
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl ShareError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for one system-share operation.
///
/// The operation starts when its returned future is first polled. Dropping the future before its
/// first poll starts no backend work. Dropping it after native presentation begins suppresses the
/// Rust result, but cannot guarantee dismissal of system share UI already presented. The backend
/// must detach callback state safely, ignore any late result, and arrange exactly one terminal
/// completion while the future remains attached. No executor or `Send` requirement is imposed.
pub trait ShareBackend {
    /// Reports whether sharing is usable in the current backend context without presenting UI.
    fn availability(&self) -> Availability;

    /// The future type for a system-share operation.
    type ShareFuture<'a>: Future<Output = Result<ShareOutcome, ShareError>> + 'a
    where
        Self: 'a;

    /// Starts sharing the owned request when the returned future is first polled.
    ///
    /// The backend receives the URL text exactly as stored in the request, but may reject,
    /// canonicalize, or otherwise transform it when constructing a native item. The backend owns
    /// the request and is responsible for any copies required to keep native items alive after
    /// Rust drops the future.
    fn share<'a>(&'a mut self, request: ShareRequest) -> Self::ShareFuture<'a>;
}

/// A thin facade over caller-owned, statically selected share-backend state.
pub struct ShareClient<B> {
    backend: B,
}

impl<B: ShareBackend> ShareClient<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without presenting UI or performing hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Presents the supplied request through the selected backend and awaits its terminal result.
    pub async fn share(&mut self, request: ShareRequest) -> Result<ShareOutcome, ShareError> {
        self.backend.share(request).await
    }

    /// Borrows the backend for platform-specific controls or native escape hatches.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for platform-specific controls or native escape hatches.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use core::cell::RefCell;
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    struct FakeState {
        result: Result<ShareOutcome, ShareError>,
        request: Option<ShareRequest>,
        start_count: u32,
        completion_count: u32,
        detach_count: u32,
        callback_attached: bool,
        system_ui_visible: bool,
    }

    struct FakeBackend {
        state: Rc<RefCell<FakeState>>,
    }

    impl FakeBackend {
        fn new(result: Result<ShareOutcome, ShareError>) -> Self {
            Self {
                state: Rc::new(RefCell::new(FakeState {
                    result,
                    request: None,
                    start_count: 0,
                    completion_count: 0,
                    detach_count: 0,
                    callback_attached: false,
                    system_ui_visible: false,
                })),
            }
        }
    }

    struct FakeShareFuture {
        state: Rc<RefCell<FakeState>>,
        request: Option<ShareRequest>,
        started: bool,
        completed: bool,
    }

    impl Future for FakeShareFuture {
        type Output = Result<ShareOutcome, ShareError>;

        fn poll(self: core::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                this.started = true;
                let mut state = this.state.borrow_mut();
                state.start_count += 1;
                state.request = this.request.take();
                state.callback_attached = true;
                state.system_ui_visible = true;
                context.waker().wake_by_ref();
                return Poll::Pending;
            }
            if this.completed {
                return Poll::Pending;
            }
            this.completed = true;
            let mut state = this.state.borrow_mut();
            state.completion_count += 1;
            state.callback_attached = false;
            Poll::Ready(state.result)
        }
    }

    impl Drop for FakeShareFuture {
        fn drop(&mut self) {
            if self.started && !self.completed {
                let mut state = self.state.borrow_mut();
                state.detach_count += 1;
                state.callback_attached = false;
            }
        }
    }

    impl ShareBackend for FakeBackend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type ShareFuture<'a>
            = FakeShareFuture
        where
            Self: 'a;

        fn share<'a>(&'a mut self, request: ShareRequest) -> Self::ShareFuture<'a> {
            FakeShareFuture {
                state: self.state.clone(),
                request: Some(request),
                started: false,
                completed: false,
            }
        }
    }

    fn request() -> ShareRequest {
        let mut request = ShareRequest::new(ShareItem::text(String::from("hello")));
        request.push(ShareItem::url(String::from("not validated as a URL")));
        request
    }

    fn run_pending_once<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(future.as_mut().poll(&mut context).is_pending());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("fake backend did not complete on its second poll"),
        }
    }

    #[test]
    fn request_owns_ordered_text_and_unvalidated_url_text() {
        let request = request();
        assert_eq!(request.items().len(), 2);
        assert_eq!(request.items()[0], ShareItem::Text(String::from("hello")));
        assert_eq!(
            request.items()[1],
            ShareItem::Url(String::from("not validated as a URL"))
        );
        assert_eq!(request.items()[1].as_str(), "not validated as a URL");
        assert_eq!(request.into_items().len(), 2);
    }

    #[test]
    fn backend_reports_completed_and_dismissed_as_portable_results() {
        for outcome in [ShareOutcome::Completed, ShareOutcome::Dismissed] {
            let backend = FakeBackend::new(Ok(outcome));
            let state = backend.state.clone();
            let mut client = ShareClient::new(backend);
            assert_eq!(client.availability(), Availability::Available);
            assert_eq!(run_pending_once(client.share(request())), Ok(outcome));
            let state = state.borrow();
            assert_eq!(state.start_count, 1);
            assert_eq!(state.completion_count, 1);
            assert!(!state.callback_attached);
            assert_eq!(state.request.as_ref().unwrap().items(), request().items());
        }
    }

    #[test]
    fn backend_errors_preserve_error_kind_and_native_code() {
        let code = PlatformErrorCode::new(-41).unwrap();
        let error =
            ShareError::Backend(Error::new(ErrorKind::PermissionDenied).with_platform_code(code));
        let backend = FakeBackend::new(Err(error));
        let mut client = ShareClient::new(backend);
        assert_eq!(run_pending_once(client.share(request())), Err(error));
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(error.platform_code(), Some(code));
    }

    #[test]
    fn dropping_before_first_poll_starts_no_presentation() {
        let backend = FakeBackend::new(Ok(ShareOutcome::Completed));
        let state = backend.state.clone();
        let mut client = ShareClient::new(backend);
        drop(client.share(request()));
        let state = state.borrow();
        assert_eq!(state.start_count, 0);
        assert_eq!(state.completion_count, 0);
        assert_eq!(state.detach_count, 0);
        assert!(!state.system_ui_visible);
    }

    #[test]
    fn dropping_after_presentation_detaches_result_but_does_not_hide_system_ui() {
        let backend = FakeBackend::new(Ok(ShareOutcome::Completed));
        let state = backend.state.clone();
        let mut client = ShareClient::new(backend);
        {
            let mut future = pin!(client.share(request()));
            let mut context = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut context).is_pending());
            let state = state.borrow();
            assert_eq!(state.start_count, 1);
            assert!(state.callback_attached);
            assert!(state.system_ui_visible);
        }
        let state = state.borrow();
        assert_eq!(state.completion_count, 0);
        assert_eq!(state.detach_count, 1);
        assert!(!state.callback_attached);
        assert!(state.system_ui_visible);
    }
}
