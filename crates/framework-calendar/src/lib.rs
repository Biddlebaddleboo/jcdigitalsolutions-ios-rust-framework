#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Calendar event-authorization status and explicit full-access request contract."]

use core::future::Future;
use framework_core::{Error, ErrorKind, PlatformErrorCode};

#[cfg(test)]
extern crate std;

/// The normalized authorization status for Calendar event data.
///
/// `WriteOnly` is distinct from `FullAccess`: it reports that the platform permits saving new
/// events but does not grant full event-data access. This contract reports status only and does
/// not expose an operation that requests write-only access.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarAuthorizationStatus {
    /// The platform returned an authorization status this contract cannot classify.
    Unknown,
    /// The platform has not yet asked the user.
    NotDetermined,
    /// Access is blocked by platform policy or device restrictions.
    Restricted,
    /// The user denied access.
    Denied,
    /// The platform permits saving new events but does not grant full access.
    WriteOnly,
    /// The platform grants full access to Calendar event data.
    FullAccess,
}

/// An error returned by a Calendar authorization backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarError {
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl CalendarError {
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

/// A statically selected backend for Calendar event-authorization status and full access.
///
/// `authorization_status` must not prompt. `request_full_access` is the only prompt-capable
/// operation in this contract, and a backend must defer native work until its returned future is
/// first polled. Dropping an unpolled future starts no backend work. Its result is the status
/// queried after native request completion, not merely a callback's grant Boolean. Dropping a
/// pending future detaches the caller from its result; it cannot promise to dismiss a native prompt
/// already shown. Callback state must stay valid until the native callback completes and must
/// complete at most once. No executor or `Send` requirement is imposed.
pub trait CalendarAuthorizationBackend {
    /// Reports current Calendar event authorization without prompting.
    fn authorization_status(&self) -> CalendarAuthorizationStatus;

    /// The future type for an explicit full-access request.
    type RequestFullAccessFuture<'a>: Future<Output = Result<CalendarAuthorizationStatus, CalendarError>>
        + 'a
    where
        Self: 'a;

    /// Explicitly requests full access to Calendar event data.
    fn request_full_access<'a>(&'a mut self) -> Self::RequestFullAccessFuture<'a>;
}

/// A small owner of a caller-selected Calendar authorization backend.
pub struct Calendar<B> {
    backend: B,
}

impl<B> Calendar<B> {
    /// Creates a client with the supplied backend and performs no native work.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Borrows the selected backend for platform-specific controls.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the selected backend for platform-specific controls.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

impl<B: CalendarAuthorizationBackend> Calendar<B> {
    /// Reports current Calendar event authorization without prompting.
    pub fn authorization_status(&self) -> CalendarAuthorizationStatus {
        self.backend.authorization_status()
    }

    /// Explicitly requests full access to Calendar event data.
    ///
    /// Creating or dropping the returned future before its first poll starts no backend work.
    /// The result is based on a status query after the native completion callback. Dropping a
    /// pending future abandons the result but cannot be assumed to dismiss a prompt already shown.
    pub async fn request_full_access(
        &mut self,
    ) -> Result<CalendarAuthorizationStatus, CalendarError> {
        self.backend.request_full_access().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::task::Wake;

    struct FakeBackend {
        status: CalendarAuthorizationStatus,
        requests: Rc<Cell<usize>>,
        polls: Rc<Cell<usize>>,
    }

    struct FakeRequest {
        polls: Rc<Cell<usize>>,
        status: CalendarAuthorizationStatus,
    }

    impl Future for FakeRequest {
        type Output = Result<CalendarAuthorizationStatus, CalendarError>;

        fn poll(self: core::pin::Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            this.polls.set(this.polls.get() + 1);
            Poll::Ready(Ok(this.status))
        }
    }

    impl CalendarAuthorizationBackend for FakeBackend {
        fn authorization_status(&self) -> CalendarAuthorizationStatus {
            self.status
        }

        type RequestFullAccessFuture<'a>
            = FakeRequest
        where
            Self: 'a;

        fn request_full_access<'a>(&'a mut self) -> Self::RequestFullAccessFuture<'a> {
            self.requests.set(self.requests.get() + 1);
            FakeRequest {
                polls: self.polls.clone(),
                status: self.status,
            }
        }
    }

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn unknown_and_not_determined_are_distinct_statuses() {
        assert_ne!(
            CalendarAuthorizationStatus::Unknown,
            CalendarAuthorizationStatus::NotDetermined
        );
    }

    #[test]
    fn write_only_and_full_access_are_distinct_statuses() {
        assert_ne!(
            CalendarAuthorizationStatus::WriteOnly,
            CalendarAuthorizationStatus::FullAccess
        );
    }

    #[test]
    fn status_query_forwards_backend_status_without_starting_a_request() {
        let requests = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let calendar = Calendar::new(FakeBackend {
            status: CalendarAuthorizationStatus::WriteOnly,
            requests: requests.clone(),
            polls: polls.clone(),
        });

        assert_eq!(
            calendar.authorization_status(),
            CalendarAuthorizationStatus::WriteOnly
        );
        assert_eq!(requests.get(), 0);
        assert_eq!(polls.get(), 0);
    }

    #[test]
    fn explicit_request_starts_when_the_facade_future_is_polled() {
        let requests = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let mut calendar = Calendar::new(FakeBackend {
            status: CalendarAuthorizationStatus::FullAccess,
            requests: requests.clone(),
            polls: polls.clone(),
        });
        let unpolled = calendar.request_full_access();
        assert_eq!(requests.get(), 0);
        assert_eq!(polls.get(), 0);
        drop(unpolled);
        assert_eq!(requests.get(), 0);
        assert_eq!(polls.get(), 0);

        let mut future = pin!(calendar.request_full_access());
        assert_eq!(requests.get(), 0);
        assert_eq!(polls.get(), 0);
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        assert!(matches!(
            future.as_mut().poll(&mut context),
            Poll::Ready(Ok(CalendarAuthorizationStatus::FullAccess))
        ));
        assert_eq!(requests.get(), 1);
        assert_eq!(polls.get(), 1);
    }

    #[test]
    fn backend_error_preserves_kind_and_optional_platform_code() {
        let code = PlatformErrorCode::new(-42).unwrap();
        let error = CalendarError::Backend(
            Error::new(ErrorKind::PermissionDenied).with_platform_code(code),
        );
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(error.platform_code(), Some(code));

        let error = CalendarError::Backend(Error::new(ErrorKind::Unavailable));
        assert_eq!(error.kind(), ErrorKind::Unavailable);
        assert_eq!(error.platform_code(), None);
    }
}
