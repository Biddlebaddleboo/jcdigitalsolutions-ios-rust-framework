#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable one-shot accelerometer values and a static backend contract."]

use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

#[cfg(test)]
extern crate std;

/// A finite acceleration vector in meters per second squared.
///
/// The x, y, and z components use the backend's device-fixed sensor axes. Values are raw
/// accelerometer readings including gravity; they are not rotated to screen or world coordinates
/// and are not gravity-removed user acceleration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Acceleration {
    x_meters_per_second_squared: f64,
    y_meters_per_second_squared: f64,
    z_meters_per_second_squared: f64,
}

impl Acceleration {
    /// Creates an acceleration vector when every component is finite.
    pub fn new(
        x_meters_per_second_squared: f64,
        y_meters_per_second_squared: f64,
        z_meters_per_second_squared: f64,
    ) -> Result<Self, MotionError> {
        if !x_meters_per_second_squared.is_finite()
            || !y_meters_per_second_squared.is_finite()
            || !z_meters_per_second_squared.is_finite()
        {
            return Err(MotionError::InvalidAcceleration);
        }
        Ok(Self {
            x_meters_per_second_squared,
            y_meters_per_second_squared,
            z_meters_per_second_squared,
        })
    }

    /// Returns the x component in meters per second squared.
    pub const fn x_meters_per_second_squared(self) -> f64 {
        self.x_meters_per_second_squared
    }

    /// Returns the y component in meters per second squared.
    pub const fn y_meters_per_second_squared(self) -> f64 {
        self.y_meters_per_second_squared
    }

    /// Returns the z component in meters per second squared.
    pub const fn z_meters_per_second_squared(self) -> f64 {
        self.z_meters_per_second_squared
    }
}

/// A measurement time in monotonic nanoseconds.
///
/// This is not Unix time or callback/return time. Values are comparable only within the same
/// backend clock domain and device boot or session; the value does not encode that domain.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MotionTimestamp(u64);

impl MotionTimestamp {
    /// Creates a timestamp from monotonic nanoseconds in the backend's clock domain.
    pub const fn from_monotonic_nanos(nanoseconds: u64) -> Self {
        Self(nanoseconds)
    }

    /// Returns the measurement time in monotonic nanoseconds.
    pub const fn monotonic_nanos(self) -> u64 {
        self.0
    }
}

/// One owned accelerometer measurement and its measurement timestamp.
///
/// The value contains only scalar data, has no heap allocation or borrowed native object, and is
/// returned by value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccelerationSample {
    acceleration: Acceleration,
    timestamp: MotionTimestamp,
}

impl AccelerationSample {
    /// Creates a sample from a validated acceleration and its measurement timestamp.
    pub const fn new(acceleration: Acceleration, timestamp: MotionTimestamp) -> Self {
        Self {
            acceleration,
            timestamp,
        }
    }

    /// Returns the acceleration vector by value.
    pub const fn acceleration(self) -> Acceleration {
        self.acceleration
    }

    /// Returns the time at which the backend measured this sample.
    pub const fn timestamp(self) -> MotionTimestamp {
        self.timestamp
    }
}

/// A stable motion error that preserves an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MotionError {
    /// An acceleration component is NaN or positive or negative infinity.
    InvalidAcceleration,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl MotionError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidAcceleration => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidAcceleration => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for one-shot raw accelerometer requests.
///
/// A request starts only when its returned future is first polled; creating the future starts no
/// work. Dropping a pending future first detaches or invalidates its callback context, then asks
/// the native operation to cancel when cancellation is available. If cancellation is unavailable
/// or races with drop, a late callback may observe detached callback state but must not access
/// freed future memory. Keep callback state alive, or use an equivalent stable token, until no
/// native callback can access it; release that state exactly once. If completion races with drop,
/// the operation yields one result only when completion wins and the future remains live; if drop
/// wins, suppress the result. Ignore duplicate and late callbacks. No executor or `Send` bound is
/// imposed.
pub trait MotionBackend {
    /// Reports non-prompting backend availability for raw accelerometer access.
    ///
    /// This signal does not guarantee that an exclusive native update channel is free or that a
    /// request will succeed.
    fn availability(&self) -> Availability;

