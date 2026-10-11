use crate::conversion::{
    NativeAccuracy, authorization_from_native, error_from_native, fix_from_native, native_accuracy,
};
use crate::operation::{CompletionCell, LocationUpdateQueue};
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
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_location::{CLLocation, CLLocationManager, CLLocationManagerDelegate};
use objc2_foundation::{NSArray, NSDate, NSError, NSObject, NSObjectProtocol};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::task::Waker;

type Completion = Arc<CompletionCell<NativeOutcome, LocationError>>;

#[derive(Clone, Copy)]
enum NativeOutcome {
    Authorization(LocationAuthorization),
    Current(LocationFix),
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
    },
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
                DelegateState::Updates { queue } => {
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
                DelegateState::Updates { queue } => {
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
            DelegateState::Updates { queue }
                if !authorization_from_native(raw_status).allows_foreground() =>
            {
                queue.push_batch([LocationUpdateKind::Error(permission_error())]);
            }
            DelegateState::RequestAuthorization { .. }
            | DelegateState::Current { .. }
            | DelegateState::Updates { .. } => {}
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

/// iOS one-shot current-location future.
pub type IosCurrentFuture<'a> = IosLocationFuture<'a, LocationFix>;

/// A foreground location-update session that owns its native manager and retained delegate.
///
/// The session exclusively borrows its backend and must be used and dropped on the backend's
/// main thread. It starts only after current authorization permits foreground location; it never
/// requests permission or enables background updates. Dropping the session detaches callback
/// state and stops the native location service.
pub struct IosLocationUpdates<'a> {
    _backend: &'a mut IosLocationBackend,
    queue: Arc<LocationUpdateQueue>,
    manager: Retained<CLLocationManager>,
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
        // SAFETY: The manager started standard updates on this thread and may still be active.
        unsafe { self.manager.stopUpdatingLocation() };
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
            transform,
            started: false,
            finished: false,
            _main_thread: marker,
            _output: PhantomData,
        }
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
                Poll::Ready(result.and_then(this.transform))
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
        if self.started && matches!(self.operation, Operation::Current(_)) {
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

impl IosLocationBackend {
    /// Creates the backend on the supplied main-thread/run-loop context.
    pub const fn new(marker: MainThreadMarker) -> Self {
        Self { marker }
    }

    /// Reads Core Location's accuracy-authorization snapshot without prompting.
    ///
    /// Returns `None` on iOS versions before 14.0, where the public property is unavailable.
    /// This synchronous query creates a manager and reads its readonly `accuracyAuthorization`
    /// property on the backend's main thread; it does not request authorization or start location
    /// services. The result is separate from `LocationAuthorization`: full accuracy does not mean
    /// location authorization is granted, and this snapshot can change before a later fix.
    pub fn accuracy_authorization(&self) -> Option<IosAccuracyAuthorization> {
        if !objc2::available!(ios = 14.0, ..) {
            return None;
        }
        autoreleasepool(|_| {
            // SAFETY: The backend is constructed with a MainThreadMarker and is main-thread-bound.
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
        if !authorization_from_native(native_authorization_status()).allows_foreground() {
            return Err(permission_error());
        }

        let queue = Arc::new(LocationUpdateQueue::new());
        let (manager, delegate) = new_operation_objects(
            self.marker,
            DelegateState::Updates {
                queue: queue.clone(),
            },
        );
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
        // SAFETY: Current authorization was checked above; the public standard service is started
        // on the manager's creation thread with a strongly retained delegate and active run loop.
        unsafe { manager.startUpdatingLocation() };

        let marker = self.marker;
        Ok(IosLocationUpdates {
            _backend: self,
            queue,
            manager,
            _delegate: delegate,
            _main_thread: marker,
        })
    }
}

fn authorization_result(outcome: NativeOutcome) -> Result<LocationAuthorization, LocationError> {
    match outcome {
        NativeOutcome::Authorization(authorization) => Ok(authorization),
        NativeOutcome::Current(_) => Err(internal_error()),
    }
}

fn current_result(outcome: NativeOutcome) -> Result<LocationFix, LocationError> {
    match outcome {
        NativeOutcome::Current(fix) => Ok(fix),
        NativeOutcome::Authorization(_) => Err(internal_error()),
    }
}

fn permission_error() -> LocationError {
    LocationError::Backend(Error::new(ErrorKind::PermissionDenied))
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
