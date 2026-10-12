use crate::configuration::{
    BackgroundModeValue, has_always_usage_descriptions, has_location_background_mode,
    is_nonempty_string_value, is_valid_temporary_accuracy_purpose_key,
    requires_legacy_always_usage_description,
};
use crate::conversion::{
    NativeAccuracy, UpdateAuthorizationRequirement, authorization_downgraded,
    authorization_from_native, configuration_error, error_from_native, fix_from_native,
    native_accuracy, should_request_always_authorization, significant_change_preflight,
    updates_permitted,
};
use crate::geofence::{IosGeofenceError, geofence_authorization_preflight};
use crate::operation::{CompletionCell, LocationUpdateQueue};
use block2::RcBlock;
use core::cell::Cell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_core::{Availability, Error, ErrorKind};
use framework_location::{
    LocationAuthorization, LocationBackend, LocationError, LocationFix, LocationRequest,
    LocationUpdate, LocationUpdateKind, LocationUpdateSession, LocationUpdatesBackend,
};
use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_location::{
    CLActivityType, CLLocation, CLLocationManager, CLLocationManagerDelegate,
};
use objc2_foundation::{
    NSArray, NSBundle, NSDate, NSDictionary, NSError, NSObject, NSObjectProtocol, NSString,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::task::Waker;

type Completion = Arc<CompletionCell<NativeOutcome, LocationError>>;

#[derive(Clone, Copy)]
enum NativeOutcome {
    Authorization(LocationAuthorization),
    AccuracyAuthorization(IosAccuracyAuthorization),
    Current(LocationFix),
    TemporaryAccuracyRequested,
}

#[derive(Clone, Copy)]
enum LocationUpdateService {
    Standard(LocationRequest),
    SignificantChange,
}

enum DelegateState {
    RequestAuthorization {
        completion: Completion,
        authorization_baseline: i32,
    },
    Current {
        completion: Completion,
    },
    Updates {
        queue: Arc<LocationUpdateQueue>,
        authorization: UpdateAuthorizationState,
    },
}

struct UpdateAuthorizationState {
    requirement: UpdateAuthorizationRequirement,
    previous: Cell<LocationAuthorization>,
}

impl UpdateAuthorizationState {
    fn new(requirement: UpdateAuthorizationRequirement, initial: LocationAuthorization) -> Self {
        Self {
            requirement,
            previous: Cell::new(initial),
        }
    }

    fn permits(&self, current: LocationAuthorization) -> bool {
        updates_permitted(current, self.requirement)
    }

    fn observe(&self, current: LocationAuthorization) -> bool {
        let previous = self.previous.replace(current);
        authorization_downgraded(previous, current, self.requirement)
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateState]
    struct LocationDelegate;

    // SAFETY: NSObjectProtocol adds no methods here; LocationDelegate derives directly from
    // NSObject and is only constructed and accessed on its owning main thread.
    unsafe impl NSObjectProtocol for LocationDelegate {}

    // SAFETY: Core Location calls this delegate on the manager's creation run loop; this backend
    // creates it on main, retains it while active, contains panics, and never stores raw pointers.
    #[allow(non_snake_case)]
    unsafe impl CLLocationManagerDelegate for LocationDelegate {
        #[unsafe(method(locationManager:didUpdateLocations:))]
        unsafe fn locationManager_didUpdateLocations(
            &self,
            manager: &CLLocationManager,
            locations: &NSArray<CLLocation>,
        ) {
            let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                return;
            };
            let Some(_manager) = (unsafe { retain_callback_manager(manager) }) else {
                return;
            };
            match self.ivars() {
                DelegateState::Current { completion } => {
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        autoreleasepool(|_| {
                            let Some(location) = locations.lastObject() else {
                                return Err(LocationError::Backend(Error::new(
                                    ErrorKind::Unavailable,
                                )));
                            };
                            fix_from_native_location(&location).map(NativeOutcome::Current)
                        })
                    }))
                    .unwrap_or_else(|_| Err(internal_error()));
                    completion.complete(result);
                }
                DelegateState::Updates {
                    queue,
                    authorization,
                } => {
                    let current_authorization =
                        authorization_from_native(native_authorization_status());
                    if authorization.observe(current_authorization) {
                        queue.push_batch([LocationUpdateKind::Error(permission_error())]);
                    }
                    if !authorization.permits(current_authorization) {
                        return;
                    }
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        autoreleasepool(|_| {
                            if locations.is_empty() {
                                queue.push_batch([LocationUpdateKind::Error(
                                    LocationError::Backend(Error::new(ErrorKind::Unavailable)),
                                )]);
                            } else {
                                queue.push_batch((0..locations.len()).map(|index| {
                                    update_kind_from_native_location(
                                        &locations.objectAtIndex(index),
                                    )
                                }));
                            }
                        })
                    }));
                    if result.is_err() {
                        queue.push_batch([LocationUpdateKind::Error(internal_error())]);
                    }
                }
                DelegateState::RequestAuthorization { .. } => {}
            }
        }

        #[unsafe(method(locationManager:didFailWithError:))]
        unsafe fn locationManager_didFailWithError(
            &self,
            manager: &CLLocationManager,
            error: &NSError,
        ) {
            let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                return;
            };
            let Some(_manager) = (unsafe { retain_callback_manager(manager) }) else {
                return;
            };
            match self.ivars() {
                DelegateState::Current { completion } => {
                    let result =
                        catch_unwind(AssertUnwindSafe(|| error_from_native(error.code() as i64)))
                            .unwrap_or_else(|_| internal_error());
                    completion.complete(Err(result));
                }
                DelegateState::Updates { queue, .. } => {
                    let result =
                        catch_unwind(AssertUnwindSafe(|| error_from_native(error.code() as i64)))
                            .unwrap_or_else(|_| internal_error());
                    queue.push_batch([LocationUpdateKind::Error(result)]);
                }
                DelegateState::RequestAuthorization { .. } => {}
            }
        }

        #[allow(deprecated)]
        #[unsafe(method(locationManager:didChangeAuthorizationStatus:))]
        unsafe fn locationManager_didChangeAuthorizationStatus(
            &self,
            manager: &CLLocationManager,
            status: objc2_core_location::CLAuthorizationStatus,
        ) {
            let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                return;
            };
            let Some(_manager) = (unsafe { retain_callback_manager(manager) }) else {
                return;
            };
            self.complete_authorization_change(status.0);
        }

        #[unsafe(method(locationManagerDidChangeAuthorization:))]
        unsafe fn locationManagerDidChangeAuthorization(&self, manager: &CLLocationManager) {
            let Some(_receiver) = (unsafe { retain_callback_receiver(self) }) else {
                return;
            };
            let Some(_manager) = (unsafe { retain_callback_manager(manager) }) else {
                return;
            };
            self.complete_authorization_change(native_authorization_status());
        }
    }
);

