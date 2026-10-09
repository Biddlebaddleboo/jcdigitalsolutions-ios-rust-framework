#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable local-notification values and a runtime-neutral static backend contract."]

extern crate alloc;

pub mod response;

use alloc::string::String;
use core::future::Future;
use framework_core::{AuthorizationState, Availability, Error, ErrorKind, PlatformErrorCode};

/// The greatest absolute Unix timestamp in milliseconds accepted by this contract.
///
/// The upper bound fits a signed 64-bit millisecond value. A backend may support a smaller date
/// range and report that limitation when asked to schedule a notification.
pub const MAX_UNIX_TIMESTAMP_MILLIS: u64 = i64::MAX as u64;

/// A caller-supplied local-notification identifier owned by the framework.
///
/// Identifiers are exact, case-sensitive UTF-8 values. Construction rejects an empty value and
/// NUL bytes; it performs no normalization, prefixing, truncation, or copy beyond taking the
/// supplied `String`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NotificationId(String);

impl NotificationId {
    /// Creates an identifier from an owned non-empty string that contains no NUL byte.
    pub fn new(value: String) -> Result<Self, NotificationError> {
        if value.is_empty() || value.as_bytes().contains(&0) {
            return Err(NotificationError::InvalidIdentifier);
        }
        Ok(Self(value))
    }

    /// Borrows the exact identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Transfers the owned identifier string to the caller.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Owned portable text fields for one user-visible notification.
///
/// The contract contains only a title and optional body. It does not model actions, categories,
/// attachments, sounds, badges, or platform-specific presentation options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationContent {
    title: String,
    body: Option<String>,
}

impl NotificationContent {
    /// Takes ownership of a title and optional body without normalization or an additional copy.
    pub const fn new(title: String, body: Option<String>) -> Self {
        Self { title, body }
    }

    /// Borrows the owned title text.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Borrows the optional owned body text.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

/// A bounded, one-shot local-notification trigger.
///
/// The only trigger forms are immediate delivery and an absolute Unix timestamp in milliseconds.
/// Repeating, calendar, interval, and relative-delay triggers are not represented.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NotificationTrigger(TriggerValue);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TriggerValue {
    Immediate,
    AtUnixMillis(u64),
}

impl NotificationTrigger {
    /// Creates a one-shot trigger eligible for delivery as soon as the backend permits.
    pub const fn immediate() -> Self {
        Self(TriggerValue::Immediate)
    }

    /// Creates a one-shot trigger for an absolute Unix timestamp in milliseconds.
    ///
    /// Timestamps range from zero through [`MAX_UNIX_TIMESTAMP_MILLIS`]. A timestamp already in
    /// the past is eligible as soon as the backend permits; precise delivery time is not promised.
    pub const fn at_unix_millis(timestamp: u64) -> Result<Self, NotificationError> {
        if timestamp <= MAX_UNIX_TIMESTAMP_MILLIS {
            Ok(Self(TriggerValue::AtUnixMillis(timestamp)))
        } else {
            Err(NotificationError::InvalidTimestamp)
        }
    }

    /// Reports whether this trigger requests immediate eligibility.
    pub const fn is_immediate(self) -> bool {
        matches!(self.0, TriggerValue::Immediate)
    }

    /// Returns the absolute Unix timestamp in milliseconds, or `None` for an immediate trigger.
    pub const fn unix_timestamp_millis(self) -> Option<u64> {
        match self.0 {
            TriggerValue::Immediate => None,
            TriggerValue::AtUnixMillis(timestamp) => Some(timestamp),
        }
    }
}

/// An owned local-notification request with a caller-selected replacement identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notification {
    identifier: NotificationId,
    content: NotificationContent,
    trigger: NotificationTrigger,
}

