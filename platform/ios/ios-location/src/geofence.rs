use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use framework_location::{Coordinate, LocationAuthorization, LocationError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Maximum number of geofence events retained before the oldest is evicted.
pub(crate) const GEOFENCE_EVENT_QUEUE_CAPACITY: usize = 16;
static MONITOR_RESERVED: AtomicBool = AtomicBool::new(false);

pub(crate) struct MonitorLease;

impl MonitorLease {
    pub(crate) fn acquire() -> Result<Self, IosGeofenceError> {
        MONITOR_RESERVED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| IosGeofenceError::MonitorAlreadyOpen)
    }
}

impl Drop for MonitorLease {
    fn drop(&mut self) {
        MONITOR_RESERVED.store(false, Ordering::Release);
    }
}

/// A validated circular WGS-84 geofence owned by the caller.
#[derive(Clone, Debug, PartialEq)]
pub struct IosGeofence {
    identifier: String,
    center: Coordinate,
    radius_meters: f64,
}

impl IosGeofence {
    /// Creates a circular geofence after validating its identifier, coordinate, and positive radius.
    pub fn new(
        identifier: impl Into<String>,
        latitude_degrees: f64,
        longitude_degrees: f64,
        radius_meters: f64,
    ) -> Result<Self, IosGeofenceError> {
        let identifier = identifier.into();
        if identifier.is_empty() {
            return Err(IosGeofenceError::InvalidIdentifier);
        }
        let center = Coordinate::new(latitude_degrees, longitude_degrees)
            .map_err(|_| IosGeofenceError::InvalidCoordinate)?;
        if !radius_meters.is_finite() || radius_meters <= 0.0 {
            return Err(IosGeofenceError::InvalidRadius);
        }
        Ok(Self {
            identifier,
            center,
            radius_meters,
        })
    }

    /// Returns the stable caller-supplied identifier.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Returns the validated WGS-84 center coordinate.
    pub const fn center(&self) -> Coordinate {
        self.center
    }

    /// Returns the positive circular radius in meters.
    pub const fn radius_meters(&self) -> f64 {
        self.radius_meters
    }
}

/// A typed configuration, permission, or native-monitor failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosGeofenceError {
    /// The persistent monitor name is empty or violates Core Location's name rules.
    InvalidMonitorName,
    /// The geofence identifier is empty.
    InvalidIdentifier,
    /// Latitude or longitude is non-finite or outside its WGS-84 range.
    InvalidCoordinate,
    /// Radius is non-finite or not positive.
    InvalidRadius,
    /// Always location authorization is not already granted; no prompt was requested.
    PermissionDenied,
    /// Required nonempty Always-authorization usage descriptions are missing from the app bundle.
    InvalidConfiguration,
    /// The current operating system does not provide `CLMonitor`.
    Unsupported,
    /// This process already has a pending or open geofence monitor.
    MonitorAlreadyOpen,
    /// The native open callback failed or returned an incompatible persisted condition.
    NativeFailure,
    /// The monitor event callback panicked and its interest was detached.
    CallbackFailure,
}

impl From<LocationError> for IosGeofenceError {
    fn from(value: LocationError) -> Self {
        match value {
            LocationError::InvalidCoordinate => Self::InvalidCoordinate,
            _ => Self::NativeFailure,
        }
    }
}

/// The state Core Location reports for a circular geofence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosGeofenceState {
    /// Core Location has not established a state yet or returned an unknown value.
    Unknown,
    /// The device is inside the monitored condition.
    Satisfied,
    /// The device is outside the monitored condition.
    Unsatisfied,
    /// Core Location cannot currently monitor the condition.
    Unmonitored,
}

/// A typed authorization, support, persistence, or condition-limit event issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosGeofenceIssue {
    /// Rust could not safely copy a native callback; event delivery remains active.
    CallbackFailure,
    /// App location authorization is denied.
    AuthorizationDenied,
    /// Location authorization is restricted.
    AuthorizationRestricted,
    /// Location Services are disabled globally.
    AuthorizationDeniedGlobally,
    /// The app is not sufficiently in use for event delivery.
    InsufficientlyInUse,
    /// Reduced accuracy prevents event delivery.
    AccuracyLimited,
    /// The condition type is not supported for monitoring.
    ConditionUnsupported,
    /// Core Location's condition-count limit is exceeded.
    ConditionLimitExceeded,
    /// Core Location cannot persist the condition or its event state.
    PersistenceUnavailable,
    /// An explicit location service session is required by app configuration.
    ServiceSessionRequired,
    /// An authorization request is currently in progress.
    AuthorizationRequestInProgress,
}

