#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable values and a static backend contract for one-shot location requests."]

use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A WGS-84 geographic coordinate in decimal degrees.
///
/// Construction rejects non-finite values, latitude outside -90 through 90 degrees, and
/// longitude outside -180 through 180 degrees. Positive and negative 180 degrees are both valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinate {
    latitude_degrees: f64,
    longitude_degrees: f64,
}

impl Coordinate {
    /// Creates a validated WGS-84 coordinate in decimal degrees.
    pub fn new(latitude_degrees: f64, longitude_degrees: f64) -> Result<Self, LocationError> {
        if !latitude_degrees.is_finite()
            || !longitude_degrees.is_finite()
            || !(-90.0..=90.0).contains(&latitude_degrees)
            || !(-180.0..=180.0).contains(&longitude_degrees)
        {
            return Err(LocationError::InvalidCoordinate);
        }
        Ok(Self {
            latitude_degrees,
            longitude_degrees,
        })
    }

    /// Returns latitude in decimal degrees.
    pub const fn latitude_degrees(self) -> f64 {
        self.latitude_degrees
    }

    /// Returns longitude in decimal degrees.
    pub const fn longitude_degrees(self) -> f64 {
        self.longitude_degrees
    }
}

/// A caller-selected desired horizontal accuracy in meters.
///
/// A smaller value requests a more precise fix. Zero is valid and requests the best available
/// precision; it does not mean that an exact coordinate exists or that a backend must meet it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccuracyTarget(f64);

impl AccuracyTarget {
    /// Creates a finite, non-negative horizontal-accuracy target in meters.
    pub fn new(meters: f64) -> Result<Self, LocationError> {
        if !meters.is_finite() || meters < 0.0 {
            return Err(LocationError::InvalidAccuracy);
        }
        Ok(Self(meters))
    }

    /// Returns the desired horizontal accuracy in meters.
    pub const fn meters(self) -> f64 {
        self.0
    }
}

/// The Unix timestamp for a location measurement, in milliseconds since 1970-01-01T00:00:00Z.
///
/// This is wall-clock time, not a monotonic clock or the time at which a backend returned the fix.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocationTimestamp(u64);

impl LocationTimestamp {
    /// Creates a timestamp from milliseconds since the Unix epoch.
    pub const fn from_unix_millis(milliseconds: u64) -> Self {
        Self(milliseconds)
    }

    /// Returns milliseconds since the Unix epoch.
    pub const fn unix_millis(self) -> u64 {
        self.0
    }
}

/// One owned current-location result.
///
/// A fix contains only scalar values and has no heap allocation or borrowed native object. Its
/// coordinate is WGS-84 decimal degrees, its horizontal-accuracy estimate is in meters, and its
/// timestamp is Unix milliseconds. The accuracy value is an estimate, not a confidence guarantee.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocationFix {
    coordinate: Coordinate,
    horizontal_accuracy_meters: f64,
    timestamp: LocationTimestamp,
}

impl LocationFix {
    /// Creates a fix with a finite, non-negative horizontal-accuracy estimate.
    pub fn new(
        coordinate: Coordinate,
        horizontal_accuracy_meters: f64,
        timestamp: LocationTimestamp,
    ) -> Result<Self, LocationError> {
        if !horizontal_accuracy_meters.is_finite() || horizontal_accuracy_meters < 0.0 {
            return Err(LocationError::InvalidAccuracy);
        }
        Ok(Self {
            coordinate,
            horizontal_accuracy_meters,
            timestamp,
        })
    }

    /// Returns the WGS-84 coordinate by value.
    pub const fn coordinate(self) -> Coordinate {
        self.coordinate
    }

    /// Returns the backend-reported horizontal-accuracy estimate in meters.
    pub const fn horizontal_accuracy_meters(self) -> f64 {
        self.horizontal_accuracy_meters
    }

    /// Returns the measurement timestamp in Unix milliseconds.
    pub const fn timestamp(self) -> LocationTimestamp {
        self.timestamp
    }
}

/// Options for one one-shot request for the current location.
///
/// The portable request carries no timeout or completion deadline. A caller that needs a deadline
/// must impose it externally and drop the pending future under the backend's cancellation contract.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocationRequest {
    accuracy_target: AccuracyTarget,
}