impl LocationDelegate {
    fn new(marker: MainThreadMarker, state: DelegateState) -> Retained<Self> {
        let allocated: Allocated<Self> = marker.alloc::<Self>();
        let allocated = allocated.set_ivars(state);
        // SAFETY: The class derives directly from NSObject, and its ivars are initialized before
        // calling NSObject's designated `init` implementation.
        unsafe { msg_send![super(allocated), init] }
    }

    fn complete_authorization_change(&self, raw_status: i32) {
        match self.ivars() {
            DelegateState::RequestAuthorization {
                completion,
                authorization_baseline,
            } if *authorization_baseline != raw_status => {
                let authorization = authorization_from_native(raw_status);
                completion.complete(Ok(NativeOutcome::Authorization(authorization)));
            }
            DelegateState::Updates {
                queue,
                authorization,
            } => {
                if authorization.observe(authorization_from_native(raw_status)) {
                    queue.push_batch([LocationUpdateKind::Error(permission_error())]);
                }
            }
            DelegateState::RequestAuthorization { .. } | DelegateState::Current { .. } => {}
        }
    }
}

fn fix_from_native_location(location: &CLLocation) -> Result<LocationFix, LocationError> {
    // SAFETY: Core Location supplies a live CLLocation in the callback array; the callback-scoped
    // reference remains valid for these property reads.
    let coordinate = unsafe { location.coordinate() };
    // SAFETY: The CLLocation is valid for the duration of this call.
    let horizontal_accuracy = unsafe { location.horizontalAccuracy() };
    // SAFETY: The CLLocation is valid for the duration of this call.
    let timestamp: Retained<NSDate> = unsafe { location.timestamp() };
    fix_from_native(
        coordinate.latitude,
        coordinate.longitude,
        horizontal_accuracy,
        timestamp.timeIntervalSince1970(),
    )
}

fn update_kind_from_native_location(location: &CLLocation) -> LocationUpdateKind {
    match fix_from_native_location(location) {
        Ok(fix) => LocationUpdateKind::Fix(fix),
        Err(error) => LocationUpdateKind::Error(error),
    }
}

/// iOS non-prompting authorization-query future.
pub type IosAuthorizationFuture<'a> = IosLocationFuture<'a, LocationAuthorization>;

/// iOS foreground-authorization request future.
pub type IosRequestAuthorizationFuture<'a> = IosLocationFuture<'a, LocationAuthorization>;

/// iOS Always-authorization request future.
pub type IosRequestAlwaysAuthorizationFuture<'a> = IosLocationFuture<'a, LocationAuthorization>;

/// iOS temporary-full-accuracy authorization request future.
pub type IosRequestTemporaryFullAccuracyAuthorizationFuture<'a> =
    IosLocationFuture<'a, IosAccuracyAuthorization>;

/// iOS one-shot current-location future.
pub type IosCurrentFuture<'a> = IosLocationFuture<'a, LocationFix>;

/// A location-update session that owns its native manager and retained delegate.
///
/// The session exclusively borrows its backend and must be used and dropped on the backend's
/// main thread. It may own standard foreground/background updates or significant-change updates.
/// Starting any session never requests permission. Dropping the session detaches callback state
/// and stops the selected native location service.
pub struct IosLocationUpdates<'a> {
    _backend: &'a mut IosLocationBackend,
    queue: Arc<LocationUpdateQueue>,
    manager: Retained<CLLocationManager>,
    service: LocationUpdateService,
    _delegate: Retained<LocationDelegate>,
    _main_thread: MainThreadMarker,
}

/// One cancellable wait for the next event from an iOS location-update session.
///
/// Dropping this future clears its waker but does not stop the session or discard queued events.
pub struct IosLocationUpdateFuture<'session, 'backend> {
    session: &'session mut IosLocationUpdates<'backend>,
    waiter: Option<Waker>,
}

