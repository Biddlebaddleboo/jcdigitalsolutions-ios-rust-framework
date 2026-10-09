use crate::conversion::{
    NativeAccuracy, authorization_from_native, error_from_native, fix_from_native, native_accuracy,
};
use crate::operation::CompletionCell;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_core::{Availability, Error, ErrorKind};
use framework_location::{
    LocationAuthorization, LocationBackend, LocationError, LocationFix, LocationRequest,
};
use objc2::rc::{Allocated, Retained, autoreleasepool};
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_location::{CLLocation, CLLocationManager, CLLocationManagerDelegate};
use objc2_foundation::{NSArray, NSDate, NSError, NSObject, NSObjectProtocol};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

type Completion = Arc<CompletionCell<NativeOutcome, LocationError>>;

#[derive(Clone, Copy)]
enum NativeOutcome {
    Authorization(LocationAuthorization),
    Current(LocationFix),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum DelegateRole {
    RequestAuthorization,
    Current,
}

struct DelegateState {
    completion: Completion,
    role: DelegateRole,
    authorization_baseline: Option<i32>,
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
            if self.ivars().role != DelegateRole::Current {
                return;
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    let Some(location) = locations.lastObject() else {
                        return Err(LocationError::Backend(Error::new(ErrorKind::Unavailable)));
                    };
                    // SAFETY: Core Location supplies a live CLLocation in the callback array;
                    // its retained last element remains valid for these property reads.
                    let coordinate = unsafe { location.coordinate() };
                    // SAFETY: The retained CLLocation is valid for the duration of this call.
                    let horizontal_accuracy = unsafe { location.horizontalAccuracy() };
                    // SAFETY: The retained CLLocation is valid for the duration of this call.
                    let timestamp: Retained<NSDate> = unsafe { location.timestamp() };
                    let timestamp_seconds = timestamp.timeIntervalSince1970();
                    fix_from_native(
                        coordinate.latitude,
                        coordinate.longitude,
                        horizontal_accuracy,
                        timestamp_seconds,
                    )
                    .map(NativeOutcome::Current)
                })
            }))
            .unwrap_or_else(|_| Err(internal_error()));
            self.ivars().completion.complete(result);
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
            if self.ivars().role != DelegateRole::Current {
                return;
            }
            let result = catch_unwind(AssertUnwindSafe(|| error_from_native(error.code() as i64)))
                .unwrap_or_else(|_| internal_error());
            self.ivars().completion.complete(Err(result));
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
    fn new(
        marker: MainThreadMarker,
        completion: Completion,
        role: DelegateRole,
        authorization_baseline: Option<i32>,
    ) -> Retained<Self> {
        let state = DelegateState {
            completion,
            role,
            authorization_baseline,
        };
        let allocated: Allocated<Self> = marker.alloc::<Self>();
        let allocated = allocated.set_ivars(state);
        // SAFETY: The class derives directly from NSObject, and its ivars are initialized before
        // calling NSObject's designated `init` implementation.
        unsafe { msg_send![super(allocated), init] }
    }

    fn complete_authorization_change(&self, raw_status: i32) {
        let ivars = self.ivars();
        if ivars.role != DelegateRole::RequestAuthorization
            || ivars.authorization_baseline == Some(raw_status)
        {
            return;
        }
        let authorization = authorization_from_native(raw_status);
        ivars
            .completion
            .complete(Ok(NativeOutcome::Authorization(authorization)));
    }
}

/// iOS non-prompting authorization-query future.
pub type IosAuthorizationFuture<'a> = IosLocationFuture<'a, LocationAuthorization>;

/// iOS foreground-authorization request future.
pub type IosRequestAuthorizationFuture<'a> = IosLocationFuture<'a, LocationAuthorization>;

/// iOS one-shot current-location future.
pub type IosCurrentFuture<'a> = IosLocationFuture<'a, LocationFix>;

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
        let (manager, delegate) =
            self.new_operation_objects(DelegateRole::RequestAuthorization, Some(status));
        self.manager = Some(manager);
        self.delegate = Some(delegate);
        let manager = self.manager.as_ref().expect("manager set above");
        // SAFETY: The backend marker keeps this future on the main thread and the manager was
        // created on that thread with its callback delegate retained by this future.
        unsafe { manager.requestWhenInUseAuthorization() };
    }

    fn start_current(&mut self, request: LocationRequest) {
        let (manager, delegate) = self.new_operation_objects(DelegateRole::Current, None);
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

    fn new_operation_objects(
        &self,
        role: DelegateRole,
        authorization_baseline: Option<i32>,
    ) -> (Retained<CLLocationManager>, Retained<LocationDelegate>) {
        autoreleasepool(|_| {
            // SAFETY: CLLocationManager's initializer is public and this backend supplies the
            // main-thread/run-loop context required for its delegate callbacks.
            let manager = unsafe { CLLocationManager::new() };
            let delegate = LocationDelegate::new(
                self.backend.marker,
                self.completion.clone(),
                role,
                authorization_baseline,
            );
            let delegate_object = ProtocolObject::from_ref(&*delegate);
            // SAFETY: CLLocationManager's delegate is a zero-ownership weak property; this future
            // retains the delegate until it is finished or dropped.
            unsafe { manager.setDelegate(Some(delegate_object)) };
            (manager, delegate)
        })
    }
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

impl IosLocationBackend {
    /// Creates the backend on the supplied main-thread/run-loop context.
    pub const fn new(marker: MainThreadMarker) -> Self {
        Self { marker }
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