/// One copied Core Location event with typed state and issue data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosGeofenceEvent {
    identifier: Option<String>,
    state: IosGeofenceState,
    timestamp_unix_millis: Option<u64>,
    issues: Vec<IosGeofenceIssue>,
    missed_before: u64,
}

impl IosGeofenceEvent {
    pub(crate) fn new(
        identifier: Option<String>,
        state: IosGeofenceState,
        timestamp_unix_millis: Option<u64>,
        issues: Vec<IosGeofenceIssue>,
    ) -> Self {
        Self {
            identifier,
            state,
            timestamp_unix_millis,
            issues,
            missed_before: 0,
        }
    }

    /// Returns the region identifier, or `None` for a monitor-level callback failure.
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    /// Returns the copied Core Location condition state.
    pub const fn state(&self) -> IosGeofenceState {
        self.state
    }

    /// Returns the event time as Unix milliseconds when representable.
    pub const fn timestamp_unix_millis(&self) -> Option<u64> {
        self.timestamp_unix_millis
    }

    /// Returns all issue flags reported by Core Location for this event.
    pub fn issues(&self) -> &[IosGeofenceIssue] {
        &self.issues
    }

    /// Returns the number of earlier events evicted before this retained event was delivered.
    pub const fn missed_before(&self) -> u64 {
        self.missed_before
    }

    pub(crate) fn callback_failure() -> Self {
        Self::new(
            None,
            IosGeofenceState::Unknown,
            None,
            vec![IosGeofenceIssue::CallbackFailure],
        )
    }
}

pub(crate) const fn geofence_authorization_preflight(
    authorization: LocationAuthorization,
) -> Result<(), IosGeofenceError> {
    if matches!(authorization, LocationAuthorization::Background) {
        Ok(())
    } else {
        Err(IosGeofenceError::PermissionDenied)
    }
}

pub(crate) const fn geofence_name_is_valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes[0] == b'_' {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        let value = bytes[index];
        if !value.is_ascii_alphanumeric() {
            return false;
        }
        index += 1;
    }
    true
}

pub(crate) const fn geofence_state_from_native(value: usize) -> IosGeofenceState {
    match value {
        1 => IosGeofenceState::Satisfied,
        2 => IosGeofenceState::Unsatisfied,
        3 => IosGeofenceState::Unmonitored,
        _ => IosGeofenceState::Unknown,
    }
}

pub(crate) fn geofence_issues_from_flags(flags: [bool; 10]) -> Vec<IosGeofenceIssue> {
    let mut issues = Vec::new();
    let mappings = [
        IosGeofenceIssue::AuthorizationDenied,
        IosGeofenceIssue::AuthorizationRestricted,
        IosGeofenceIssue::AuthorizationDeniedGlobally,
        IosGeofenceIssue::InsufficientlyInUse,
        IosGeofenceIssue::AccuracyLimited,
        IosGeofenceIssue::ConditionUnsupported,
        IosGeofenceIssue::ConditionLimitExceeded,
        IosGeofenceIssue::PersistenceUnavailable,
        IosGeofenceIssue::ServiceSessionRequired,
        IosGeofenceIssue::AuthorizationRequestInProgress,
    ];
    for (enabled, issue) in flags.into_iter().zip(mappings) {
        if enabled {
            issues.push(issue);
        }
    }
    issues
}

/// A bounded, cancellation-safe queue for native monitor callbacks.
pub(crate) struct GeofenceEventQueue {
    state: Mutex<EventQueueState>,
}

struct EventQueueState {
    events: [Option<IosGeofenceEvent>; GEOFENCE_EVENT_QUEUE_CAPACITY],
    head: usize,
    len: usize,
    evicted: u64,
    closed: bool,
    waker: Option<Waker>,
}