impl Future for IosLocationUpdateFuture<'_, '_> {
    type Output = LocationUpdate;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.session.queue.poll(context) {
            Poll::Ready(event) => {
                this.waiter = None;
                Poll::Ready(event)
            }
            Poll::Pending => {
                this.waiter = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl Drop for IosLocationUpdateFuture<'_, '_> {
    fn drop(&mut self) {
        if let Some(waker) = self.waiter.take() {
            self.session.queue.clear_waker(&waker);
        }
    }
}

impl Drop for IosLocationUpdates<'_> {
    fn drop(&mut self) {
        self.queue.detach();
        // SAFETY: The session and manager are main-thread-bound; the retained delegate remains
        // alive through both calls, and clearing it prevents any callback after detachment.
        unsafe { self.manager.setDelegate(None) };
        match self.service {
            LocationUpdateService::Standard(_) => {
                // SAFETY: The manager started standard updates on this thread and may be active.
                unsafe { self.manager.stopUpdatingLocation() };
            }
            LocationUpdateService::SignificantChange => {
                // SAFETY: The manager started significant-change monitoring on this thread and
                // may be active; this is its matching public stop operation.
                unsafe { self.manager.stopMonitoringSignificantLocationChanges() };
            }
        }
    }
}

/// A lazy operation that owns its native manager, delegate, and exactly-once callback state.
///
/// The future borrows its backend mutably, so a caller cannot start another operation through
/// that backend until this future completes or is dropped. It is main-thread-affine and must be
/// polled and dropped on the main thread that owns the backend.
pub struct IosLocationFuture<'a, T> {
    backend: &'a mut IosLocationBackend,
    operation: Operation,
    completion: Completion,
    manager: Option<Retained<CLLocationManager>>,
    delegate: Option<Retained<LocationDelegate>>,
    temporary_accuracy_purpose_key: Option<Retained<NSString>>,
    transform: fn(NativeOutcome) -> Result<T, LocationError>,
    started: bool,
    finished: bool,
    _main_thread: MainThreadMarker,
    _output: PhantomData<T>,
}

#[derive(Clone, Copy)]
enum Operation {
    Authorization,
    RequestAuthorization,
    RequestAlwaysAuthorization,
    RequestTemporaryFullAccuracyAuthorization,
    Current(LocationRequest),
}

impl<'a, T> IosLocationFuture<'a, T> {
    fn new(
        backend: &'a mut IosLocationBackend,
        operation: Operation,
        transform: fn(NativeOutcome) -> Result<T, LocationError>,
    ) -> Self {
        let marker = backend.marker;
        Self {
            backend,
            operation,
            completion: Arc::new(CompletionCell::new()),
            manager: None,
            delegate: None,
            temporary_accuracy_purpose_key: None,
            transform,
            started: false,
            finished: false,
            _main_thread: marker,
            _output: PhantomData,
        }
    }

    fn with_temporary_accuracy_purpose_key(mut self, purpose_key: Retained<NSString>) -> Self {
        self.temporary_accuracy_purpose_key = Some(purpose_key);
        self
    }

    /// Borrows the operation-scoped native manager after the future's first poll.
    ///
    /// Direct Core Location calls through this handle can change the active request's behavior.
    pub fn native_location_manager(&self) -> Option<&CLLocationManager> {
        self.manager.as_deref()
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        match self.operation {
            Operation::Authorization => {
                self.completion.complete(Ok(NativeOutcome::Authorization(
                    authorization_from_native(native_authorization_status()),
                )));
            }
            Operation::RequestAuthorization => self.start_authorization_request(),
            Operation::RequestAlwaysAuthorization => self.start_always_authorization_request(),
            Operation::RequestTemporaryFullAccuracyAuthorization => {
                self.start_temporary_full_accuracy_authorization_request()
            }
            Operation::Current(request) => self.start_current(request),
        }
    }

    fn start_authorization_request(&mut self) {
        let status = native_authorization_status();
        let (manager, delegate) = new_operation_objects(
            self.backend.marker,
            DelegateState::RequestAuthorization {
                completion: self.completion.clone(),
                authorization_baseline: status,
            },
        );
        self.manager = Some(manager);
        self.delegate = Some(delegate);
        let manager = self.manager.as_ref().expect("manager set above");
        // SAFETY: The backend marker keeps this future on the main thread and the manager was
        // created on that thread with its callback delegate retained by this future.
        unsafe { manager.requestWhenInUseAuthorization() };
    }

    fn start_always_authorization_request(&mut self) {
        let status = native_authorization_status();
        let authorization = authorization_from_native(status);
        if !should_request_always_authorization(authorization) {
            self.completion
                .complete(Ok(NativeOutcome::Authorization(authorization)));
            return;
        }
        if !main_bundle_has_always_usage_descriptions() {
            self.completion.complete(Err(configuration_error()));
            return;
        }
        let (manager, delegate) = new_operation_objects(
            self.backend.marker,
            DelegateState::RequestAuthorization {
                completion: self.completion.clone(),
                authorization_baseline: status,
            },
        );
        self.manager = Some(manager);
        self.delegate = Some(delegate);
        let manager = self.manager.as_ref().expect("manager set above");
        // SAFETY: This explicit operation was polled on the backend's main thread, has the
        // required host usage-description keys, and retains its callback delegate.
        unsafe { manager.requestAlwaysAuthorization() };
    }

