#[cfg(target_os = "ios")]
use alloc::boxed::Box;
#[cfg(target_os = "ios")]
use alloc::sync::Arc;
#[cfg(target_os = "ios")]
use alloc::task::Wake;
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use core::future::Future;
use core::mem::{align_of, size_of};
use core::panic::AssertUnwindSafe;
#[cfg(target_os = "ios")]
use core::pin::Pin;
use core::ptr;
#[cfg(target_os = "ios")]
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
#[cfg(target_os = "ios")]
use core::task::{Context, Poll, Waker};
use framework_abi::{ABI_VERSION_MAJOR, FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use ::ios_location::IosLocationBackend;
#[cfg(target_os = "ios")]
use framework_core::Availability;
#[cfg(target_os = "ios")]
use framework_location::{
    AccuracyTarget, LocationAuthorization, LocationBackend, LocationError, LocationFix,
    LocationRequest,
};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;

/// Fixed-width location availability tag.
pub type FrameworkIosLocationAvailability = u32;
/// Availability is not known to this ABI version.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN: FrameworkIosLocationAvailability = 0;
/// Location services are enabled.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_AVAILABLE: FrameworkIosLocationAvailability = 1;
/// This target does not support the operation.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNSUPPORTED: FrameworkIosLocationAvailability = 2;
/// User authorization is required.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_PERMISSION:
    FrameworkIosLocationAvailability = 3;
/// A platform entitlement is required.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_ENTITLEMENT:
    FrameworkIosLocationAvailability = 4;
/// Location services are temporarily unavailable.
pub const FRAMEWORK_IOS_LOCATION_AVAILABILITY_TEMPORARILY_UNAVAILABLE:
    FrameworkIosLocationAvailability = 5;

/// Fixed-width location-authorization tag.
pub type FrameworkIosLocationAuthorization = u32;
/// Authorization is not known to this ABI version.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN: FrameworkIosLocationAuthorization = 0;
/// The user has not yet chosen an authorization state.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_NOT_DETERMINED: FrameworkIosLocationAuthorization =
    1;
/// The user denied location authorization.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_DENIED: FrameworkIosLocationAuthorization = 2;
/// A policy or device restriction blocks location authorization.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_RESTRICTED: FrameworkIosLocationAuthorization = 3;
/// Foreground location use is authorized.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_FOREGROUND: FrameworkIosLocationAuthorization = 4;
/// Background authorization is reported; this ABI exposes no background operation.
pub const FRAMEWORK_IOS_LOCATION_AUTHORIZATION_BACKGROUND: FrameworkIosLocationAuthorization = 5;

/// Fixed-width operation-kind tag.
pub type FrameworkIosLocationOperationKind = u32;
/// Non-prompting authorization query operation.
pub const FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_QUERY: FrameworkIosLocationOperationKind =
    0;
/// Explicit foreground-authorization request operation.
pub const FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_REQUEST:
    FrameworkIosLocationOperationKind = 1;
/// One-shot current-location operation.
pub const FRAMEWORK_IOS_LOCATION_OPERATION_CURRENT: FrameworkIosLocationOperationKind = 2;

/// One C readiness notification for an accepted location operation.
pub type FrameworkIosLocationReady = Option<unsafe extern "C" fn(context: *mut c_void)>;

/// One fully overwritten result from an authorization or current-location operation.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkIosLocationResultV1 {
    /// Exact byte size of this record.
    pub struct_size: u32,
    /// ABI major version from `framework_abi_version`.
    pub abi_version: u32,
    /// Operation-kind tag.
    pub operation_kind: FrameworkIosLocationOperationKind,
    /// Terminal operation status; inspect only when `out_ready` is 1.
    pub status: FrameworkStatus,
    /// Authorization tag for authorization operations; otherwise unknown.
    pub authorization: FrameworkIosLocationAuthorization,
    /// Reserved; always zero.
    pub reserved: u32,
    /// WGS-84 latitude in degrees for a successful current-location result; otherwise zero.
    pub latitude_degrees: f64,
    /// WGS-84 longitude in degrees for a successful current-location result; otherwise zero.
    pub longitude_degrees: f64,
    /// Backend-reported horizontal-accuracy estimate in meters; otherwise zero.
    pub horizontal_accuracy_meters: f64,
    /// Measurement time in Unix milliseconds; otherwise zero.
    pub timestamp_unix_millis: u64,
    /// Optional native error code; zero when no code is available.
    pub native_code: i32,
    /// Reserved; always zero.
    pub reserved2: u32,
}

