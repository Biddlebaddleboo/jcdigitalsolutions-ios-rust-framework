#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Calendar and Reminders authorization, event, and create contracts."]

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

/// The normalized authorization status for Reminders data.
///
/// EventKit currently exposes full access as the only requestable reminder-access level. A
/// status that does not safely identify full Reminders access, including a write-only raw value,
/// maps to `Unknown` rather than implying that reminder creation is authorized.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ReminderAuthorizationStatus {
    /// The platform returned a status this contract cannot classify as a Reminders state.
    Unknown,
    /// The platform has not yet asked the user for Reminders access.
    NotDetermined,
    /// Access is blocked by platform policy or device restrictions.
    Restricted,
    /// The user denied Reminders access.
    Denied,
    /// The platform grants full read and write access to Reminders data.
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

/// A Rust-owned snapshot of an existing event-capable calendar that currently allows changes.
///
/// Its identifier is an opaque native selection token, not a durable identity. A backend must
/// resolve it again and recheck writability when creating an event because calendars can change
/// after this snapshot is returned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WritableEventCalendar {
    identifier: String,
    title: String,
}

impl WritableEventCalendar {
    /// Creates a calendar snapshot with a non-empty opaque identifier and owned title.
    pub fn new(identifier: String, title: String) -> Result<Self, CalendarEventCreateError> {
        if identifier.is_empty() {
            return Err(CalendarEventCreateError::InvalidCalendarIdentifier);
        }
        Ok(Self { identifier, title })
    }

    /// Borrows the opaque calendar identifier for backend selection.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Borrows the owned calendar title for caller display.
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// Caller-owned fields for creating or replacing one Calendar event.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEventDraft {
    title: String,
    start: CalendarEventTime,
    end: CalendarEventTime,
    all_day: bool,
    location: Option<String>,
    notes: Option<String>,
}

impl CalendarEventDraft {
    /// Creates one draft with finite ordered times and unchanged caller text.
    pub fn new(
        title: String,
        start: CalendarEventTime,
        end: CalendarEventTime,
        all_day: bool,
        location: Option<String>,
        notes: Option<String>,
    ) -> Result<Self, CalendarEventCreateError> {
        if start >= end {
            return Err(CalendarEventCreateError::InvalidEventInterval);
        }
        Ok(Self {
            title,
            start,
            end,
            all_day,
            location,
            notes,
        })
    }

    /// Borrows the exact caller-provided title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the caller-provided start instant.
    pub const fn start(&self) -> CalendarEventTime {
        self.start
    }

    /// Returns the caller-provided end instant.
    pub const fn end(&self) -> CalendarEventTime {
        self.end
    }

    /// Returns the caller-provided all-day flag.
    pub const fn is_all_day(&self) -> bool {
        self.all_day
    }

    /// Borrows the optional caller-provided location.
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// Borrows the optional caller-provided notes.
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

/// An error returned while selecting a writable calendar or creating one event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarEventCreateError {
    /// The caller supplied an empty calendar identifier.
    InvalidCalendarIdentifier,
    /// The caller supplied a non-finite, empty, or reversed event interval.
    InvalidEventInterval,
    /// Current authorization does not permit full event-data access.
    NotFullAccess(CalendarAuthorizationStatus),
    /// The selected calendar identifier no longer resolves in the event store.
    CalendarNotFound,
    /// The selected calendar does not support events.
    CalendarDoesNotSupportEvents,
    /// The selected calendar does not currently allow changes.
    CalendarNotWritable,
    /// The backend returned a framework or native platform error.
    Backend(Error),
}

impl CalendarEventCreateError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidCalendarIdentifier | Self::InvalidEventInterval => ErrorKind::InvalidInput,
            Self::NotFullAccess(_) => ErrorKind::PermissionDenied,
            Self::CalendarNotFound => ErrorKind::NotFound,
            Self::CalendarDoesNotSupportEvents => ErrorKind::Unsupported,
            Self::CalendarNotWritable => ErrorKind::Unavailable,
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

/// A caller-owned title for creating one reminder in the system default list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarReminderDraft {
    title: String,
}

impl CalendarReminderDraft {
    /// Creates a reminder draft with a non-empty title preserved exactly as supplied.
    pub fn new(title: String) -> Result<Self, CalendarReminderCreateError> {
        if title.is_empty() {
            return Err(CalendarReminderCreateError::EmptyTitle);
        }
        Ok(Self { title })
    }

    /// Borrows the exact caller-provided title.
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// An error returned while creating one Reminders item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarReminderCreateError {
    /// The caller supplied an empty title.
    EmptyTitle,
    /// Current authorization does not permit full Reminders access.
    NotFullAccess(ReminderAuthorizationStatus),
    /// EventKit has no system default Reminders list.
    DefaultListUnavailable,
    /// The backend returned a framework or native platform error.
    Backend(Error),
}

impl CalendarReminderCreateError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::EmptyTitle => ErrorKind::InvalidInput,
            Self::NotFullAccess(_) => ErrorKind::PermissionDenied,
            Self::DefaultListUnavailable => ErrorKind::Unavailable,
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