impl GeofenceEventQueue {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(EventQueueState {
                events: core::array::from_fn(|_| None),
                head: 0,
                len: 0,
                evicted: 0,
                closed: false,
                waker: None,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn push(&self, event: IosGeofenceEvent) {
        wake(self.push_deferred(event));
    }

    pub(crate) fn push_deferred(&self, event: IosGeofenceEvent) -> Option<Waker> {
        {
            let mut state = self.lock();
            if state.closed {
                return None;
            }
            if state.len == GEOFENCE_EVENT_QUEUE_CAPACITY {
                let head = state.head;
                state.events[head] = None;
                state.head = (head + 1) % GEOFENCE_EVENT_QUEUE_CAPACITY;
                state.len -= 1;
                state.evicted = state.evicted.saturating_add(1);
            }
            let index = (state.head + state.len) % GEOFENCE_EVENT_QUEUE_CAPACITY;
            state.events[index] = Some(event);
            state.len += 1;
            state.waker.take()
        }
    }

    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Option<IosGeofenceEvent>> {
        let mut new_waker = None;
        loop {
            let mut state = self.lock();
            if state.len != 0 {
                let head = state.head;
                let Some(mut event) = state.events[head].take() else {
                    state.len = 0;
                    state.head = 0;
                    return Poll::Ready(None);
                };
                state.head = (head + 1) % GEOFENCE_EVENT_QUEUE_CAPACITY;
                state.len -= 1;
                event.missed_before = core::mem::replace(&mut state.evicted, 0);
                drop(state);
                drop(new_waker);
                return Poll::Ready(Some(event));
            }
            if state.closed {
                drop(state);
                drop(new_waker);
                return Poll::Ready(None);
            }
            if state
                .waker
                .as_ref()
                .is_some_and(|waker| waker.will_wake(context.waker()))
            {
                drop(state);
                drop(new_waker);
                return Poll::Pending;
            }
            if let Some(new_waker) = new_waker.take() {
                let previous = state.waker.replace(new_waker);
                drop(state);
                drop(previous);
                return Poll::Pending;
            }
            drop(state);
            new_waker = Some(context.waker().clone());
        }
    }

    pub(crate) fn clear_waker(&self, waker: &Waker) {
        let previous = {
            let mut state = self.lock();
            if state
                .waker
                .as_ref()
                .is_some_and(|registered| registered.will_wake(waker))
            {
                state.waker.take()
            } else {
                None
            }
        };
        drop(previous);
    }

    pub(crate) fn detach(&self) {
        let waker = {
            let mut state = self.lock();
            state.closed = true;
            state.events.fill(None);
            state.head = 0;
            state.len = 0;
            state.waker.take()
        };
        drop(waker);
    }

    fn lock(&self) -> MutexGuard<'_, EventQueueState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

fn wake(waker: Option<Waker>) {
    if let Some(waker) = waker {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
    }
}

pub(crate) struct OpenCompletionCell {
    state: Mutex<OpenState>,
}

struct OpenState {
    completed: bool,
    detached: bool,
    result: Option<Result<(), IosGeofenceError>>,
    waker: Option<Waker>,
}

impl OpenCompletionCell {
    pub(crate) const fn new() -> Self {
        Self {
            state: Mutex::new(OpenState {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    pub(crate) fn complete_with(
        &self,
        result: Result<(), IosGeofenceError>,
        accept: impl FnOnce(),
    ) -> bool {
        let (accepted, waker) = self.complete_with_deferred(result, accept);
        wake(waker);
        accepted
    }

    pub(crate) fn complete_with_deferred(
        &self,
        result: Result<(), IosGeofenceError>,
        accept: impl FnOnce(),
    ) -> (bool, Option<Waker>) {
        let waker = {
            let mut state = self.lock();
            if state.detached || state.completed {
                return (false, None);
            }
            if result.is_ok() {
                accept();
            }
            state.completed = true;
            state.result = Some(result);
            state.waker.take()
        };
        (true, waker)
    }

    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Result<(), IosGeofenceError>> {
        let mut new_waker = None;
        loop {
            let mut state = self.lock();
            if let Some(result) = state.result.take() {
                drop(state);
                drop(new_waker);
                return Poll::Ready(result);
            }
            if state.detached || state.completed {
                drop(state);
                drop(new_waker);
                return Poll::Ready(Err(IosGeofenceError::NativeFailure));
            }
            if state
                .waker
                .as_ref()
                .is_some_and(|waker| waker.will_wake(context.waker()))
            {
                drop(state);
                drop(new_waker);
                return Poll::Pending;
            }
            if let Some(new_waker) = new_waker.take() {
                let previous = state.waker.replace(new_waker);
                drop(state);
                drop(previous);
                return Poll::Pending;
            }
            drop(state);
            new_waker = Some(context.waker().clone());
        }
    }

    pub(crate) fn detach(&self) {
        let waker = {
            let mut state = self.lock();
            state.detached = true;
            state.result = None;
            state.waker.take()
        };
        drop(waker);
    }

    fn lock(&self) -> MutexGuard<'_, OpenState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// An awaitable read of the next geofence event.
pub struct GeofenceEventFuture<'a> {
    queue: &'a GeofenceEventQueue,
    completed: bool,
    waiter: Option<Waker>,
}

impl<'a> GeofenceEventFuture<'a> {
    pub(crate) fn new(queue: &'a GeofenceEventQueue) -> Self {
        Self {
            queue,
            completed: false,
            waiter: None,
        }
    }
}

impl Future for GeofenceEventFuture<'_> {
    type Output = Option<IosGeofenceEvent>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.completed {
            return Poll::Ready(None);
        }
        match this.queue.poll(context) {
            Poll::Ready(event) => {
                this.completed = true;
                this.waiter = None;
                Poll::Ready(event)
            }
            Poll::Pending => {
                if this
                    .waiter
                    .as_ref()
                    .is_none_or(|waker| !waker.will_wake(context.waker()))
                {
                    this.waiter = Some(context.waker().clone());
                }
                Poll::Pending
            }
        }
    }
}

impl Drop for GeofenceEventFuture<'_> {
    fn drop(&mut self) {
        if !self.completed {
            if let Some(waker) = self.waiter.take() {
                self.queue.clear_waker(&waker);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};

    struct WakeCount(AtomicUsize);

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn event(id: &str, raw_state: usize) -> IosGeofenceEvent {
        IosGeofenceEvent::new(
            Some(id.to_owned()),
            geofence_state_from_native(raw_state),
            None,
            Vec::new(),
        )
    }

    fn poll(queue: &GeofenceEventQueue) -> Poll<Option<IosGeofenceEvent>> {
        queue.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn geofence_validates_each_negative_input() {
        assert_eq!(
            IosGeofence::new("", 0.0, 0.0, 1.0),
            Err(IosGeofenceError::InvalidIdentifier)
        );
        assert_eq!(
            IosGeofence::new("id", f64::NAN, 0.0, 1.0),
            Err(IosGeofenceError::InvalidCoordinate)
        );
        assert_eq!(
            IosGeofence::new("id", 91.0, 0.0, 1.0),
            Err(IosGeofenceError::InvalidCoordinate)
        );
        assert_eq!(
            IosGeofence::new("id", 0.0, f64::INFINITY, 1.0),
            Err(IosGeofenceError::InvalidCoordinate)
        );
        assert_eq!(
            IosGeofence::new("id", 0.0, 0.0, 0.0),
            Err(IosGeofenceError::InvalidRadius)
        );
        assert_eq!(
            IosGeofence::new("id", 0.0, 0.0, f64::INFINITY),
            Err(IosGeofenceError::InvalidRadius)
        );
    }

    #[test]
    fn monitor_name_rules_reject_empty_underscore_and_non_alphanumeric_values() {
        assert!(!geofence_name_is_valid(""));
        assert!(!geofence_name_is_valid("_private"));
        assert!(!geofence_name_is_valid("a-b"));
        assert!(!geofence_name_is_valid("é"));
        assert!(geofence_name_is_valid("Route42"));
    }

    #[test]
    fn authorization_preflight_requires_already_granted_always_without_prompting() {
        for authorization in [
            LocationAuthorization::NotDetermined,
            LocationAuthorization::Restricted,
            LocationAuthorization::Denied,
            LocationAuthorization::Foreground,
            LocationAuthorization::Unknown,
        ] {
            assert_eq!(
                geofence_authorization_preflight(authorization),
                Err(IosGeofenceError::PermissionDenied)
            );
        }
        assert_eq!(
            geofence_authorization_preflight(LocationAuthorization::Background),
            Ok(())
        );
    }

    #[test]
    fn native_state_mapping_preserves_known_states_and_unknown_values() {
        assert_eq!(geofence_state_from_native(0), IosGeofenceState::Unknown);
        assert_eq!(geofence_state_from_native(1), IosGeofenceState::Satisfied);
        assert_eq!(geofence_state_from_native(2), IosGeofenceState::Unsatisfied);
        assert_eq!(geofence_state_from_native(3), IosGeofenceState::Unmonitored);
        assert_eq!(geofence_state_from_native(99), IosGeofenceState::Unknown);
    }

    #[test]
    fn event_queue_preserves_order_and_reports_evicted_count_on_delivery() {
        let queue = GeofenceEventQueue::new();
        for index in 0..=GEOFENCE_EVENT_QUEUE_CAPACITY {
            queue.push(event(&index.to_string(), index % 3));
        }
        let Poll::Ready(Some(first)) = poll(&queue) else {
            panic!("retained geofence event was not ready");
        };
        assert_eq!(first.identifier(), Some("1"));
        assert_eq!(first.missed_before(), 1);
        for index in 2..=GEOFENCE_EVENT_QUEUE_CAPACITY {
            let Poll::Ready(Some(next)) = poll(&queue) else {
                panic!("queued geofence event was not ready");
            };
            assert_eq!(next.identifier(), Some(index.to_string().as_str()));
            assert_eq!(next.missed_before(), 0);
        }
    }

    #[test]
    fn event_wait_cancel_and_late_open_callback_are_detached_safely() {
        let queue = GeofenceEventQueue::new();
        let wakes = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wakes.clone());
        let mut context = Context::from_waker(&waker);
        let mut wait = Box::pin(GeofenceEventFuture::new(&queue));
        assert!(wait.as_mut().poll(&mut context).is_pending());
        drop(wait);
        queue.push(event("late", 1));
        assert_eq!(wakes.0.load(Ordering::SeqCst), 0);
        assert!(matches!(poll(&queue), Poll::Ready(Some(_))));

        let open = OpenCompletionCell::new();
        assert!(open.poll(&mut context).is_pending());
        open.detach();
        assert!(!open.complete_with(Ok(()), || panic!(
            "detached callback must not acquire monitor"
        )));
        assert_eq!(wakes.0.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn deferred_native_completion_wake_does_not_run_inline_and_monitor_lease_reopens() {
        let open = OpenCompletionCell::new();
        let wakes = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wakes.clone());
        let mut context = Context::from_waker(&waker);
        assert!(open.poll(&mut context).is_pending());
        let (accepted, deferred) = open.complete_with_deferred(Ok(()), || {});
        assert!(accepted);
        assert_eq!(wakes.0.load(Ordering::SeqCst), 0);
        deferred.expect("pending opener has a waker").wake();
        assert_eq!(wakes.0.load(Ordering::SeqCst), 1);

        let mut opener = Some(MonitorLease::acquire().expect("first singleton lease"));
        assert_eq!(
            MonitorLease::acquire().err(),
            Some(IosGeofenceError::MonitorAlreadyOpen)
        );
        let session_lease = opener.take().expect("completed opener transfers its lease");
        assert!(opener.is_none());
        assert_eq!(
            MonitorLease::acquire().err(),
            Some(IosGeofenceError::MonitorAlreadyOpen)
        );
        drop(session_lease);
        assert!(MonitorLease::acquire().is_ok());
    }

    #[test]
    fn callback_copy_failure_is_separate_from_eviction_count() {
        let queue = GeofenceEventQueue::new();
        queue.push(IosGeofenceEvent::callback_failure());
        queue.push(event("region", 1));
        let Poll::Ready(Some(failure)) = poll(&queue) else {
            panic!("callback-copy failure event was not ready");
        };
        assert_eq!(failure.identifier(), None);
        assert_eq!(failure.issues(), &[IosGeofenceIssue::CallbackFailure]);
        assert_eq!(failure.missed_before(), 0);
        let Poll::Ready(Some(region)) = poll(&queue) else {
            panic!("region event was not ready");
        };
        assert_eq!(region.identifier(), Some("region"));
        assert_eq!(region.missed_before(), 0);
    }

    #[test]
    fn native_issue_flags_map_to_typed_authorization_support_and_persistence_issues() {
        let issues = geofence_issues_from_flags([
            true, false, true, false, true, true, false, true, false, false,
        ]);
        assert_eq!(
            issues,
            [
                IosGeofenceIssue::AuthorizationDenied,
                IosGeofenceIssue::AuthorizationDeniedGlobally,
                IosGeofenceIssue::AccuracyLimited,
                IosGeofenceIssue::ConditionUnsupported,
                IosGeofenceIssue::PersistenceUnavailable,
            ]
        );
        let failure = IosGeofenceEvent::callback_failure();
        assert_eq!(failure.identifier(), None);
        assert_eq!(failure.issues(), &[IosGeofenceIssue::CallbackFailure]);
    }
}