/// One unique operation handle; all operations and destruction are main-thread-only on iOS.
pub struct FrameworkIosLocationOperation {
    #[cfg(target_os = "ios")]
    kind: FrameworkIosLocationOperationKind,
    #[cfg(target_os = "ios")]
    future: Option<LocationOperationFuture>,
    #[cfg(target_os = "ios")]
    result: Option<LocationOperationValue>,
    #[cfg(target_os = "ios")]
    cancelled: bool,
    #[cfg(target_os = "ios")]
    consumed: bool,
    #[cfg(target_os = "ios")]
    signal: Arc<ReadySignal>,
}

#[cfg(target_os = "ios")]
#[derive(Clone, Copy)]
enum LocationOperationValue {
    Authorization(Result<LocationAuthorization, LocationError>),
    Current(Result<LocationFix, LocationError>),
}

#[cfg(target_os = "ios")]
type LocationOperationFuture = Pin<Box<dyn Future<Output = LocationOperationValue>>>;

#[cfg(target_os = "ios")]
struct ReadySignal {
    callback: unsafe extern "C" fn(context: *mut c_void),
    context: AtomicPtr<c_void>,
    active: AtomicBool,
    pending: AtomicBool,
    notified: AtomicBool,
    polling: AtomicBool,
    deferred: AtomicBool,
}

#[cfg(target_os = "ios")]
impl ReadySignal {
    fn new(callback: unsafe extern "C" fn(context: *mut c_void), context: *mut c_void) -> Self {
        Self {
            callback,
            context: AtomicPtr::new(context),
            active: AtomicBool::new(false),
            pending: AtomicBool::new(false),
            notified: AtomicBool::new(false),
            polling: AtomicBool::new(false),
            deferred: AtomicBool::new(false),
        }
    }

    fn activate(&self) {
        self.active.store(true, Ordering::Release);
        if self.pending.swap(false, Ordering::AcqRel) {
            self.notify_once();
        }
    }

    fn signal(&self) {
        if self.polling.load(Ordering::Acquire) {
            self.deferred.store(true, Ordering::Release);
            return;
        }
        if self.active.load(Ordering::Acquire) {
            self.notify_once();
        } else {
            self.pending.store(true, Ordering::Release);
            if self.active.load(Ordering::Acquire) && self.pending.swap(false, Ordering::AcqRel) {
                self.notify_once();
            }
        }
    }

    fn begin_poll(&self) {
        self.polling.store(true, Ordering::Release);
    }

    fn end_poll(&self, result_ready: bool) {
        self.polling.store(false, Ordering::Release);
        if result_ready {
            self.deferred.store(false, Ordering::Release);
            self.pending.store(false, Ordering::Release);
            self.notified.store(true, Ordering::Release);
        } else if self.deferred.swap(false, Ordering::AcqRel) {
            self.notify_once();
        }
    }

    fn notify_once(&self) {
        if self.active.load(Ordering::Acquire) && !self.notified.swap(true, Ordering::AcqRel) {
            let context = self.context.swap(ptr::null_mut(), Ordering::AcqRel);
            // SAFETY: The host promises a live callback and context through this notification and
            // that the callback does not unwind or re-enter this ABI. B5 wakes on the manager's
            // main run loop, and every C entry point requires that same main thread.
            unsafe { (self.callback)(context) };
        }
    }

    fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
        self.context.store(ptr::null_mut(), Ordering::Release);
    }
}

#[cfg(target_os = "ios")]
impl Wake for ReadySignal {
    fn wake(self: Arc<Self>) {
        self.signal();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.signal();
    }
}

#[cfg(target_os = "ios")]
impl FrameworkIosLocationOperation {
    fn poll_future(&mut self) {
        if self.cancelled || self.consumed || self.result.is_some() {
            return;
        }
        let Some(future) = self.future.as_mut() else {
            return;
        };
        let waker = Waker::from(self.signal.clone());
        let mut context = Context::from_waker(&waker);
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            self.future.take();
            self.result = Some(result);
        }
    }
}

#[cfg(target_os = "ios")]
impl Drop for FrameworkIosLocationOperation {
    fn drop(&mut self) {
        self.signal.deactivate();
        self.future.take();
    }
}