    fn start_temporary_full_accuracy_authorization_request(&mut self) {
        if !objc2::available!(ios = 14.0, ..) {
            self.completion.complete(Err(unsupported_error()));
            return;
        }
        if !authorization_from_native(native_authorization_status()).allows_foreground() {
            self.completion.complete(Err(permission_error()));
            return;
        }
        let Some(accuracy) = current_accuracy_authorization() else {
            self.completion.complete(Err(unsupported_error()));
            return;
        };
        match accuracy {
            IosAccuracyAuthorization::FullAccuracy | IosAccuracyAuthorization::Unknown(_) => {
                self.completion
                    .complete(Ok(NativeOutcome::AccuracyAuthorization(accuracy)));
                return;
            }
            IosAccuracyAuthorization::ReducedAccuracy => {}
        }
        let Some(purpose_key) = self.temporary_accuracy_purpose_key.as_deref() else {
            self.completion.complete(Err(internal_error()));
            return;
        };
        if !main_bundle_has_temporary_full_accuracy_usage_description(purpose_key) {
            self.completion.complete(Err(configuration_error()));
            return;
        }

        // SAFETY: The backend's main-thread marker keeps this manager and its callback on the
        // main run loop required by Core Location's completion-handler contract.
        let manager = unsafe { CLLocationManager::new() };
        self.manager = Some(manager);
        let completion_state = Arc::clone(&self.completion);
        let completion: RcBlock<dyn Fn(*mut NSError)> =
            RcBlock::new(move |native_error: *mut NSError| {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        autoreleasepool(|_| {
                            // SAFETY: Core Location supplies a nullable NSError valid for this block
                            // invocation; its code is copied before the callback returns.
                            if let Some(error) = unsafe { native_error.as_ref() } {
                                return Err(error_from_native(error.code() as i64));
                            }
                            Ok(NativeOutcome::TemporaryAccuracyRequested)
                        })
                    }))
                    .unwrap_or_else(|_| Err(internal_error()));
                    completion_state.complete(result);
                }));
            });
        let purpose_key = self
            .temporary_accuracy_purpose_key
            .as_deref()
            .expect("temporary-accuracy operation owns its purpose key");
        let manager = self.manager.as_ref().expect("manager set above");
        // SAFETY: iOS 14 availability, foreground-capable app authorization, a reduced-accuracy
        // snapshot, and an exact nonempty Info.plist purpose-key entry were checked first. This
        // is the explicit caller-requested prompt path; the callback block owns no manager state.
        unsafe {
            manager.requestTemporaryFullAccuracyAuthorizationWithPurposeKey_completion(
                purpose_key,
                Some(&completion),
            )
        };
    }

    fn start_current(&mut self, request: LocationRequest) {
        let (manager, delegate) = new_operation_objects(
            self.backend.marker,
            DelegateState::Current {
                completion: self.completion.clone(),
            },
        );
        self.manager = Some(manager);
        self.delegate = Some(delegate);
        let manager = self.manager.as_ref().expect("manager set above");
        let native_accuracy = match native_accuracy(request.accuracy_target()) {
            NativeAccuracy::Best => {
                // SAFETY: Core Location declares this public accuracy constant for iOS.
                unsafe { objc2_core_location::kCLLocationAccuracyBest }
            }
            NativeAccuracy::Meters(meters) => meters,
        };
        // SAFETY: The backend marker keeps this manager on its creation thread; the desired
        // accuracy is a finite, non-negative portable request mapped to Core Location meters.
        unsafe { manager.setDesiredAccuracy(native_accuracy) };
        // SAFETY: Core Location's documented one-shot request is used with the retained delegate
        // and the main run loop on which this manager was created.
        unsafe { manager.requestLocation() };
    }
}

fn new_operation_objects(
    marker: MainThreadMarker,
    state: DelegateState,
) -> (Retained<CLLocationManager>, Retained<LocationDelegate>) {
    autoreleasepool(|_| {
        // SAFETY: CLLocationManager's initializer is public and this backend supplies the
        // main-thread/run-loop context required for its delegate callbacks.
        let manager = unsafe { CLLocationManager::new() };
        let delegate = LocationDelegate::new(marker, state);
        let delegate_object = ProtocolObject::from_ref(&*delegate);
        // SAFETY: CLLocationManager's delegate is a zero-ownership weak property; the operation
        // that receives these objects retains the delegate for its full lifetime.
        unsafe { manager.setDelegate(Some(delegate_object)) };
        (manager, delegate)
    })
}

impl<T: Unpin> Future for IosLocationFuture<'_, T> {
    type Output = Result<T, LocationError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            let start_result = catch_unwind(AssertUnwindSafe(|| this.start()));
            if start_result.is_err() {
                this.completion.complete(Err(internal_error()));
            }
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result.and_then(|outcome| {
                    match outcome {
                        NativeOutcome::TemporaryAccuracyRequested => this
                            .backend
                            .accuracy_authorization()
                            .map(NativeOutcome::AccuracyAuthorization)
                            .ok_or_else(unsupported_error)
                            .and_then(this.transform),
                        outcome => (this.transform)(outcome),
                    }
                }))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> Drop for IosLocationFuture<'_, T> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.completion.detach();
        if self.started && matches!(&self.operation, Operation::Current(_)) {
            if let Some(manager) = &self.manager {
                // SAFETY: A pending current operation is cancelled on the same main thread where
                // its manager was created; Core Location documents stopUpdatingLocation for this.
                unsafe { manager.stopUpdatingLocation() };
            }
        }
    }
}

