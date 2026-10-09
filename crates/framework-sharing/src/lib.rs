#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable clipboard and outgoing-share contracts with statically selected backends."]

extern crate alloc;

pub mod share;
pub use share::{ShareBackend, ShareClient, ShareError, ShareItem, ShareOutcome, ShareRequest};

use alloc::string::String;
use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A stable clipboard error that preserves an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ClipboardError {
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl ClipboardError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for plain-text clipboard operations.
///
/// Each operation starts when its returned future is first polled. Dropping a future before its
/// first poll starts no backend work. Dropping a started future ends Rust interest in its result;
/// the backend must request native cancellation when supported. If the native operation cannot be
/// cancelled or races with cancellation, the backend must detach its callback state safely,
/// discard any late result, and release that state exactly once. A native write or clear may still
/// change shared clipboard state after its Rust future is dropped. No executor or `Send`
/// requirement is imposed.
///
/// The clipboard is shared state. Other applications or native code may change it at any time, so
/// a successful read does not reserve its value and a successful write or clear does not promise
/// that the value remains unchanged.
pub trait ClipboardBackend {
    /// Reports the backend's current view of plain-text clipboard availability.
    ///
    /// This is a backend judgment, not a guarantee that a later read, write, or clear will
    /// succeed. The portable contract does not assign permission or privacy semantics to this
    /// value.
    fn availability(&self) -> Availability;

    /// The future type for a plain-text read.
    type ReadFuture<'a>: Future<Output = Result<Option<String>, ClipboardError>> + 'a
    where
        Self: 'a;

    /// Reads the current plain-text value as an owned UTF-8 string.
    ///
    /// `Ok(None)` means that no readable plain-text representation exists. A backend failure is
    /// returned as `Err`; it must not be mapped to `None`. The returned string belongs to Rust and
    /// must not borrow native clipboard storage.
    fn read<'a>(&'a mut self) -> Self::ReadFuture<'a>;

    /// The future type for a plain-text write.
    type WriteFuture<'a>: Future<Output = Result<(), ClipboardError>> + 'a
    where
        Self: 'a;

    /// Writes UTF-8 text as the clipboard's plain-text value.
    ///
    /// The input is borrowed until the future completes or is dropped. A backend that needs the
    /// text after that borrow ends must copy it into backend-owned storage. Other representations
    /// on the shared clipboard may be preserved, replaced, or changed by the platform backend.
    fn write<'a>(&'a mut self, text: &'a str) -> Self::WriteFuture<'a>;

    /// The future type for a plain-text clear.
    type ClearFuture<'a>: Future<Output = Result<(), ClipboardError>> + 'a
    where
        Self: 'a;

    /// Removes the clipboard's plain-text value.
    ///
    /// This does not claim exclusive access to the shared clipboard and does not promise to remove
    /// rich text, images, files, or other platform-specific representations.
    fn clear<'a>(&'a mut self) -> Self::ClearFuture<'a>;
}

/// A thin facade over caller-owned, statically selected clipboard-backend state.
pub struct Clipboard<B> {
    backend: B,
}