/// An error returned while deleting one caller-selected Calendar event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarEventDeleteError {
    /// The selected event summary has no non-empty native event identifier.
    InvalidEventIdentifier,
    /// Current authorization does not permit full event-data access.
    NotFullAccess(CalendarAuthorizationStatus),
    /// The selected identifier or exact event snapshot no longer resolves.
    EventNotFound,
    /// The resolved event's calendar no longer supports events.
    CalendarDoesNotSupportEvents,
    /// The resolved event's calendar does not currently allow changes.
    CalendarNotWritable,
    /// The backend returned a framework or native platform error.
    Backend(Error),
}

impl CalendarEventDeleteError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidEventIdentifier => ErrorKind::InvalidInput,
            Self::NotFullAccess(_) => ErrorKind::PermissionDenied,
            Self::EventNotFound => ErrorKind::NotFound,
            Self::CalendarDoesNotSupportEvents => ErrorKind::Unsupported,
            Self::CalendarNotWritable => ErrorKind::Unavailable,
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

/// An error returned while updating one caller-selected Calendar event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CalendarEventUpdateError {
    /// The selected event summary has no non-empty native event identifier.
    InvalidEventIdentifier,
    /// The caller supplied an empty or reversed event interval.
    InvalidEventInterval,
    /// Current authorization does not permit full event-data access.
    NotFullAccess(CalendarAuthorizationStatus),
    /// The selected identifier or exact event snapshot no longer resolves.
    EventNotFound,
    /// The resolved event's calendar no longer supports events.
    CalendarDoesNotSupportEvents,
    /// The resolved event's calendar does not currently allow changes.
    CalendarNotWritable,
    /// The backend returned a framework or native platform error.
    Backend(Error),
}

impl CalendarEventUpdateError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidEventIdentifier | Self::InvalidEventInterval => ErrorKind::InvalidInput,
            Self::NotFullAccess(_) => ErrorKind::PermissionDenied,
            Self::EventNotFound => ErrorKind::NotFound,
            Self::CalendarDoesNotSupportEvents => ErrorKind::Unsupported,
            Self::CalendarNotWritable => ErrorKind::Unavailable,
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

/// A statically selected backend for non-prompting Reminders status and explicit full access.
///
/// Status queries must not prompt. EventKit currently supports only a full-access reminder
/// request; a backend must defer that request until the returned future is first polled and report
/// status queried after native completion, not the callback's grant Boolean. Dropping an unpolled
/// future starts no native work. Dropping a pending future detaches result interest but cannot
/// promise to dismiss a prompt already shown. Callback state must remain safe through completion
/// and settle at most once. No executor or `Send` requirement is imposed.
pub trait CalendarReminderAuthorizationBackend {
    /// Reports current Reminders authorization without prompting.
    fn reminder_authorization_status(&self) -> ReminderAuthorizationStatus;

    /// The future type for an explicit full Reminders access request.
    type RequestFullReminderAccessFuture<'a>: Future<Output = Result<ReminderAuthorizationStatus, CalendarError>>
        + 'a
    where
        Self: 'a;

    /// Explicitly requests full read and write access to Reminders data.
    fn request_full_reminder_access<'a>(&'a mut self) -> Self::RequestFullReminderAccessFuture<'a>;
}

/// A statically selected backend for creating one reminder in the system default list.
///
/// Creation must recheck current Reminders status, require `FullAccess`, obtain only the system
/// default Reminders list, and save exactly one caller-described reminder. It must not prompt,
/// read or enumerate existing reminders, or return native EventKit objects. Calls may block and
/// leave thread choice to the caller.
pub trait CalendarReminderCreateBackend {
    /// Creates exactly one caller-titled reminder in the current system default list.
    fn create_reminder(
        &self,
        draft: &CalendarReminderDraft,
    ) -> Result<(), CalendarReminderCreateError>;
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

/// A statically selected backend for non-prompting writable-calendar selection and event creation.
///
/// Both operations must recheck current authorization and require `FullAccess`; neither may
/// prompt. Calendar snapshots must contain only event-capable calendars that currently allow
/// modifications. Creation must resolve the selected opaque identifier again, recheck those
/// properties, and save exactly one event or return an error. Calls may block and leave thread
/// choice to the caller.
pub trait CalendarEventCreateBackend {
    /// Returns Rust-owned snapshots of calendars that currently accept events and modifications.
    fn writable_event_calendars(
        &self,
    ) -> Result<Vec<WritableEventCalendar>, CalendarEventCreateError>;