/// Caller-owned, main-thread-bound one-shot Core Location backend.
pub struct IosLocationBackend {
    marker: MainThreadMarker,
}

/// The app's Core Location authorization for full or reduced accuracy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosAccuracyAuthorization {
    /// Core Location reports full-accuracy authorization.
    FullAccuracy,
    /// Core Location reports reduced-accuracy authorization.
    ReducedAccuracy,
    /// A future or unrecognized signed `NSInteger` value, preserved without loss.
    Unknown(isize),
}

/// Public Core Location activity hints for a continuous update session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosLocationActivityType {
    /// General location use.
    Other,
    /// Road-vehicle navigation.
    AutomotiveNavigation,
    /// Walking, running, cycling, or similar fitness activity.
    Fitness,
    /// Navigation that does not primarily use a road vehicle.
    OtherNavigation,
    /// Airborne activity.
    Airborne,
}

impl IosLocationActivityType {
    fn native(self) -> CLActivityType {
        match self {
            Self::Other => CLActivityType::Other,
            Self::AutomotiveNavigation => CLActivityType::AutomotiveNavigation,
            Self::Fitness => CLActivityType::Fitness,
            Self::OtherNavigation => CLActivityType::OtherNavigation,
            Self::Airborne => CLActivityType::Airborne,
        }
    }
}

/// Configuration hints for an explicitly background-enabled location session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IosBackgroundLocationOptions {
    activity_type: IosLocationActivityType,
    pauses_automatically: bool,
    shows_background_indicator: bool,
}

impl IosBackgroundLocationOptions {
    /// Creates default options: general activity, automatic pausing, no explicit indicator.
    pub const fn new() -> Self {
        Self {
            activity_type: IosLocationActivityType::Other,
            pauses_automatically: true,
            shows_background_indicator: false,
        }
    }

    /// Selects the public Core Location activity hint.
    pub const fn with_activity_type(mut self, activity_type: IosLocationActivityType) -> Self {
        self.activity_type = activity_type;
        self
    }

    /// Selects whether Core Location may automatically pause continuous updates.
    pub const fn with_automatic_pausing(mut self, enabled: bool) -> Self {
        self.pauses_automatically = enabled;
        self
    }

    /// Requests the iOS 11+ background location indicator when true.
    pub const fn with_background_indicator(mut self, enabled: bool) -> Self {
        self.shows_background_indicator = enabled;
        self
    }
}

impl Default for IosBackgroundLocationOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl IosLocationBackend {
    /// Creates the backend on the supplied main-thread/run-loop context.
    pub const fn new(marker: MainThreadMarker) -> Self {
        Self { marker }
    }

    /// Opens the caller-named persistent `CLMonitor` after a non-prompting Always preflight.
    ///
    /// The returned future starts the native open request on first poll. Core Location requires
    /// the monitor name to contain only ASCII letters and digits and not start with `_`. Only one
    /// monitor may be pending or open through this crate in a process; this avoids a second open
    /// request for a name that Core Location already owns. Existing conditions persist until
    /// explicitly removed. After app launch, reopen the same name to receive events; this API
    /// does not promise delivery, relaunch, or permission prompts.
    pub fn open_geofence_monitor(
        &mut self,
        name: &str,
    ) -> Result<crate::geofence_platform::IosGeofenceMonitorFuture, IosGeofenceError> {
        if !objc2::available!(ios = 17.0, ..) {
            return Err(IosGeofenceError::Unsupported);
        }
        geofence_authorization_preflight(authorization_from_native(native_authorization_status()))?;
        if !main_bundle_has_always_usage_descriptions() {
            return Err(IosGeofenceError::InvalidConfiguration);
        }
        crate::geofence_platform::prepare_monitor_open(name, self.marker)
    }

