#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Calendar authorization and event-range contracts."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
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

/// A finite Unix timestamp in seconds, preserving the native date's fractional precision.
///
/// iOS EventKit uses `NSTimeInterval` (`f64`) for `NSDate`; this type keeps that value without
/// rounding it to a finer, unsupported precision.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct CalendarEventTime(f64);

impl CalendarEventTime {
    /// Creates a timestamp from finite seconds since 1970-01-01T00:00:00Z.
    pub fn from_unix_seconds(seconds: f64) -> Result<Self, CalendarEventError> {
        if !seconds.is_finite() {
            return Err(CalendarEventError::InvalidTime);
        }
        Ok(Self(seconds))
    }

    /// Returns seconds since the Unix epoch with the original floating-point precision.
    pub const fn unix_seconds(self) -> f64 {
        self.0
    }
}

/// A finite caller-selected query range with an ordered start and end.
///
/// The backend applies its native event-range matching semantics; this value does not add a
/// second filter over native results.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarEventRange {
    start: CalendarEventTime,
    end: CalendarEventTime,
}

impl CalendarEventRange {
    /// Creates a non-empty range from `start` to `end` Unix seconds.
    pub fn new(
        start: CalendarEventTime,
        end: CalendarEventTime,
    ) -> Result<Self, CalendarEventError> {
        if start >= end {
            return Err(CalendarEventError::InvalidRange);
        }
        Ok(Self { start, end })
    }

    /// Returns the query start bound.
    pub const fn start(self) -> CalendarEventTime {
        self.start
    }

    /// Returns the query end bound.
    pub const fn end(self) -> CalendarEventTime {
        self.end
    }
}

/// One Rust-owned summary for a Calendar event returned by the selected backend.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    event_identifier: Option<String>,
    title: String,
    start: CalendarEventTime,
    end: CalendarEventTime,
    all_day: bool,
    location: Option<String>,
    notes: Option<String>,
}

impl CalendarEvent {
    /// Creates a summary with owned text and an ordered event interval.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_identifier: Option<String>,
        title: String,
        start: CalendarEventTime,
        end: CalendarEventTime,
        all_day: bool,
        location: Option<String>,
        notes: Option<String>,
    ) -> Result<Self, CalendarEventError> {
        if start > end {
            return Err(CalendarEventError::InvalidEventInterval);
        }
        Ok(Self {
            event_identifier,
            title,
            start,
            end,
            all_day,
            location,
            notes,
        })
    }

    /// Borrows the optional native event identifier; it is not a durable global identity.
    pub fn event_identifier(&self) -> Option<&str> {
        self.event_identifier.as_deref()
    }

    /// Borrows the owned event title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the event's native start instant.
    pub const fn start(&self) -> CalendarEventTime {
        self.start
    }

    /// Returns the event's native end instant.
    pub const fn end(&self) -> CalendarEventTime {
        self.end
    }

    /// Returns whether the event is marked as all-day by the backend.
    pub const fn is_all_day(&self) -> bool {
        self.all_day
    }

    /// Borrows the optional event location.
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// Borrows the optional event notes.
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

/// An error returned by a Calendar event-range query.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarEventError {
    /// A timestamp was not finite.
    InvalidTime,
    /// The caller supplied an empty or reversed query range.
    InvalidRange,
    /// The caller supplied an event interval whose end precedes its start.
    InvalidEventInterval,
    /// The caller's range exceeds the selected backend's documented query limit.
    RangeTooWide,
    /// The native event query cannot represent an event date or interval.
    InvalidNativeEvent,
    /// The current authorization is not full event-data access.
    NotFullAccess(CalendarAuthorizationStatus),
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl CalendarEventError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidTime
            | Self::InvalidRange
            | Self::InvalidEventInterval
            | Self::RangeTooWide => ErrorKind::InvalidInput,
            Self::InvalidNativeEvent => ErrorKind::Platform,
            Self::NotFullAccess(_) => ErrorKind::PermissionDenied,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
            _ => None,
        }
    }
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

/// A statically selected backend for synchronous Calendar event-range reads.
///
/// `events_in_range` must recheck current authorization at the native read boundary, require full
/// event-data access, fail closed for every other status, and never prompt. It may block; the
/// caller chooses the thread on which to invoke it. Returned events and text must be Rust-owned.
pub trait CalendarEventBackend {
    /// Reads native events matched by the caller's finite range.
    fn events_in_range(
        &self,
        range: CalendarEventRange,
    ) -> Result<Vec<CalendarEvent>, CalendarEventError>;
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

impl<B: CalendarEventBackend> Calendar<B> {
    /// Reads events in the caller-selected range through the owned backend.
    ///
    /// This operation is synchronous and may block depending on the backend. It does not create a
    /// worker thread or executor; callers may invoke it off-main when their backend permits it.
    pub fn events_in_range(
        &self,
        range: CalendarEventRange,
    ) -> Result<Vec<CalendarEvent>, CalendarEventError> {
        self.backend.events_in_range(range)
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