    /// Creates exactly one caller-described event in the selected calendar.
    fn create_event(
        &self,
        calendar: &WritableEventCalendar,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventCreateError>;
}

/// A statically selected backend for deleting one caller-selected event.
///
/// A backend must reject absent or empty identifiers before native lookup, recheck current
/// `FullAccess` without prompting, resolve only the selected event, verify its current event
/// calendar is modifiable, and remove exactly one occurrence with the narrowest native scope.
/// Calls may block and leave thread choice to the caller.
pub trait CalendarEventDeleteBackend {
    /// Deletes exactly the event represented by the caller-selected summary.
    fn delete_event(&self, event: &CalendarEvent) -> Result<(), CalendarEventDeleteError>;
}

/// A statically selected backend for updating one caller-selected event occurrence.
///
/// A backend must reject absent or empty identifiers before native lookup, recheck current
/// `FullAccess` without prompting, resolve only the selected event, verify its current identifier
/// and exact start/end snapshot, and confirm its current event calendar is modifiable. It must
/// replace only fields from the supplied draft, preserve the event's current calendar, and commit
/// exactly one occurrence. Calls may block and leave thread choice to the caller.
pub trait CalendarEventUpdateBackend {
    /// Replaces the caller-described fields of exactly one selected event occurrence.
    fn update_event(
        &self,
        event: &CalendarEvent,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventUpdateError>;
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

impl<B: CalendarReminderAuthorizationBackend> Calendar<B> {
    /// Reports current Reminders authorization without prompting.
    pub fn reminder_authorization_status(&self) -> ReminderAuthorizationStatus {
        self.backend.reminder_authorization_status()
    }

    /// Explicitly requests full read and write access to Reminders data.
    ///
    /// Creating or dropping the returned future before its first poll starts no backend work. The
    /// result is based on a status query after native completion. Dropping a pending future
    /// abandons the result but cannot be assumed to dismiss a prompt already shown.
    pub async fn request_full_reminder_access(
        &mut self,
    ) -> Result<ReminderAuthorizationStatus, CalendarError> {
        self.backend.request_full_reminder_access().await
    }
}

impl<B: CalendarReminderCreateBackend> Calendar<B> {
    /// Creates one reminder in the system default Reminders list after current full access is
    /// verified by the backend.
    ///
    /// The operation is synchronous and may block. It never prompts and does not read existing
    /// reminders. Success reports only that the native save call succeeded, not later sync or
    /// durable persistence.
    pub fn create_reminder(
        &self,
        draft: &CalendarReminderDraft,
    ) -> Result<(), CalendarReminderCreateError> {
        if draft.title.is_empty() {
            return Err(CalendarReminderCreateError::EmptyTitle);
        }
        self.backend.create_reminder(draft)
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

impl<B: CalendarEventCreateBackend> Calendar<B> {
    /// Returns a non-prompting snapshot of calendars that can currently accept events.
    pub fn writable_event_calendars(
        &self,
    ) -> Result<Vec<WritableEventCalendar>, CalendarEventCreateError> {
        self.backend.writable_event_calendars()
    }

    /// Creates exactly one event in a caller-selected writable calendar snapshot.
    ///
    /// The backend re-resolves the snapshot identifier and rechecks authorization and calendar
    /// writability immediately before native mutation. EventKit identifiers can change after
    /// synchronization; if a snapshot becomes stale, callers should request a fresh list and
    /// select again. Success reports only that the native save API succeeded and returns no event
    /// identifier.
    pub fn create_event(
        &self,
        calendar: &WritableEventCalendar,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventCreateError> {
        self.backend.create_event(calendar, draft)
    }
}

impl<B: CalendarEventDeleteBackend> Calendar<B> {
    /// Deletes exactly one caller-selected event returned by an event-range read.
    ///
    /// Missing or empty native identifiers fail before the backend is called. Event identifiers
    /// can become stale after sync or a calendar move; the backend must resolve the selected ID
    /// again, verify the selected occurrence and current calendar writability, and fail without
    /// mutation if the snapshot no longer matches. The operation is synchronous and may block.
    pub fn delete_event(&self, event: &CalendarEvent) -> Result<(), CalendarEventDeleteError> {
        let identifier = event.event_identifier();
        if identifier.is_none_or(|identifier| identifier.is_empty()) {
            return Err(CalendarEventDeleteError::InvalidEventIdentifier);
        }
        self.backend.delete_event(event)
    }
}

impl<B: CalendarEventUpdateBackend> Calendar<B> {
    /// Updates exactly one caller-selected event occurrence from a complete replacement draft.
    ///
    /// Missing or empty native identifiers fail before backend dispatch. The backend must
    /// re-resolve the selected ID, verify the exact selected occurrence and current calendar
    /// writability, then update only title, start, end, all-day, location, and notes. The event's
    /// current calendar is preserved, and stale snapshots fail without mutation. The operation is
    /// synchronous and may block.
    pub fn update_event(
        &self,
        event: &CalendarEvent,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventUpdateError> {
        let identifier = event.event_identifier();
        if identifier.is_none_or(|identifier| identifier.is_empty()) {
            return Err(CalendarEventUpdateError::InvalidEventIdentifier);
        }
        if draft.start() >= draft.end() {
            return Err(CalendarEventUpdateError::InvalidEventInterval);
        }
        self.backend.update_event(event, draft)
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