impl LocationRequest {
    /// Creates a current-location request with the caller's desired accuracy target.
    pub const fn new(accuracy_target: AccuracyTarget) -> Self {
        Self { accuracy_target }
    }

    /// Returns the desired horizontal accuracy; a backend may return a less precise fix.
    pub const fn accuracy_target(self) -> AccuracyTarget {
        self.accuracy_target
    }
}

/// The normalized location-authorization state reported by a backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LocationAuthorization {
    /// Authorization has not been queried or cannot be classified.
    Unknown,
    /// The platform has not yet asked the user.
    NotDetermined,
    /// The user or platform denied location access.
    Denied,
    /// Location access is blocked by policy or device restrictions.
    Restricted,
    /// Location access is allowed while the application is in the foreground.
    Foreground,
    /// The reported authorization includes background access; this is status only, not a
    /// background-location operation.
    Background,
}

impl LocationAuthorization {
    /// Reports whether this state permits a foreground location request.
    pub const fn allows_foreground(self) -> bool {
        matches!(self, Self::Foreground | Self::Background)
    }
}

/// A stable location error that preserves an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LocationError {
    /// A coordinate is non-finite or outside the WGS-84 latitude/longitude ranges.
    InvalidCoordinate,
    /// An accuracy target or estimate is negative or non-finite.
    InvalidAccuracy,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl LocationError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidCoordinate | Self::InvalidAccuracy => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidCoordinate | Self::InvalidAccuracy => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for one-shot current-location operations.
///
/// A request starts only when its returned future is first polled. If a pending current-location
/// future is dropped, the backend must request cancellation of its native one-shot operation when
/// the native API supports cancellation. A native completion racing with cancellation is ignored
/// after the Rust future is dropped; callback state must remain safe until released and must be
/// released exactly once. Each started backend operation has one terminal success or error. If
/// its future remains live through that terminal state, it must yield that result once; if the
/// caller drops the future first, the result is suppressed. Duplicate or late native completions
/// are ignored. No executor or Send requirement is imposed.
pub trait LocationBackend {
    /// Reports non-prompting backend availability, separately from authorization state.
    ///
    /// This signal does not imply that permission is granted or that a request will succeed; use
    /// [`LocationBackend::authorization`] to query the normalized authorization state.
    fn availability(&self) -> Availability;

    /// The future type for a non-prompting authorization-state query.
    type AuthorizationFuture<'a>: Future<Output = Result<LocationAuthorization, LocationError>> + 'a
    where
        Self: 'a;

    /// Queries the normalized authorization state without requesting permission.
    fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a>;

    /// The future type for an explicit foreground-location authorization request.
    type RequestAuthorizationFuture<'a>: Future<Output = Result<LocationAuthorization, LocationError>>
        + 'a
    where
        Self: 'a;

    /// Explicitly requests authorization for foreground, one-shot location use.
    ///
    /// A native backend may prompt only because this operation was explicitly requested and
    /// polled. It must not silently request background authorization. Dropping this future
    /// abandons its result but cannot be assumed to dismiss a permission prompt already shown;
    /// any native callback state must remain safe and complete at most once.
    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a>;

    /// The future type for one owned location fix.
    type CurrentFuture<'a>: Future<Output = Result<LocationFix, LocationError>> + 'a
    where
        Self: 'a;

    /// Requests one current-location fix using the supplied accuracy target.
    ///
    /// The target is a preference, not a required accuracy guarantee. The backend may return a
    /// valid fix with a larger horizontal-accuracy estimate or fail if its own policy requires
    /// another outcome. The timestamp must describe the measurement time, including when a cached
    /// fix is returned. The portable contract defines no timeout or completion deadline; a
    /// backend-reported timeout is returned as a backend error with its category and optional
    /// native code preserved.
    fn current<'a>(&'a mut self, request: LocationRequest) -> Self::CurrentFuture<'a>;
}

/// A thin facade over caller-owned, statically selected location-backend state.
pub struct Location<B> {
    backend: B,
}