impl<B: ClipboardBackend> Clipboard<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    ///
    /// The backend's current view does not guarantee that a later clipboard operation will
    /// succeed.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Reads the current plain-text value as an owned UTF-8 string, or `None` if none is readable.
    pub async fn read(&mut self) -> Result<Option<String>, ClipboardError> {
        self.backend.read().await
    }

    /// Writes borrowed UTF-8 text as the clipboard's plain-text value.
    ///
    /// The input borrow lasts until this future completes or is dropped. Dropping a started
    /// operation suppresses its result but may not prevent a native write from taking effect.
    pub async fn write(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.backend.write(text).await
    }

    /// Removes the clipboard's plain-text value.
    ///
    /// Dropping a started operation suppresses its result but may not prevent a native clear from
    /// taking effect.
    pub async fn clear(&mut self) -> Result<(), ClipboardError> {
        self.backend.clear().await
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
    use core::cell::Cell;
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    struct Backend {
        value: Option<String>,
        read_error: Option<ClipboardError>,
        read_count: Rc<Cell<u32>>,
        write_count: u32,
        clear_count: u32,
        cancel_count: Rc<Cell<u32>>,
        result_count: Rc<Cell<u32>>,
    }

    impl Backend {
        fn new(value: Option<&str>) -> Self {
            Self {
                value: value.map(String::from),
                read_error: None,
                read_count: Rc::new(Cell::new(0)),
                write_count: 0,
                clear_count: 0,
                cancel_count: Rc::new(Cell::new(0)),
                result_count: Rc::new(Cell::new(0)),
            }
        }
    }

    trait FakeOperation {
        type Output: Unpin;

        fn start(self, backend: &mut Backend) -> Result<Self::Output, ClipboardError>;
    }

    struct ReadOp;

    impl FakeOperation for ReadOp {
        type Output = Option<String>;

        fn start(self, backend: &mut Backend) -> Result<Self::Output, ClipboardError> {
            backend.read_count.set(backend.read_count.get() + 1);
            match backend.read_error {
                Some(error) => Err(error),
                None => Ok(backend.value.clone()),
            }
        }
    }

    struct WriteOp<'a>(&'a str);

    impl FakeOperation for WriteOp<'_> {
        type Output = ();

        fn start(self, backend: &mut Backend) -> Result<Self::Output, ClipboardError> {
            backend.write_count += 1;
            backend.value = Some(String::from(self.0));
            Ok(())
        }
    }

    struct ClearOp;

    impl FakeOperation for ClearOp {
        type Output = ();

        fn start(self, backend: &mut Backend) -> Result<Self::Output, ClipboardError> {
            backend.clear_count += 1;
            backend.value = None;
            Ok(())
        }
    }

    struct PendingOnce<'a, O: FakeOperation> {
        backend: &'a mut Backend,
        operation: Option<O>,
        result: Option<Result<O::Output, ClipboardError>>,
        started: bool,
        completed: bool,
    }

    impl<'a, O: FakeOperation> PendingOnce<'a, O> {
        fn new(backend: &'a mut Backend, operation: O) -> Self {
            Self {
                backend,
                operation: Some(operation),
                result: None,
                started: false,
                completed: false,
            }
        }
    }

    impl<O: FakeOperation + Unpin> Future for PendingOnce<'_, O> {
        type Output = Result<O::Output, ClipboardError>;

        fn poll(self: core::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                this.started = true;
                this.result = Some(
                    this.operation
                        .take()
                        .expect("fake operation starts once")
                        .start(this.backend),
                );
                context.waker().wake_by_ref();
                return Poll::Pending;
            }
            this.completed = true;
            this.backend
                .result_count
                .set(this.backend.result_count.get() + 1);
            Poll::Ready(this.result.take().expect("fake result exists"))
        }
    }

    impl<O: FakeOperation> Drop for PendingOnce<'_, O> {
        fn drop(&mut self) {
            if self.started && !self.completed {
                let count = self.backend.cancel_count.get();
                self.backend.cancel_count.set(count + 1);
            }
        }
    }

    impl ClipboardBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type ReadFuture<'a>
            = PendingOnce<'a, ReadOp>
        where
            Self: 'a;

        fn read<'a>(&'a mut self) -> Self::ReadFuture<'a> {
            PendingOnce::new(self, ReadOp)
        }

        type WriteFuture<'a>
            = PendingOnce<'a, WriteOp<'a>>
        where
            Self: 'a;

        fn write<'a>(&'a mut self, text: &'a str) -> Self::WriteFuture<'a> {
            PendingOnce::new(self, WriteOp(text))
        }

        type ClearFuture<'a>
            = PendingOnce<'a, ClearOp>
        where
            Self: 'a;

        fn clear<'a>(&'a mut self) -> Self::ClearFuture<'a> {
            PendingOnce::new(self, ClearOp)
        }
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
    fn read_returns_owned_text_or_none_and_preserves_backend_errors() {
        let mut clipboard = Clipboard::new(Backend::new(Some("plain text 🦀")));
        assert_eq!(clipboard.availability(), Availability::Available);
        let owned = run_pending_once(clipboard.read()).unwrap().unwrap();
        clipboard.backend_mut().value = Some(String::from("changed later"));
        assert_eq!(owned, "plain text 🦀");
        assert_eq!(clipboard.backend().read_count.get(), 1);
        assert_eq!(clipboard.backend().result_count.get(), 1);

        let mut clipboard = Clipboard::new(Backend::new(None));
        assert_eq!(run_pending_once(clipboard.read()), Ok(None));
        assert_eq!(clipboard.backend().read_count.get(), 1);

        let mut backend = Backend::new(None);
        let code = PlatformErrorCode::new(-41).unwrap();
        let error = ClipboardError::Backend(
            Error::new(ErrorKind::PermissionDenied).with_platform_code(code),
        );
        backend.read_error = Some(error);
        let mut clipboard = Clipboard::new(backend);
        assert_eq!(run_pending_once(clipboard.read()), Err(error));
        assert_eq!(clipboard.backend().read_count.get(), 1);
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(error.platform_code(), Some(code));
    }

    #[test]
    fn write_borrows_text_and_clear_removes_the_plain_text_value() {
        let mut clipboard = Clipboard::new(Backend::new(Some("old")));
        let text = String::from("new text");
        assert_eq!(run_pending_once(clipboard.write(&text)), Ok(()));
        assert_eq!(clipboard.backend().value.as_deref(), Some("new text"));
        assert_eq!(clipboard.backend().write_count, 1);

        assert_eq!(run_pending_once(clipboard.clear()), Ok(()));
        assert_eq!(clipboard.backend().value, None);
        assert_eq!(clipboard.backend().clear_count, 1);
    }

    #[test]
    fn dropping_an_unpolled_operation_starts_no_backend_work() {
        let mut clipboard = Clipboard::new(Backend::new(Some("existing")));
        let read_count = clipboard.backend().read_count.clone();
        drop(clipboard.write("not started"));
        drop(clipboard.read());
        drop(clipboard.clear());
        assert_eq!(read_count.get(), 0);
        assert_eq!(clipboard.backend().write_count, 0);
        assert_eq!(clipboard.backend().clear_count, 0);
        assert_eq!(clipboard.backend().cancel_count.get(), 0);
        assert_eq!(clipboard.backend().result_count.get(), 0);
        assert_eq!(clipboard.backend().value.as_deref(), Some("existing"));
    }

    #[test]
    fn read_starts_on_first_poll_and_dropping_pending_read_suppresses_its_result() {
        let mut clipboard = Clipboard::new(Backend::new(Some("late result")));
        let read_count = clipboard.backend().read_count.clone();
        {
            let mut future = pin!(clipboard.read());
            let mut context = Context::from_waker(Waker::noop());
            assert_eq!(read_count.get(), 0);
            assert!(future.as_mut().poll(&mut context).is_pending());
            assert_eq!(read_count.get(), 1);
        }
        assert_eq!(clipboard.backend().cancel_count.get(), 1);
        assert_eq!(clipboard.backend().result_count.get(), 0);
        assert_eq!(clipboard.backend().value.as_deref(), Some("late result"));
    }

    #[test]
    fn dropping_a_pending_write_cancels_interest_and_suppresses_its_result() {
        let mut clipboard = Clipboard::new(Backend::new(Some("before")));
        {
            let mut future = pin!(clipboard.write("after"));
            let mut context = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut context).is_pending());
        }
        assert_eq!(clipboard.backend().write_count, 1);
        assert_eq!(clipboard.backend().value.as_deref(), Some("after"));
        assert_eq!(clipboard.backend().cancel_count.get(), 1);
        assert_eq!(clipboard.backend().result_count.get(), 0);
    }

    #[test]
    fn dropping_a_pending_clear_cancels_interest_and_suppresses_its_result() {
        let mut clipboard = Clipboard::new(Backend::new(Some("before")));
        {
            let mut future = pin!(clipboard.clear());
            let mut context = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut context).is_pending());
        }
        assert_eq!(clipboard.backend().clear_count, 1);
        assert_eq!(clipboard.backend().value, None);
        assert_eq!(clipboard.backend().cancel_count.get(), 1);
        assert_eq!(clipboard.backend().result_count.get(), 0);
    }
}