    /// The future type for one owned accelerometer sample.
    type CurrentAccelerationFuture<'a>: Future<Output = Result<AccelerationSample, MotionError>>
        + 'a
    where
        Self: 'a;

    /// Requests one current accelerometer sample.
    ///
    /// The returned future must defer starting work until its first poll. It must return one raw
    /// device-axis sample including gravity, or one error, and must follow the cancellation and
    /// callback-state rules documented on [`MotionBackend`].
    fn current_acceleration<'a>(&'a mut self) -> Self::CurrentAccelerationFuture<'a>;
}

/// A thin facade over caller-owned, statically selected motion-backend state.
pub struct Motion<B> {
    backend: B,
}

impl<B: MotionBackend> Motion<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Requests one owned raw accelerometer sample.
    ///
    /// The operation starts on first poll. Dropping this future before its first poll starts no
    /// work. Dropping it while pending detaches or invalidates callback state before requesting
    /// native cancellation when available; late results are suppressed safely. The returned
    /// sample has no freshness or latency guarantee, and its timestamp is measurement time rather
    /// than callback or return time.
    pub async fn current_acceleration(&mut self) -> Result<AccelerationSample, MotionError> {
        self.backend.current_acceleration().await
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
    use core::cell::RefCell;
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    use std::rc::Rc;
    use std::vec::Vec;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        Started,
        Detached,
        CancellationRequested,
        ResultDelivered,
    }

    struct FakeState {
        request_count: u32,
        start_count: u32,
        callback_count: u32,
        late_callback_count: u32,
        delivered_count: u32,
        detach_count: u32,
        cancellation_count: u32,
        callback_attached: bool,
        callback_claimed: bool,
        result: Option<Result<AccelerationSample, MotionError>>,
        events: Vec<Event>,
    }

    impl FakeState {
        fn new() -> Self {
            Self {
                request_count: 0,
                start_count: 0,
                callback_count: 0,
                late_callback_count: 0,
                delivered_count: 0,
                detach_count: 0,
                cancellation_count: 0,
                callback_attached: false,
                callback_claimed: false,
                result: None,
                events: Vec::new(),
            }
        }
    }

    #[derive(Clone)]
    struct FakeCallback(Rc<RefCell<FakeState>>);

    impl FakeCallback {
        fn complete(&self, result: Result<AccelerationSample, MotionError>) -> bool {
            let mut state = self.0.borrow_mut();
            if !state.callback_attached || state.callback_claimed {
                state.late_callback_count += 1;
                return false;
            }
            state.callback_claimed = true;
            state.callback_attached = false;
            state.callback_count += 1;
            state.result = Some(result);
            true
        }
    }

    struct FakeBackend {
        state: Rc<RefCell<FakeState>>,
    }

    impl FakeBackend {
        fn new() -> Self {
            Self {
                state: Rc::new(RefCell::new(FakeState::new())),
            }
        }

        fn callback(&self) -> FakeCallback {
            FakeCallback(self.state.clone())
        }
    }

    struct FakeFuture {
        state: Rc<RefCell<FakeState>>,
        started: bool,
        completed: bool,
    }

    impl Future for FakeFuture {
        type Output = Result<AccelerationSample, MotionError>;

        fn poll(self: core::pin::Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                this.started = true;
                let mut state = this.state.borrow_mut();
                state.start_count += 1;
                state.callback_attached = true;
                state.events.push(Event::Started);
            }

            let mut state = this.state.borrow_mut();
            match state.result.take() {
                Some(result) => {
                    this.completed = true;
                    state.delivered_count += 1;
                    state.events.push(Event::ResultDelivered);
                    Poll::Ready(result)
                }
                None => Poll::Pending,
            }
        }
    }

    impl Drop for FakeFuture {
        fn drop(&mut self) {
            if self.started && !self.completed {
                let mut state = self.state.borrow_mut();
                if state.callback_claimed {
                    state.result = None;
                } else {
                    if state.callback_attached {
                        state.callback_attached = false;
                        state.detach_count += 1;
                        state.events.push(Event::Detached);
                    }
                    state.cancellation_count += 1;
                    state.events.push(Event::CancellationRequested);
                }
            }
        }
    }

    impl MotionBackend for FakeBackend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type CurrentAccelerationFuture<'a>
            = FakeFuture
        where
            Self: 'a;

        fn current_acceleration<'a>(&'a mut self) -> Self::CurrentAccelerationFuture<'a> {
            self.state.borrow_mut().request_count += 1;
            FakeFuture {
                state: self.state.clone(),
                started: false,
                completed: false,
            }
        }
    }

    fn sample(x: f64, y: f64, z: f64, timestamp: u64) -> AccelerationSample {
        AccelerationSample::new(
            Acceleration::new(x, y, z).unwrap(),
            MotionTimestamp::from_monotonic_nanos(timestamp),
        )
    }

    fn context() -> Context<'static> {
        Context::from_waker(Waker::noop())
    }

    #[test]
    fn acceleration_rejects_non_finite_values_and_preserves_device_axes() {
        let acceleration = Acceleration::new(-1.25, 2.5, 9.80665).unwrap();
        assert_eq!(acceleration.x_meters_per_second_squared(), -1.25);
        assert_eq!(acceleration.y_meters_per_second_squared(), 2.5);
        assert_eq!(acceleration.z_meters_per_second_squared(), 9.80665);

        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                Acceleration::new(value, 0.0, 0.0),
                Err(MotionError::InvalidAcceleration)
            );
            assert_eq!(
                Acceleration::new(0.0, value, 0.0),
                Err(MotionError::InvalidAcceleration)
            );
            assert_eq!(
                Acceleration::new(0.0, 0.0, value),
                Err(MotionError::InvalidAcceleration)
            );
        }
    }

    #[test]
    fn timestamp_and_sample_preserve_monotonic_nanosecond_units() {
        let timestamp = MotionTimestamp::from_monotonic_nanos(1_700_000_123_456_789_012);
        assert_eq!(timestamp.monotonic_nanos(), 1_700_000_123_456_789_012);
        assert_eq!(
            MotionTimestamp::from_monotonic_nanos(u64::MAX).monotonic_nanos(),
            u64::MAX
        );
        let sample = AccelerationSample::new(Acceleration::new(1.0, 2.0, 3.0).unwrap(), timestamp);
        assert_eq!(sample.timestamp(), timestamp);
    }

    #[test]
    fn fake_backend_starts_on_first_poll_and_returns_one_sample() {
        let backend = FakeBackend::new();
        let callback = backend.callback();
        let state = backend.state.clone();
        let mut motion = Motion::new(backend);
        assert_eq!(motion.availability(), Availability::Available);

        let mut future = pin!(motion.current_acceleration());
        {
            let state = state.borrow();
            assert_eq!(state.request_count, 0);
            assert_eq!(state.start_count, 0);
        }
        let mut context = context();
        assert!(future.as_mut().poll(&mut context).is_pending());
        assert!(future.as_mut().poll(&mut context).is_pending());
        {
            let state = state.borrow();
            assert_eq!(state.request_count, 1);
            assert_eq!(state.start_count, 1);
            assert_eq!(state.callback_count, 0);
        }

        let expected = sample(-0.25, 0.5, 9.80665, 987_654_321);
        assert!(callback.complete(Ok(expected)));
        assert_eq!(
            future.as_mut().poll(&mut context),
            Poll::Ready(Ok(expected))
        );
        let state = state.borrow();
        assert_eq!(state.request_count, 1);
        assert_eq!(state.start_count, 1);
        assert_eq!(state.callback_count, 1);
        assert_eq!(state.delivered_count, 1);
        assert_eq!(state.events, [Event::Started, Event::ResultDelivered]);
    }

    #[test]
    fn error_mapping_preserves_category_and_native_code() {
        let code = PlatformErrorCode::new(-41).unwrap();
        let error = Error::new(ErrorKind::PermissionDenied).with_platform_code(code);
        let backend = FakeBackend::new();
        let callback = backend.callback();
        let mut motion = Motion::new(backend);
        let mut future = pin!(motion.current_acceleration());
        let mut context = context();
        assert!(future.as_mut().poll(&mut context).is_pending());
        assert!(callback.complete(Err(MotionError::Backend(error))));
        assert_eq!(
            future.as_mut().poll(&mut context),
            Poll::Ready(Err(MotionError::Backend(error)))
        );
        assert_eq!(
            MotionError::Backend(error).kind(),
            ErrorKind::PermissionDenied
        );
        assert_eq!(MotionError::Backend(error).platform_code(), Some(code));

        let invalid = Acceleration::new(f64::NAN, 0.0, 0.0).unwrap_err();
        assert_eq!(invalid.kind(), ErrorKind::InvalidInput);
        assert_eq!(invalid.platform_code(), None);
    }

    #[test]
    fn dropping_before_first_poll_starts_no_request_or_callback() {
        let backend = FakeBackend::new();
        let state = backend.state.clone();
        let mut motion = Motion::new(backend);
        drop(motion.current_acceleration());

        let state = state.borrow();
        assert_eq!(state.request_count, 0);
        assert_eq!(state.start_count, 0);
        assert_eq!(state.callback_count, 0);
        assert_eq!(state.detach_count, 0);
        assert_eq!(state.cancellation_count, 0);
        assert!(state.events.is_empty());
    }

    #[test]
    fn dropping_pending_request_detaches_before_cancel_and_ignores_late_callback() {
        let backend = FakeBackend::new();
        let callback = backend.callback();
        let state = backend.state.clone();
        let mut motion = Motion::new(backend);
        {
            let mut future = pin!(motion.current_acceleration());
            let mut context = context();
            assert!(future.as_mut().poll(&mut context).is_pending());
        }

        assert!(!callback.complete(Ok(sample(1.0, 2.0, 3.0, 4))));
        let state = state.borrow();
        assert_eq!(state.request_count, 1);
        assert_eq!(state.start_count, 1);
        assert_eq!(state.callback_count, 0);
        assert_eq!(state.late_callback_count, 1);
        assert_eq!(state.delivered_count, 0);
        assert_eq!(state.detach_count, 1);
        assert_eq!(state.cancellation_count, 1);
        assert_eq!(
            state.events,
            [
                Event::Started,
                Event::Detached,
                Event::CancellationRequested
            ]
        );
    }

    #[test]
    fn duplicate_and_late_callbacks_do_not_deliver_more_than_once() {
        let backend = FakeBackend::new();
        let callback = backend.callback();
        let state = backend.state.clone();
        let mut motion = Motion::new(backend);
        let mut future = pin!(motion.current_acceleration());
        let mut context = context();
        assert!(future.as_mut().poll(&mut context).is_pending());

        let first = sample(1.0, 2.0, 3.0, 10);
        let duplicate = sample(4.0, 5.0, 6.0, 20);
        assert!(callback.complete(Ok(first)));
        assert!(!callback.complete(Ok(duplicate)));
        assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(Ok(first)));
        assert!(!callback.complete(Ok(duplicate)));

        let state = state.borrow();
        assert_eq!(state.callback_count, 1);
        assert_eq!(state.late_callback_count, 2);
        assert_eq!(state.delivered_count, 1);
        assert_eq!(state.detach_count, 0);
        assert_eq!(state.cancellation_count, 0);
    }

    #[test]
    fn dropping_after_callback_suppresses_an_unobserved_result() {
        let backend = FakeBackend::new();
        let callback = backend.callback();
        let state = backend.state.clone();
        let mut motion = Motion::new(backend);
        {
            let mut future = pin!(motion.current_acceleration());
            let mut context = context();
            assert!(future.as_mut().poll(&mut context).is_pending());
            assert!(callback.complete(Ok(sample(1.0, 2.0, 3.0, 10))));
        }

        let state = state.borrow();
        assert_eq!(state.callback_count, 1);
        assert_eq!(state.delivered_count, 0);
        assert_eq!(state.result, None);
        assert_eq!(state.detach_count, 0);
        assert_eq!(state.cancellation_count, 0);
    }
}