    /// Explicitly requests Always location authorization when the current status allows a request.
    ///
    /// The request starts only when this future is first polled. Call it from a meaningful user
    /// action; it never runs silently. `NotDetermined` and `Foreground` may request a transition,
    /// and the future completes with the first changed authorization status. A `Background`,
    /// `Denied`, `Restricted`, or `Unknown` snapshot is returned immediately without another
    /// prompt. Apple's first grant can be provisional and may later downgrade; inspect subsequent
    /// authorization changes through an active background session. Missing required usage keys
    /// return `InvalidInput` before invoking Core Location.
    pub fn request_always_authorization(&mut self) -> IosRequestAlwaysAuthorizationFuture<'_> {
        IosLocationFuture::new(
            self,
            Operation::RequestAlwaysAuthorization,
            authorization_result,
        )
    }

    /// Starts standard continuous updates with explicit background delivery enabled.
    ///
    /// The current authorization must already be `Background`; this method never prompts. The
    /// host app's main `Info.plist` must contain `UIBackgroundModes` with the exact `location`
    /// value. The backend checks that configuration before setting
    /// `allowsBackgroundLocationUpdates`, whose setter would otherwise terminate the process.
    /// A missing or malformed key returns `InvalidInput`. The configured accuracy remains a
    /// preference, not a fix guarantee. A later downgrade below Always yields a nonterminal
    /// `PermissionDenied` event and suppresses fixes until sufficient authorization returns.
    /// Start the session in the foreground when requesting continuation after the app backgrounds;
    /// iOS controls actual delivery. Dropping the session detaches its delegate and stops updates.
    pub fn start_background_updates<'a>(
        &'a mut self,
        request: LocationRequest,
        options: IosBackgroundLocationOptions,
    ) -> Result<IosLocationUpdates<'a>, LocationError> {
        start_location_updates(
            self,
            LocationUpdateService::Standard(request),
            UpdateAuthorizationRequirement::Background,
            Some(options),
        )
    }

    /// Starts the opt-in significant-change location service.
    ///
    /// The current authorization must already permit foreground location use; this method never
    /// prompts. It returns `Unsupported` when Core Location reports that the device lacks support.
    /// The service does not use the portable accuracy target or standard distance filter, and this
    /// API promises no fixed distance threshold, update cadence, background delivery, or app launch.
    /// Apple describes significant-change monitoring as a low-power service, but actual power use
    /// and update delivery remain controlled by the operating system. The session shares the
    /// bounded update queue and callback lifecycle used by standard updates; dropping it detaches
    /// the delegate and calls `stopMonitoringSignificantLocationChanges()`.
    pub fn start_significant_change_updates<'a>(
        &'a mut self,
    ) -> Result<IosLocationUpdates<'a>, LocationError> {
        start_location_updates(
            self,
            LocationUpdateService::SignificantChange,
            UpdateAuthorizationRequirement::Foreground,
            None,
        )
    }

    /// Explicitly requests temporary full-accuracy authorization for one declared purpose.
    ///
    /// Available from iOS 14.0. The request starts only when the returned future is first polled.
    /// `purpose_key` must be nonempty, contain no NUL, and match a nonempty entry under
    /// `NSLocationTemporaryUsageDescriptionDictionary` in the host app's `Info.plist`. The app
    /// must already have foreground-capable location authorization; this method never requests
    /// When In Use or Always authorization. If accuracy is already full, it returns that snapshot
    /// without prompting. An unknown accuracy value is preserved and does not trigger a request.
    /// Core Location may suppress its prompt, including while the app is in the background, and
    /// reports such a failure through the returned error. On success, the future returns a fresh
    /// accuracy snapshot. Temporary accuracy expires under Core Location's policy. Dropping the
    /// future detaches Rust interest but cannot retract an authorization prompt already handed to
    /// iOS; any late callback is contained and ignored.
    pub fn request_temporary_full_accuracy_authorization<'a>(
        &'a mut self,
        purpose_key: &str,
    ) -> Result<IosRequestTemporaryFullAccuracyAuthorizationFuture<'a>, LocationError> {
        if !objc2::available!(ios = 14.0, ..) {
            return Err(unsupported_error());
        }
        if !is_valid_temporary_accuracy_purpose_key(purpose_key) {
            return Err(configuration_error());
        }
        let purpose_key = NSString::from_str(purpose_key);
        Ok(IosLocationFuture::new(
            self,
            Operation::RequestTemporaryFullAccuracyAuthorization,
            temporary_accuracy_result,
        )
        .with_temporary_accuracy_purpose_key(purpose_key))
    }

    /// Reads Core Location's accuracy-authorization snapshot without prompting.
    ///
    /// Returns `None` on iOS versions before 14.0, where the public property is unavailable.
    /// This synchronous query creates a manager and reads its readonly `accuracyAuthorization`
    /// property on the backend's main thread; it does not request authorization or start location
    /// services. The result is separate from `LocationAuthorization`: full accuracy does not mean
    /// location authorization is granted, and this snapshot can change before a later fix.
    pub fn accuracy_authorization(&self) -> Option<IosAccuracyAuthorization> {
        current_accuracy_authorization()
    }
}

impl LocationBackend for IosLocationBackend {
    fn availability(&self) -> Availability {
        // SAFETY: This public class query has no instance ownership requirement; the backend is
        // main-thread-bound in any case.
        if unsafe { CLLocationManager::locationServicesEnabled_class() } {
            Availability::Available
        } else {
            Availability::TemporarilyUnavailable
        }
    }

    type AuthorizationFuture<'a>
        = IosAuthorizationFuture<'a>
    where
        Self: 'a;

    fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
        IosLocationFuture::new(self, Operation::Authorization, authorization_result)
    }

    type RequestAuthorizationFuture<'a>
        = IosRequestAuthorizationFuture<'a>
    where
        Self: 'a;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        IosLocationFuture::new(self, Operation::RequestAuthorization, authorization_result)
    }

    type CurrentFuture<'a>
        = IosCurrentFuture<'a>
    where
        Self: 'a;

    fn current<'a>(&'a mut self, request: LocationRequest) -> Self::CurrentFuture<'a> {
        IosLocationFuture::new(self, Operation::Current(request), current_result)
    }
}

impl<'backend> LocationUpdateSession for IosLocationUpdates<'backend> {
    type NextUpdateFuture<'session>
        = IosLocationUpdateFuture<'session, 'backend>
    where
        Self: 'session;

    fn next_update<'session>(&'session mut self) -> Self::NextUpdateFuture<'session> {
        IosLocationUpdateFuture {
            session: self,
            waiter: None,
        }
    }
}