impl Notification {
    /// Takes ownership of an identifier, content, and one-shot trigger.
    ///
    /// ```
    /// extern crate alloc;
    /// use alloc::string::String;
    /// use framework_notifications::{
    ///     Notification, NotificationContent, NotificationId, NotificationTrigger,
    /// };
    ///
    /// # fn main() -> Result<(), framework_notifications::NotificationError> {
    /// let request = Notification::new(
    ///     NotificationId::new(String::from("daily-reminder"))?,
    ///     NotificationContent::new(
    ///         String::from("Reminder"),
    ///         Some(String::from("Review today's notes")),
    ///     ),
    ///     NotificationTrigger::immediate(),
    /// );
    /// # let _ = request;
    /// # Ok(())
    /// # }
    /// ```
    pub const fn new(
        identifier: NotificationId,
        content: NotificationContent,
        trigger: NotificationTrigger,
    ) -> Self {
        Self {
            identifier,
            content,
            trigger,
        }
    }

    /// Borrows the caller-supplied identifier.
    pub const fn identifier(&self) -> &NotificationId {
        &self.identifier
    }

    /// Borrows the owned notification content.
    pub const fn content(&self) -> &NotificationContent {
        &self.content
    }

    /// Returns the one-shot trigger.
    pub const fn trigger(&self) -> NotificationTrigger {
        self.trigger
    }
}

/// A stable local-notification error with an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NotificationError {
    /// An identifier is empty or contains a NUL byte.
    InvalidIdentifier,
    /// An absolute timestamp is outside the supported signed-millisecond range.
    InvalidTimestamp,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl NotificationError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidIdentifier | Self::InvalidTimestamp => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidIdentifier | Self::InvalidTimestamp => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for local-notification operations.
///
/// Each operation starts when the corresponding [`Notifications`] future is first polled. A
/// backend operation has exactly one terminal success or error result. A live future observes that
/// result once; a dropped future observes none and does not cause a second completion. Dropping a
/// caller-facing future drops interest in its result but does not cancel an operation already
/// started; a backend must keep any in-flight native callback state safe until its one terminal
/// outcome. Cancellation of a scheduled pending notification is the explicit
/// [`Notifications::cancel`] operation.
///
/// A successful schedule for an identifier replaces any pending notification with that same
/// identifier; it must not create a duplicate pending request. If operations from multiple clients
/// race for one identifier, the backend's serialization order determines which successful
/// schedule remains. Cancelling reports whether the backend observed a matching pending request at
/// its documented cancellation point. A backend whose native API separates lookup from removal
/// must document that race; the boolean does not imply an atomic compare-and-remove. Cancellation
/// does not promise to withdraw a notification already presented or delivered. This contract does
/// not promise delivery at an exact time or provide an executor, process-wide service, or
/// permission prompt by itself.
pub trait NotificationBackend {
    /// Reports whether local notifications are usable in the current context.
    fn availability(&self) -> Availability;

    /// The future type for a non-prompting authorization-state query.
    type AuthorizationFuture<'a>: Future<Output = Result<AuthorizationState, NotificationError>>
        + 'a
    where
        Self: 'a;

    /// Queries the normalized authorization state without requesting permission.
    fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a>;

    /// The future type for a permission-authorization request.
    type RequestAuthorizationFuture<'a>: Future<Output = Result<AuthorizationState, NotificationError>>
        + 'a
    where
        Self: 'a;

    /// Requests authorization and returns the resulting normalized state.
    ///
    /// A native backend may show a platform permission prompt when this operation is polled. This
    /// portable contract has no prompt or permission side effect on its own.
    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a>;

    /// The future type for scheduling one owned notification request.
    type ScheduleFuture<'a>: Future<Output = Result<(), NotificationError>> + 'a
    where
        Self: 'a;

    /// Schedules one owned notification, replacing a pending request with the same identifier.
    ///
    /// Success means the backend accepted the request, not that a user will see it. Backends must
    /// document any smaller date range or content restrictions than this portable contract.
    fn schedule<'a>(&'a mut self, notification: Notification) -> Self::ScheduleFuture<'a>;

    /// The future type for cancelling one pending notification by identifier.
    type CancelFuture<'a>: Future<Output = Result<bool, NotificationError>> + 'a
    where
        Self: 'a;

    /// Requests removal of a pending request and reports whether the backend observed one.
    ///
    /// The backend defines and documents its cancellation observation point. This result does not
    /// imply an atomic compare-and-remove when its native API separates lookup from removal.
    fn cancel<'a>(&'a mut self, identifier: NotificationId) -> Self::CancelFuture<'a>;
}

