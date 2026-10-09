#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, read/write Photos authorization status and request contract."]

#[cfg(test)]
extern crate std;

use core::future::Future;

/// The native Photos authorization result for read/write access.
///
/// `Limited` is distinct from `Authorized`. This type reports authorization status only; it does
/// not enumerate assets, read or modify library data, or establish that any later operation will
/// succeed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PhotoLibraryAuthorizationStatus {
    /// Authorization has not yet been determined for this access level.
    NotDetermined,
    /// Access is restricted by system policy.
    Restricted,
    /// Access was denied.
    Denied,
    /// Access is limited to the assets selected by the user.
    Limited,
    /// Read/write access is authorized without the limited status.
    Authorized,
    /// The native framework returned a status value not represented by this version.
    Unknown,
}

/// A static backend for Photos read/write authorization only.
///
/// The synchronous status query must not prompt. A request starts no earlier than the first poll
/// of its returned future. Dropping that future suppresses its Rust result; it does not promise
/// cancellation or dismissal of native authorization UI already requested. No executor or `Send`
/// requirement is imposed.
pub trait PhotoLibraryAuthorizationBackend {
    /// The future returned by one explicit authorization request.
    type RequestAuthorizationFuture<'a>: Future<Output = PhotoLibraryAuthorizationStatus> + 'a
    where
        Self: 'a;

    /// Queries current read/write authorization status without requesting authorization.
    fn authorization_status(&self) -> PhotoLibraryAuthorizationStatus;

    /// Returns one request whose native work begins on first poll.
    fn request_authorization(&mut self) -> Self::RequestAuthorizationFuture<'_>;
}

/// A thin facade over caller-owned, statically selected Photos authorization state.
pub struct PhotoLibrary<B> {
    backend: B,
}

impl<B: PhotoLibraryAuthorizationBackend> PhotoLibrary<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Queries read/write authorization status without requesting authorization.
    pub fn authorization_status(&self) -> PhotoLibraryAuthorizationStatus {
        self.backend.authorization_status()
    }

    /// Explicitly requests read/write authorization.
    ///
    /// The backend begins native work when the returned future is first polled. Dropping a
    /// pending future abandons its Rust result but does not promise cancellation of native work.
    pub fn request_authorization(&mut self) -> B::RequestAuthorizationFuture<'_> {
        self.backend.request_authorization()
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
mod tests {
    use super::*;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    struct FakeBackend {
        status: PhotoLibraryAuthorizationStatus,
        request_count: Arc<AtomicUsize>,
    }

    struct FakeRequest {
        status: PhotoLibraryAuthorizationStatus,
        request_count: Arc<AtomicUsize>,
        started: bool,
    }

    impl Future for FakeRequest {
        type Output = PhotoLibraryAuthorizationStatus;

        fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.started {
                self.started = true;
                self.request_count.fetch_add(1, Ordering::SeqCst);
            }
            Poll::Ready(self.status)
        }
    }

    impl PhotoLibraryAuthorizationBackend for FakeBackend {
        type RequestAuthorizationFuture<'a> = FakeRequest;

        fn authorization_status(&self) -> PhotoLibraryAuthorizationStatus {
            self.status
        }

        fn request_authorization(&mut self) -> Self::RequestAuthorizationFuture<'_> {
            FakeRequest {
                status: self.status,
                request_count: Arc::clone(&self.request_count),
                started: false,
            }
        }
    }

    fn library(
        status: PhotoLibraryAuthorizationStatus,
    ) -> (PhotoLibrary<FakeBackend>, Arc<AtomicUsize>) {
        let request_count = Arc::new(AtomicUsize::new(0));
        (
            PhotoLibrary::new(FakeBackend {
                status,
                request_count: Arc::clone(&request_count),
            }),
            request_count,
        )
    }

    #[test]
    fn limited_status_stays_distinct_from_authorized() {
        let (library, _) = library(PhotoLibraryAuthorizationStatus::Limited);
        assert_eq!(
            library.authorization_status(),
            PhotoLibraryAuthorizationStatus::Limited
        );
        assert_ne!(
            library.authorization_status(),
            PhotoLibraryAuthorizationStatus::Authorized
        );
    }

    #[test]
    fn request_starts_only_on_first_poll_and_returns_status() {
        let (mut library, request_count) = library(PhotoLibraryAuthorizationStatus::Limited);
        let mut request = library.request_authorization();
        assert_eq!(request_count.load(Ordering::SeqCst), 0);
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        assert_eq!(
            Pin::new(&mut request).poll(&mut context),
            Poll::Ready(PhotoLibraryAuthorizationStatus::Limited)
        );
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dropping_unpolled_request_does_not_start_it() {
        let (mut library, request_count) = library(PhotoLibraryAuthorizationStatus::NotDetermined);
        drop(library.request_authorization());
        assert_eq!(request_count.load(Ordering::SeqCst), 0);
    }
}