impl LocationUpdatesBackend for IosLocationBackend {
    type UpdateSession<'a>
        = IosLocationUpdates<'a>
    where
        Self: 'a;

    fn start_updates<'a>(
        &'a mut self,
        request: LocationRequest,
    ) -> Result<Self::UpdateSession<'a>, LocationError> {
        start_location_updates(
            self,
            LocationUpdateService::Standard(request),
            UpdateAuthorizationRequirement::Foreground,
            None,
        )
    }
}

fn start_location_updates<'a>(
    backend: &'a mut IosLocationBackend,
    service: LocationUpdateService,
    requirement: UpdateAuthorizationRequirement,
    background_options: Option<IosBackgroundLocationOptions>,
) -> Result<IosLocationUpdates<'a>, LocationError> {
    let authorization = authorization_from_native(native_authorization_status());
    if matches!(service, LocationUpdateService::SignificantChange) {
        // SAFETY: This public class query is available from iOS 4.0, below the package's iOS 9.0
        // base API floor, and reports whether this device supports the significant-change service.
        let supported =
            unsafe { CLLocationManager::significantLocationChangeMonitoringAvailable() };
        significant_change_preflight(supported, authorization)?;
    } else if !updates_permitted(authorization, requirement) {
        return Err(permission_error());
    }
    match (service, requirement, background_options) {
        (LocationUpdateService::Standard(_), UpdateAuthorizationRequirement::Foreground, None) => {}
        (
            LocationUpdateService::Standard(_),
            UpdateAuthorizationRequirement::Background,
            Some(_),
        ) => {
            if !main_bundle_has_location_background_mode() {
                return Err(configuration_error());
            }
            if !objc2::available!(ios = 9.0, ..) {
                return Err(unsupported_error());
            }
        }
        (
            LocationUpdateService::SignificantChange,
            UpdateAuthorizationRequirement::Foreground,
            None,
        ) => {}
        _ => return Err(internal_error()),
    }
    if background_options.is_some_and(|options| options.shows_background_indicator)
        && !objc2::available!(ios = 11.0, ..)
    {
        return Err(unsupported_error());
    }

    let queue = Arc::new(LocationUpdateQueue::new());
    let (manager, delegate) = new_operation_objects(
        backend.marker,
        DelegateState::Updates {
            queue: queue.clone(),
            authorization: UpdateAuthorizationState::new(requirement, authorization),
        },
    );
    if let LocationUpdateService::Standard(request) = service {
        let native_accuracy = match native_accuracy(request.accuracy_target()) {
            NativeAccuracy::Best => {
                // SAFETY: Core Location declares this public accuracy constant for iOS.
                unsafe { objc2_core_location::kCLLocationAccuracyBest }
            }
            NativeAccuracy::Meters(meters) => meters,
        };
        // SAFETY: The validated portable target is finite, non-negative meters, and the manager
        // is created on this backend's main-thread/run-loop context.
        unsafe { manager.setDesiredAccuracy(native_accuracy) };
    }

    if let Some(options) = background_options {
        // SAFETY: Core Location's default is no distance threshold; set it explicitly for this
        // continuous service. The manager is main-thread-bound and its request is valid.
        unsafe { manager.setDistanceFilter(objc2_core_location::kCLDistanceFilterNone) };
        // SAFETY: The caller supplied a typed public activity hint and the manager is main-bound.
        unsafe { manager.setActivityType(options.activity_type.native()) };
        // SAFETY: The option controls only Core Location's public auto-pause policy.
        unsafe { manager.setPausesLocationUpdatesAutomatically(options.pauses_automatically) };
        if options.shows_background_indicator {
            // SAFETY: The iOS 11 runtime-availability check above precedes this setter.
            unsafe { manager.setShowsBackgroundLocationIndicator(true) };
        }
        // SAFETY: `main_bundle_has_location_background_mode()` passed before this point, current
        // authorization was checked as Always, and iOS 9 availability was checked above.
        unsafe { manager.setAllowsBackgroundLocationUpdates(true) };
    }

    // SAFETY: Authorization, device support, and any background configuration were preflighted;
    // the retained delegate receives callbacks on the manager's active main run loop.
    match service {
        LocationUpdateService::Standard(_) => unsafe { manager.startUpdatingLocation() },
        LocationUpdateService::SignificantChange => unsafe {
            manager.startMonitoringSignificantLocationChanges()
        },
    }

    let marker = backend.marker;
    Ok(IosLocationUpdates {
        _backend: backend,
        queue,
        manager,
        service,
        _delegate: delegate,
        _main_thread: marker,
    })
}

fn authorization_result(outcome: NativeOutcome) -> Result<LocationAuthorization, LocationError> {
    match outcome {
        NativeOutcome::Authorization(authorization) => Ok(authorization),
        NativeOutcome::AccuracyAuthorization(_)
        | NativeOutcome::Current(_)
        | NativeOutcome::TemporaryAccuracyRequested => Err(internal_error()),
    }
}

fn temporary_accuracy_result(
    outcome: NativeOutcome,
) -> Result<IosAccuracyAuthorization, LocationError> {
    match outcome {
        NativeOutcome::AccuracyAuthorization(authorization) => Ok(authorization),
        NativeOutcome::Authorization(_)
        | NativeOutcome::Current(_)
        | NativeOutcome::TemporaryAccuracyRequested => Err(internal_error()),
    }
}