fn empty_result(operation_kind: FrameworkIosLocationOperationKind) -> FrameworkIosLocationResultV1 {
    FrameworkIosLocationResultV1 {
        struct_size: size_of::<FrameworkIosLocationResultV1>() as u32,
        abi_version: ABI_VERSION_MAJOR,
        operation_kind,
        status: FrameworkStatus::OK,
        authorization: FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN,
        reserved: 0,
        latitude_degrees: 0.0,
        longitude_degrees: 0.0,
        horizontal_accuracy_meters: 0.0,
        timestamp_unix_millis: 0,
        native_code: 0,
        reserved2: 0,
    }
}

fn pointer_aligned<T>(pointer: *const T) -> bool {
    (pointer as usize) % align_of::<T>() == 0
}

fn byte_ranges_overlap(
    first: *const u8,
    first_length: usize,
    second: *const u8,
    second_length: usize,
) -> bool {
    let Some(first_end) = (first as usize).checked_add(first_length) else {
        return true;
    };
    let Some(second_end) = (second as usize).checked_add(second_length) else {
        return true;
    };
    (first as usize) < second_end && (second as usize) < first_end
}

#[cfg(target_os = "ios")]
fn availability_tag(value: Availability) -> FrameworkIosLocationAvailability {
    match value {
        Availability::Unknown => FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN,
        Availability::Available => FRAMEWORK_IOS_LOCATION_AVAILABILITY_AVAILABLE,
        Availability::Unsupported => FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNSUPPORTED,
        Availability::RequiresPermission => FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_PERMISSION,
        Availability::RequiresEntitlement => {
            FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_ENTITLEMENT
        }
        Availability::TemporarilyUnavailable => {
            FRAMEWORK_IOS_LOCATION_AVAILABILITY_TEMPORARILY_UNAVAILABLE
        }
        _ => FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn authorization_tag(value: LocationAuthorization) -> FrameworkIosLocationAuthorization {
    match value {
        LocationAuthorization::Unknown => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN,
        LocationAuthorization::NotDetermined => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_NOT_DETERMINED,
        LocationAuthorization::Denied => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_DENIED,
        LocationAuthorization::Restricted => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_RESTRICTED,
        LocationAuthorization::Foreground => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_FOREGROUND,
        LocationAuthorization::Background => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_BACKGROUND,
        _ => FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn error_status(value: LocationError) -> FrameworkStatus {
    match value {
        LocationError::InvalidCoordinate | LocationError::InvalidAccuracy => {
            FrameworkStatus::INVALID_ARGUMENT
        }
        LocationError::Backend(error) => FrameworkStatus::from_error(error),
        _ => FrameworkStatus::INTERNAL_ERROR,
    }
}

#[cfg(target_os = "ios")]
fn native_code(value: LocationError) -> i32 {
    value.platform_code().map_or(0, |code| code.get())
}

#[cfg(target_os = "ios")]
fn result_record(
    operation_kind: FrameworkIosLocationOperationKind,
    value: LocationOperationValue,
) -> FrameworkIosLocationResultV1 {
    let mut result = empty_result(operation_kind);
    match value {
        LocationOperationValue::Authorization(Ok(authorization)) => {
            result.authorization = authorization_tag(authorization);
        }
        LocationOperationValue::Authorization(Err(error)) => {
            result.status = error_status(error);
            result.native_code = native_code(error);
        }
        LocationOperationValue::Current(Ok(fix)) => {
            let coordinate = fix.coordinate();
            result.latitude_degrees = coordinate.latitude_degrees();
            result.longitude_degrees = coordinate.longitude_degrees();
            result.horizontal_accuracy_meters = fix.horizontal_accuracy_meters();
            result.timestamp_unix_millis = fix.timestamp().unix_millis();
        }
        LocationOperationValue::Current(Err(error)) => {
            result.status = error_status(error);
            result.native_code = native_code(error);
        }
    }
    result
}

#[cfg(target_os = "ios")]
fn begin_operation(
    operation_kind: FrameworkIosLocationOperationKind,
    future: LocationOperationFuture,
    callback: unsafe extern "C" fn(context: *mut c_void),
    context: *mut c_void,
    out_operation: *mut *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    let signal = Arc::new(ReadySignal::new(callback, context));
    let mut operation = Box::new(FrameworkIosLocationOperation {
        kind: operation_kind,
        future: Some(future),
        result: None,
        cancelled: false,
        consumed: false,
        signal: signal.clone(),
    });
    operation.poll_future();
    let ready = operation.result.is_some();
    let raw = Box::into_raw(operation);
    // SAFETY: The exported start function validated and initialized this unique writable slot.
    unsafe { out_operation.write(raw) };
    signal.activate();
    if ready {
        signal.signal();
    }
    FrameworkStatus::OK
}

/// Reads point-in-time Core Location availability without requesting authorization.
///
/// # Safety
/// `out_availability` must point to aligned writable `uint32_t` storage that does not overlap
/// a live location-operation handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_availability(
    out_availability: *mut FrameworkIosLocationAvailability,
) -> FrameworkStatus {
    if out_availability.is_null() || !pointer_aligned(out_availability) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises aligned writable output storage.
    unsafe { out_availability.write(FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let backend = IosLocationBackend::new(marker.into_objc2());
            // SAFETY: The output remains writable and no handle is read or changed here.
            unsafe { out_availability.write(availability_tag(backend.availability())) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            // SAFETY: The caller supplied writable output storage.
            unsafe { out_availability.write(FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNSUPPORTED) };
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Starts a non-prompting authorization query and returns one operation handle.
///
/// # Safety
/// `completion` must be non-null and must not unwind or re-enter this ABI. `out_operation` must
/// point to an aligned writable handle slot that is not live on entry and does not overlap a live
/// operation handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_authorization_query_start(
    completion: FrameworkIosLocationReady,
    context: *mut c_void,
    out_operation: *mut *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    if out_operation.is_null() || !pointer_aligned(out_operation) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises an aligned writable handle slot.
    unsafe { out_operation.write(ptr::null_mut()) };
    let Some(completion) = completion else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let future = Box::pin(async move {
                let mut backend = IosLocationBackend::new(marker.into_objc2());
                LocationOperationValue::Authorization(backend.authorization().await)
            });
            begin_operation(
                FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_QUERY,
                future,
                completion,
                context,
                out_operation,
            )
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (completion, context);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Explicitly starts a foreground authorization request; this may display Apple's permission UI.
///
/// # Safety
/// `completion` must be non-null and must not unwind or re-enter this ABI. `out_operation` must
/// point to an aligned writable handle slot that is not live on entry and does not overlap a live
/// operation handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_authorization_request_start(
    completion: FrameworkIosLocationReady,
    context: *mut c_void,
    out_operation: *mut *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    if out_operation.is_null() || !pointer_aligned(out_operation) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises an aligned writable handle slot.
    unsafe { out_operation.write(ptr::null_mut()) };
    let Some(completion) = completion else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let future = Box::pin(async move {
                let mut backend = IosLocationBackend::new(marker.into_objc2());
                LocationOperationValue::Authorization(backend.request_authorization().await)
            });
            begin_operation(
                FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_REQUEST,
                future,
                completion,
                context,
                out_operation,
            )
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (completion, context);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Starts one foreground current-location request with a desired accuracy in meters.
///
/// # Safety
/// `completion` must be non-null and must not unwind or re-enter this ABI. `out_operation` must
/// point to an aligned writable handle slot that is not live on entry and does not overlap a live
/// operation handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_current_start(
    accuracy_target_meters: f64,
    completion: FrameworkIosLocationReady,
    context: *mut c_void,
    out_operation: *mut *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    if out_operation.is_null() || !pointer_aligned(out_operation) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises an aligned writable handle slot.
    unsafe { out_operation.write(ptr::null_mut()) };
    let Some(completion) = completion else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if !accuracy_target_meters.is_finite() || accuracy_target_meters < 0.0 {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let Ok(accuracy) = AccuracyTarget::new(accuracy_target_meters) else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            let request = LocationRequest::new(accuracy);
            let future = Box::pin(async move {
                let mut backend = IosLocationBackend::new(marker.into_objc2());
                LocationOperationValue::Current(backend.current(request).await)
            });
            begin_operation(
                FRAMEWORK_IOS_LOCATION_OPERATION_CURRENT,
                future,
                completion,
                context,
                out_operation,
            )
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (completion, context);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Polls one operation and consumes its terminal result at most once.
///
/// # Safety
/// On iOS, `operation` must be a live unique handle used only on the main thread. Both outputs
/// must be naturally aligned, writable, disjoint from each other, and disjoint from the handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_operation_poll(
    operation: *mut FrameworkIosLocationOperation,
    out_ready: *mut u8,
    out_result: *mut FrameworkIosLocationResultV1,
) -> FrameworkStatus {
    let ready_valid = !out_ready.is_null() && pointer_aligned(out_ready);
    let result_valid = !out_result.is_null() && pointer_aligned(out_result);
    if ready_valid {
        // SAFETY: The caller promises writable output storage.
        unsafe { out_ready.write(0) };
    }
    if result_valid {
        // SAFETY: The caller promises writable output storage.
        unsafe { out_result.write(empty_result(0)) };
    }
    if !ready_valid || !result_valid {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if byte_ranges_overlap(
        out_ready.cast(),
        size_of::<u8>(),
        out_result.cast(),
        size_of::<FrameworkIosLocationResultV1>(),
    ) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        if operation.is_null() || !pointer_aligned(operation) {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if byte_ranges_overlap(
                operation.cast(),
                size_of::<FrameworkIosLocationOperation>(),
                out_ready.cast(),
                size_of::<u8>(),
            ) || byte_ranges_overlap(
                operation.cast(),
                size_of::<FrameworkIosLocationOperation>(),
                out_result.cast(),
                size_of::<FrameworkIosLocationResultV1>(),
            ) {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller promises a live unique operation handle and no concurrent call.
            let operation = unsafe { &mut *operation };
            // SAFETY: The caller supplied aligned writable output storage.
            unsafe { out_result.write(empty_result(operation.kind)) };
            if operation.consumed {
                return FrameworkStatus::NOT_FOUND;
            }
            if operation.cancelled {
                let mut result = empty_result(operation.kind);
                result.status = FrameworkStatus::CANCELLED;
                operation.consumed = true;
                // SAFETY: The caller supplied aligned writable output storage.
                unsafe {
                    out_result.write(result);
                    out_ready.write(1);
                }
                return FrameworkStatus::OK;
            }
            operation.signal.begin_poll();
            operation.poll_future();
            operation.signal.end_poll(operation.result.is_some());
            if let Some(value) = operation.result.take() {
                let result = result_record(operation.kind, value);
                operation.consumed = true;
                operation.signal.notify_once();
                // SAFETY: The caller supplied aligned writable output storage.
                unsafe {
                    out_result.write(result);
                    out_ready.write(1);
                }
            }
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Cancels a pending operation by dropping its future on the main thread.
///
/// # Safety
/// On iOS, `operation` must be a live unique handle used only on the main thread. Do not call
/// while the operation's readiness callback is active.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_operation_cancel(
    operation: *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if operation.is_null() || !pointer_aligned(operation) {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller promises one live unique handle on the main thread.
            let operation = unsafe { &mut *operation };
            if operation.cancelled
                || operation.consumed
                || operation.result.is_some()
                || operation.signal.notified.load(Ordering::Acquire)
            {
                return FrameworkStatus::NOT_FOUND;
            }
            operation.signal.deactivate();
            operation.future.take();
            operation.cancelled = true;
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            if operation.is_null() || !pointer_aligned(operation) {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Destroys an operation and clears its original handle slot.
///
/// # Safety
/// `operation` must be null or the original aligned writable slot returned by a start function.
/// On iOS, call on the main thread; an off-main call returns unavailable without reading or
/// changing the slot. Do not copy the handle, destroy an alias, or race any operation with destroy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_location_operation_destroy(
    operation: *mut *mut FrameworkIosLocationOperation,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        if MainThread::current().is_none() {
            return FrameworkStatus::UNAVAILABLE;
        }
        if operation.is_null() {
            return FrameworkStatus::OK;
        }
        if !pointer_aligned(operation) {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        // SAFETY: The caller supplies the original writable pointer slot.
        let value = unsafe { operation.read() };
        if value.is_null() {
            return FrameworkStatus::OK;
        }
        #[cfg(target_os = "ios")]
        {
            // SAFETY: Clear the unique original slot before dropping the operation on main.
            unsafe { operation.write(ptr::null_mut()) };
            // SAFETY: A successful start returned this unique Box allocation.
            unsafe { drop(Box::from_raw(value)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