/// A thin facade over caller-owned, statically selected notification-backend state.
pub struct Notifications<B> {
    backend: B,
}

impl<B: NotificationBackend> Notifications<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Queries the backend's normalized authorization state without requesting permission.
    pub async fn authorization(&mut self) -> Result<AuthorizationState, NotificationError> {
        self.backend.authorization().await
    }

    /// Requests authorization through the selected backend and returns its normalized state.
    pub async fn request_authorization(&mut self) -> Result<AuthorizationState, NotificationError> {
        self.backend.request_authorization().await
    }

    /// Schedules one owned notification through the selected backend.
    ///
    /// The operation begins on first poll. Dropping its future does not cancel a started backend
    /// operation; use [`Self::cancel`] to request removal of a pending notification by identifier.
    pub async fn schedule(&mut self, notification: Notification) -> Result<(), NotificationError> {
        self.backend.schedule(notification).await
    }

    /// Requests cancellation and reports whether the backend observed a matching pending request.
    ///
    /// This is distinct from dropping a future returned by another operation. The selected
    /// backend documents its observation point and whether native lookup/removal can race. A
    /// backend cannot promise to withdraw a notification already presented or delivered.
    pub async fn cancel(&mut self, identifier: NotificationId) -> Result<bool, NotificationError> {
        self.backend.cancel(identifier).await
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
    use alloc::vec::Vec;
    use core::cell::RefCell;
    use core::future::{Future, Ready, ready};
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    struct Backend {
        authorization: AuthorizationState,
        request_result: AuthorizationState,
        request_count: u32,
        pending: Vec<Notification>,
        schedule_count: u32,
    }

    impl Backend {
        fn new() -> Self {
            Self {
                authorization: AuthorizationState::NotDetermined,
                request_result: AuthorizationState::Authorized,
                request_count: 0,
                pending: Vec::new(),
                schedule_count: 0,
            }
        }
    }

    impl NotificationBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type AuthorizationFuture<'a>
            = Ready<Result<AuthorizationState, NotificationError>>
        where
            Self: 'a;

        fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
            ready(Ok(self.authorization))
        }

        type RequestAuthorizationFuture<'a>
            = Ready<Result<AuthorizationState, NotificationError>>
        where
            Self: 'a;

        fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
            self.request_count += 1;
            self.authorization = self.request_result;
            ready(Ok(self.authorization))
        }

        type ScheduleFuture<'a>
            = Ready<Result<(), NotificationError>>
        where
            Self: 'a;

        fn schedule<'a>(&'a mut self, notification: Notification) -> Self::ScheduleFuture<'a> {
            self.schedule_count += 1;
            if let Some(index) = self
                .pending
                .iter()
                .position(|pending| pending.identifier() == notification.identifier())
            {
                self.pending[index] = notification;
            } else {
                self.pending.push(notification);
            }
            ready(Ok(()))
        }

        type CancelFuture<'a>
            = Ready<Result<bool, NotificationError>>
        where
            Self: 'a;

        fn cancel<'a>(&'a mut self, identifier: NotificationId) -> Self::CancelFuture<'a> {
            let removed = self
                .pending
                .iter()
                .position(|pending| pending.identifier() == &identifier)
                .map(|index| self.pending.remove(index))
                .is_some();
            ready(Ok(removed))
        }
    }

    struct PendingState {
        schedule_calls: u32,
        work_started: u32,
        cancellation_calls: u32,
        dropped_interest: u32,
        completion_count: u32,
        delivered_count: u32,
        callback_attached: bool,
        completed: bool,
    }

    impl PendingState {
        fn new() -> Self {
            Self {
                schedule_calls: 0,
                work_started: 0,
                cancellation_calls: 0,
                dropped_interest: 0,
                completion_count: 0,
                delivered_count: 0,
                callback_attached: false,
                completed: false,
            }
        }
    }

    struct PendingBackend {
        state: Rc<RefCell<PendingState>>,
    }

    impl PendingBackend {
        fn new() -> Self {
            Self {
                state: Rc::new(RefCell::new(PendingState::new())),
            }
        }

        fn callback(&self) -> PendingCallback {
            PendingCallback(self.state.clone())
        }
    }

    struct PendingCallback(Rc<RefCell<PendingState>>);

    impl PendingCallback {
        fn complete(&self) -> bool {
            let mut state = self.0.borrow_mut();
            if !state.callback_attached || state.completed {
                return false;
            }
            state.callback_attached = false;
            state.completed = true;
            state.completion_count += 1;
            true
        }
    }

    struct PendingScheduleFuture {
        state: Rc<RefCell<PendingState>>,
        started: bool,
        completed: bool,
    }

    impl Future for PendingScheduleFuture {
        type Output = Result<(), NotificationError>;

        fn poll(self: core::pin::Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                this.started = true;
                let mut state = this.state.borrow_mut();
                state.work_started += 1;
                state.callback_attached = true;
            }
            let mut state = this.state.borrow_mut();
            if state.completed {
                this.completed = true;
                state.delivered_count += 1;
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }

    impl Drop for PendingScheduleFuture {
        fn drop(&mut self) {
            if self.started && !self.completed {
                self.state.borrow_mut().dropped_interest += 1;
            }
        }
    }

    impl NotificationBackend for PendingBackend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type AuthorizationFuture<'a>
            = Ready<Result<AuthorizationState, NotificationError>>
        where
            Self: 'a;

        fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
            ready(Ok(AuthorizationState::NotDetermined))
        }

        type RequestAuthorizationFuture<'a>
            = Ready<Result<AuthorizationState, NotificationError>>
        where
            Self: 'a;

        fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
            ready(Ok(AuthorizationState::NotDetermined))
        }

        type ScheduleFuture<'a>
            = PendingScheduleFuture
        where
            Self: 'a;

        fn schedule<'a>(&'a mut self, _notification: Notification) -> Self::ScheduleFuture<'a> {
            self.state.borrow_mut().schedule_calls += 1;
            PendingScheduleFuture {
                state: self.state.clone(),
                started: false,
                completed: false,
            }
        }

        type CancelFuture<'a>
            = Ready<Result<bool, NotificationError>>
        where
            Self: 'a;

        fn cancel<'a>(&'a mut self, _identifier: NotificationId) -> Self::CancelFuture<'a> {
            self.state.borrow_mut().cancellation_calls += 1;
            ready(Ok(false))
        }
    }

    fn run_ready<F: Future>(future: F) -> F::Output {
        let mut future = pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("ready fake-backend operation returned pending"),
        }
    }

    fn id(value: &str) -> NotificationId {
        NotificationId::new(String::from(value)).unwrap()
    }

    fn notification(identifier: &str, title: &str) -> Notification {
        Notification::new(
            id(identifier),
            NotificationContent::new(String::from(title), None),
            NotificationTrigger::immediate(),
        )
    }

    #[test]
    fn identifiers_reject_empty_and_nul_values_without_normalization() {
        assert_eq!(
            NotificationId::new(String::new()).err(),
            Some(NotificationError::InvalidIdentifier)
        );
        assert_eq!(
            NotificationId::new(String::from("with\0nul")).err(),
            Some(NotificationError::InvalidIdentifier)
        );
        let identifier = NotificationId::new(String::from("Exact-ID_1")).unwrap();
        assert_eq!(identifier.as_str(), "Exact-ID_1");
        assert_eq!(identifier.into_string(), "Exact-ID_1");
    }

    #[test]
    fn trigger_forms_and_timestamp_bounds_are_explicit() {
        let immediate = NotificationTrigger::immediate();
        assert!(immediate.is_immediate());
        assert_eq!(immediate.unix_timestamp_millis(), None);

        let epoch = NotificationTrigger::at_unix_millis(0).unwrap();
        assert!(!epoch.is_immediate());
        assert_eq!(epoch.unix_timestamp_millis(), Some(0));

        let maximum = NotificationTrigger::at_unix_millis(MAX_UNIX_TIMESTAMP_MILLIS).unwrap();
        assert_eq!(
            maximum.unix_timestamp_millis(),
            Some(MAX_UNIX_TIMESTAMP_MILLIS)
        );
        assert_eq!(
            NotificationTrigger::at_unix_millis(MAX_UNIX_TIMESTAMP_MILLIS + 1),
            Err(NotificationError::InvalidTimestamp)
        );
    }

    #[test]
    fn authorization_query_and_request_keep_normalized_states() {
        let states = [
            AuthorizationState::Unknown,
            AuthorizationState::NotDetermined,
            AuthorizationState::Denied,
            AuthorizationState::Restricted,
            AuthorizationState::Authorized,
        ];
        let mut backend = Backend::new();
        let mut notifications = Notifications::new(backend);
        assert_eq!(notifications.availability(), Availability::Available);

        for state in states {
            notifications.backend_mut().authorization = state;
            assert_eq!(run_ready(notifications.authorization()), Ok(state));
        }

        assert_eq!(
            run_ready(notifications.request_authorization()),
            Ok(AuthorizationState::Authorized)
        );
        backend = notifications.into_backend();
        assert_eq!(backend.request_count, 1);
        assert_eq!(backend.authorization, AuthorizationState::Authorized);
    }

    #[test]
    fn fake_backend_replaces_duplicate_ids_and_cancel_reports_pending_state() {
        let mut notifications = Notifications::new(Backend::new());
        assert_eq!(
            run_ready(notifications.schedule(notification("reminder", "old"))),
            Ok(())
        );
        assert_eq!(
            run_ready(notifications.schedule(notification("reminder", "new"))),
            Ok(())
        );
        assert_eq!(notifications.backend().pending.len(), 1);
        assert_eq!(notifications.backend().pending[0].content().title(), "new");
        assert_eq!(notifications.backend().schedule_count, 2);
        assert_eq!(run_ready(notifications.cancel(id("reminder"))), Ok(true));
        assert_eq!(run_ready(notifications.cancel(id("reminder"))), Ok(false));
        assert!(notifications.backend().pending.is_empty());
    }

    #[test]
    fn dropping_unpolled_schedule_future_does_not_start_backend_work() {
        let mut notifications = Notifications::new(Backend::new());
        drop(notifications.schedule(notification("not-started", "title")));
        assert_eq!(notifications.backend().schedule_count, 0);
        assert!(notifications.backend().pending.is_empty());
    }

    #[test]
    fn dropping_started_schedule_future_drops_interest_without_cancelling_work() {
        let backend = PendingBackend::new();
        let callback = backend.callback();
        let state = backend.state.clone();
        let mut notifications = Notifications::new(backend);
        {
            let mut future = pin!(notifications.schedule(notification("started", "title")));
            let mut context = Context::from_waker(Waker::noop());
            {
                let state = state.borrow();
                assert_eq!(state.schedule_calls, 0);
                assert_eq!(state.work_started, 0);
            }
            assert!(future.as_mut().poll(&mut context).is_pending());
        }

        {
            let state = state.borrow();
            assert_eq!(state.schedule_calls, 1);
            assert_eq!(state.work_started, 1);
            assert_eq!(state.dropped_interest, 1);
            assert_eq!(state.cancellation_calls, 0);
            assert!(state.callback_attached);
            assert_eq!(state.completion_count, 0);
            assert_eq!(state.delivered_count, 0);
        }
        assert!(callback.complete());
        assert!(!callback.complete());
        let state = state.borrow();
        assert!(!state.callback_attached);
        assert_eq!(state.completion_count, 1);
        assert_eq!(state.delivered_count, 0);
        assert_eq!(state.cancellation_calls, 0);
    }
}