fn current_result(outcome: NativeOutcome) -> Result<LocationFix, LocationError> {
    match outcome {
        NativeOutcome::Current(fix) => Ok(fix),
        NativeOutcome::Authorization(_)
        | NativeOutcome::AccuracyAuthorization(_)
        | NativeOutcome::TemporaryAccuracyRequested => Err(internal_error()),
    }
}

fn permission_error() -> LocationError {
    LocationError::Backend(Error::new(ErrorKind::PermissionDenied))
}

fn unsupported_error() -> LocationError {
    LocationError::Backend(Error::new(ErrorKind::Unsupported))
}

fn current_accuracy_authorization() -> Option<IosAccuracyAuthorization> {
    if !objc2::available!(ios = 14.0, ..) {
        return None;
    }
    autoreleasepool(|_| {
        // SAFETY: Callers are bound to the backend's main thread; this synchronous query installs
        // no delegate and starts no service.
        let manager = unsafe { CLLocationManager::new() };
        // SAFETY: The iOS 14 availability guard precedes the public readonly property call.
        let authorization = unsafe { manager.accuracyAuthorization() };
        Some(match authorization.0 {
            0 => IosAccuracyAuthorization::FullAccuracy,
            1 => IosAccuracyAuthorization::ReducedAccuracy,
            raw => IosAccuracyAuthorization::Unknown(raw),
        })
    })
}

fn main_bundle_has_location_background_mode() -> bool {
    let bundle = NSBundle::mainBundle();
    let Some(value) =
        bundle.objectForInfoDictionaryKey(objc2_foundation::ns_string!("UIBackgroundModes"))
    else {
        return false;
    };
    let Ok(modes) = value.downcast::<NSArray<AnyObject>>() else {
        return false;
    };
    let entries = (0..modes.len()).map(|index| {
        let entry = modes.objectAtIndex(index);
        if entry
            .downcast_ref::<NSString>()
            .is_some_and(|value| value.isEqualToString(objc2_foundation::ns_string!("location")))
        {
            BackgroundModeValue::Location
        } else {
            BackgroundModeValue::Other
        }
    });
    has_location_background_mode(Some(entries))
}

fn main_bundle_has_always_usage_descriptions() -> bool {
    has_always_usage_descriptions(
        main_bundle_has_nonempty_info_string(objc2_foundation::ns_string!(
            "NSLocationWhenInUseUsageDescription"
        )),
        main_bundle_has_nonempty_info_string(objc2_foundation::ns_string!(
            "NSLocationAlwaysAndWhenInUseUsageDescription"
        )),
        main_bundle_has_nonempty_info_string(objc2_foundation::ns_string!(
            "NSLocationAlwaysUsageDescription"
        )),
        main_bundle_requires_legacy_always_usage_description(),
    )
}

fn main_bundle_has_temporary_full_accuracy_usage_description(purpose_key: &NSString) -> bool {
    NSBundle::mainBundle()
        .objectForInfoDictionaryKey(objc2_foundation::ns_string!(
            "NSLocationTemporaryUsageDescriptionDictionary"
        ))
        .and_then(|value| value.downcast::<NSDictionary>().ok())
        .and_then(|descriptions| descriptions.objectForKey(purpose_key))
        .and_then(|description| description.downcast::<NSString>().ok())
        .is_some_and(|description| is_nonempty_string_value(true, description.length()))
}

fn main_bundle_requires_legacy_always_usage_description() -> bool {
    let bundle = NSBundle::mainBundle();
    let minimum_version = bundle
        .objectForInfoDictionaryKey(objc2_foundation::ns_string!("MinimumOSVersion"))
        .and_then(|value| value.downcast::<NSString>().ok())
        .map(|value| value.doubleValue());
    requires_legacy_always_usage_description(minimum_version)
}

fn main_bundle_has_nonempty_info_string(key: &NSString) -> bool {
    let bundle = NSBundle::mainBundle();
    bundle
        .objectForInfoDictionaryKey(key)
        .and_then(|value| value.downcast::<NSString>().ok())
        .is_some_and(|value| is_nonempty_string_value(true, value.length()))
}

#[allow(deprecated)]
fn native_authorization_status() -> i32 {
    // SAFETY: The class-level query is public and is retained for iOS 9–13 compatibility; the
    // instance replacement was added in iOS 14, while this backend's requestLocation floor is 9.
    unsafe { CLLocationManager::authorizationStatus_class().0 }
}

unsafe fn retain_callback_receiver(
    receiver: &LocationDelegate,
) -> Option<Retained<LocationDelegate>> {
    // SAFETY: objc2 invokes the delegate implementation with a live Objective-C receiver; taking
    // one retain keeps it alive through a potentially reentrant waker call.
    unsafe { Retained::retain(receiver as *const LocationDelegate as *mut LocationDelegate) }
}

unsafe fn retain_callback_manager(
    manager: &CLLocationManager,
) -> Option<Retained<CLLocationManager>> {
    // SAFETY: Core Location invokes the delegate with a live manager reference; retaining it keeps
    // the callback argument valid if a reentrant waker drops the future's manager ownership.
    unsafe { Retained::retain(manager as *const CLLocationManager as *mut CLLocationManager) }
}

fn internal_error() -> LocationError {
    LocationError::Backend(Error::new(ErrorKind::Internal))
}