impl<B: LocationBackend> Location<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Queries the backend's normalized authorization state without requesting permission.
    pub async fn authorization(&mut self) -> Result<LocationAuthorization, LocationError> {
        self.backend.authorization().await
    }

    /// Explicitly requests foreground-location authorization through the selected backend.
    pub async fn request_authorization(&mut self) -> Result<LocationAuthorization, LocationError> {
        self.backend.request_authorization().await
    }

    /// Requests one owned current-location fix through the selected backend.
    ///
    /// The operation begins on first poll. Dropping a pending future requests native cancellation
    /// when the backend's native API supports it; a late completion is discarded safely.
    pub async fn current(
        &mut self,
        request: LocationRequest,
    ) -> Result<LocationFix, LocationError> {
        self.backend.current(request).await
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
    use core::future::{Future, Ready, ready};
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    struct Backend {
        authorization: LocationAuthorization,
        request_result: LocationAuthorization,
        request_count: u32,
        last_request: Option<LocationRequest>,
        current_result: Option<Result<LocationFix, LocationError>>,
        current_starts: u32,
        native_cancellations: u32,
    }

    impl Backend {
        fn new() -> Self {
            Self {
                authorization: LocationAuthorization::NotDetermined,
                request_result: LocationAuthorization::Foreground,
                request_count: 0,
                last_request: None,
                current_result: None,
                current_starts: 0,
                native_cancellations: 0,
            }
        }
    }

    struct CurrentFuture<'a> {
        backend: &'a mut Backend,
        result: Option<Result<LocationFix, LocationError>>,
        started: bool,
        completed: bool,
    }

    impl Future for CurrentFuture<'_> {
        type Output = Result<LocationFix, LocationError>;

        fn poll(self: core::pin::Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                this.started = true;
                this.backend.current_starts += 1;
            }
            match this.result.take() {
                Some(result) => {
                    this.completed = true;
                    Poll::Ready(result)
                }
                None => Poll::Pending,
            }
        }
    }

    impl Drop for CurrentFuture<'_> {
        fn drop(&mut self) {
            if self.started && !self.completed {
                self.backend.native_cancellations += 1;
            }
        }
    }

    impl LocationBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type AuthorizationFuture<'a>
            = Ready<Result<LocationAuthorization, LocationError>>
        where
            Self: 'a;

        fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
            ready(Ok(self.authorization))
        }

        type RequestAuthorizationFuture<'a>
            = Ready<Result<LocationAuthorization, LocationError>>
        where
            Self: 'a;

        fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
            self.request_count += 1;
            self.authorization = self.request_result;
            ready(Ok(self.authorization))
        }

        type CurrentFuture<'a>
            = CurrentFuture<'a>
        where
            Self: 'a;

        fn current<'a>(&'a mut self, request: LocationRequest) -> Self::CurrentFuture<'a> {
            self.last_request = Some(request);
            let result = self.current_result;
            self.current_result = None;
            CurrentFuture {
                backend: self,
                result,
                started: false,
                completed: false,
            }
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

    fn coordinate() -> Coordinate {
        Coordinate::new(43.6532, -79.3832).unwrap()
    }

    fn request() -> LocationRequest {
        LocationRequest::new(AccuracyTarget::new(25.0).unwrap())
    }

    fn fix() -> LocationFix {
        LocationFix::new(
            coordinate(),
            12.5,
            LocationTimestamp::from_unix_millis(1_700_000_000_000),
        )
        .unwrap()
    }

    #[test]
    fn coordinates_accept_inclusive_bounds_and_reject_out_of_range_values() {
        assert_eq!(
            Coordinate::new(-90.0, -180.0).unwrap().latitude_degrees(),
            -90.0
        );
        assert_eq!(
            Coordinate::new(90.0, 180.0).unwrap().longitude_degrees(),
            180.0
        );
        assert_eq!(
            Coordinate::new(-90.000_001, 0.0),
            Err(LocationError::InvalidCoordinate)
        );
        assert_eq!(
            Coordinate::new(90.000_001, 0.0),
            Err(LocationError::InvalidCoordinate)
        );
        assert_eq!(
            Coordinate::new(0.0, -180.000_001),
            Err(LocationError::InvalidCoordinate)
        );
        assert_eq!(
            Coordinate::new(0.0, 180.000_001),
            Err(LocationError::InvalidCoordinate)
        );
    }

    #[test]
    fn coordinates_reject_nan_and_both_infinities_in_each_component() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                Coordinate::new(value, 0.0),
                Err(LocationError::InvalidCoordinate)
            );
            assert_eq!(
                Coordinate::new(0.0, value),
                Err(LocationError::InvalidCoordinate)
            );
        }
    }

    #[test]
    fn request_and_fix_values_validate_accuracy_and_preserve_units() {
        let target = AccuracyTarget::new(0.0).unwrap();
        let request = LocationRequest::new(target);
        assert_eq!(request.accuracy_target().meters(), 0.0);
        assert_eq!(
            AccuracyTarget::new(-1.0),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            AccuracyTarget::new(f64::NAN),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            AccuracyTarget::new(f64::INFINITY),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            AccuracyTarget::new(f64::NEG_INFINITY),
            Err(LocationError::InvalidAccuracy)
        );

        let timestamp = LocationTimestamp::from_unix_millis(1_700_000_000_000);
        let fix = LocationFix::new(coordinate(), 4.25, timestamp).unwrap();
        assert_eq!(fix.coordinate(), coordinate());
        assert_eq!(fix.horizontal_accuracy_meters(), 4.25);
        assert_eq!(fix.timestamp().unix_millis(), 1_700_000_000_000);
        assert_eq!(
            LocationFix::new(coordinate(), f64::NEG_INFINITY, timestamp),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            LocationFix::new(coordinate(), -1.0, timestamp),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            LocationFix::new(coordinate(), f64::NAN, timestamp),
            Err(LocationError::InvalidAccuracy)
        );
        assert_eq!(
            LocationFix::new(coordinate(), f64::INFINITY, timestamp),
            Err(LocationError::InvalidAccuracy)
        );
    }

    #[test]
    fn authorization_states_map_only_to_their_declared_location_scopes() {
        assert!(!LocationAuthorization::Unknown.allows_foreground());
        assert!(!LocationAuthorization::NotDetermined.allows_foreground());
        assert!(!LocationAuthorization::Denied.allows_foreground());
        assert!(!LocationAuthorization::Restricted.allows_foreground());
        assert!(LocationAuthorization::Foreground.allows_foreground());
        assert!(LocationAuthorization::Background.allows_foreground());

        let mut location = Location::new(Backend::new());
        assert_eq!(
            run_ready(location.authorization()),
            Ok(LocationAuthorization::NotDetermined)
        );
        assert_eq!(
            run_ready(location.request_authorization()),
            Ok(LocationAuthorization::Foreground)
        );
        assert_eq!(location.backend().request_count, 1);
    }

    #[test]
    fn fake_backend_returns_one_fix_and_preserves_backend_error_category_and_code() {
        let mut backend = Backend::new();
        backend.current_result = Some(Ok(fix()));
        let mut location = Location::new(backend);
        let request = request();

        assert_eq!(location.availability(), Availability::Available);
        assert_eq!(run_ready(location.current(request)), Ok(fix()));
        assert_eq!(location.backend().last_request, Some(request));
        assert_eq!(location.backend().current_starts, 1);
        assert_eq!(location.backend().native_cancellations, 0);

        let native_code = PlatformErrorCode::new(-23).unwrap();
        let backend_error = LocationError::Backend(
            Error::new(ErrorKind::PermissionDenied).with_platform_code(native_code),
        );
        location.backend_mut().current_result = Some(Err(backend_error));
        assert_eq!(run_ready(location.current(request)), Err(backend_error));
        assert_eq!(backend_error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(backend_error.platform_code(), Some(native_code));
    }

    #[test]
    fn dropping_an_unpolled_request_starts_no_backend_work() {
        let mut location = Location::new(Backend::new());
        drop(location.current(request()));
        assert_eq!(location.backend().current_starts, 0);
        assert_eq!(location.backend().native_cancellations, 0);
    }

    #[test]
    fn dropping_a_pending_fix_future_requests_native_cancellation_once() {
        let mut location = Location::new(Backend::new());
        {
            let mut future = pin!(location.current(request()));
            let mut context = Context::from_waker(Waker::noop());
            assert!(future.as_mut().poll(&mut context).is_pending());
        }
        assert_eq!(location.backend().current_starts, 1);
        assert_eq!(location.backend().native_cancellations, 1);
    }
}
