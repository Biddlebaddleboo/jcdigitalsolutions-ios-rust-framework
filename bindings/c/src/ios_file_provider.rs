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
use core::task::{Context, Poll, Waker};
use framework_abi::{
    ABI_VERSION_MAJOR, FrameworkOwnedBuffer, FrameworkStatus, catch_unwind_status,
};
#[cfg(target_os = "ios")]
use std::sync::{Condvar, Mutex, MutexGuard};

#[cfg(target_os = "ios")]
use ::ios_file_provider::{FileProviderQueryError, RegisteredDomainPresence};

/// One C readiness notification for an accepted registered-domain query.
pub type FrameworkIosFileProviderReady = Option<unsafe extern "C" fn(context: *mut c_void)>;

/// One fully overwritten result from a File Provider registered-domain query.
#[repr(C)]
pub struct FrameworkIosFileProviderResultV1 {
    /// Exact byte size of this record.
    pub struct_size: u32,
    /// ABI major version from `framework_abi_version`.
    pub abi_version: u32,
    /// Terminal query status; inspect only when `out_ready` is 1.
    pub status: FrameworkStatus,
    /// One when the app's own provider has registered domains; otherwise zero.
    pub has_registered_domains: u8,
    /// Reserved; always zero.
    pub reserved: [u8; 3],
    /// Owned UTF-8 bytes for a native NSError domain; destroy this field with
    /// `framework_owned_buffer_destroy`.
    pub native_error_domain: FrameworkOwnedBuffer,
    /// Exact native NSError NSInteger code when `status` is PLATFORM_ERROR; otherwise zero.
    pub native_error_code: i64,
}

/// One unique operation handle; serialize calls on this handle and do not use it from its callback.
pub struct FrameworkIosFileProviderOperation {
    #[cfg(target_os = "ios")]
    future: Option<FileProviderFuture>,
    #[cfg(target_os = "ios")]
    result: Option<Result<RegisteredDomainPresence, FileProviderQueryError>>,
    #[cfg(target_os = "ios")]
    signal: Arc<ReadinessSignal>,
    #[cfg(target_os = "ios")]
    consumed: bool,
}

#[cfg(target_os = "ios")]
type FileProviderFuture =
    Pin<Box<dyn Future<Output = Result<RegisteredDomainPresence, FileProviderQueryError>> + Send>>;

#[cfg(target_os = "ios")]
struct ReadinessState {
    published: bool,
    closed: bool,
    pending: bool,
    notified: bool,
    in_flight: usize,
    callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
    context: usize,
}

#[cfg(target_os = "ios")]
struct ReadinessSignal {
    state: Mutex<ReadinessState>,
    callback_finished: Condvar,
}

#[cfg(target_os = "ios")]
impl ReadinessSignal {
    fn new(callback: unsafe extern "C" fn(context: *mut c_void), context: *mut c_void) -> Self {
        Self {
            state: Mutex::new(ReadinessState {
                published: false,
                closed: false,
                pending: false,
                notified: false,
                in_flight: 0,
                callback: Some(callback),
                context: context as usize,
            }),
            callback_finished: Condvar::new(),
        }
    }

    fn activate(&self) {
        let invocation = {
            let mut state = self.lock();
            if state.closed {
                return;
            }
            state.published = true;
            if state.pending {
                state.pending = false;
                Self::reserve_callback(&mut state)
            } else {
                None
            }
        };
        self.invoke(invocation);
    }

    fn signal(&self) {
        let invocation = {
            let mut state = self.lock();
            if state.closed || state.notified {
                return;
            }
            if !state.published {
                state.pending = true;
                return;
            }
            Self::reserve_callback(&mut state)
        };
        self.invoke(invocation);
    }

    fn close_and_wait(&self) {
        let mut state = self.lock();
        state.closed = true;
        state.pending = false;
        state.callback = None;
        state.context = 0;
        while state.in_flight != 0 {
            state = self
                .callback_finished
                .wait(state)
                .unwrap_or_else(|poison| poison.into_inner());
        }
    }

    fn reserve_callback(
        state: &mut ReadinessState,
    ) -> Option<(unsafe extern "C" fn(context: *mut c_void), usize)> {
        if state.closed || state.notified {
            return None;
        }
        let callback = state.callback?;
        state.notified = true;
        state.in_flight += 1;
        Some((callback, state.context))
    }

    fn invoke(&self, invocation: Option<(unsafe extern "C" fn(*mut c_void), usize)>) {
        let Some((callback, context)) = invocation else {
            return;
        };
        // SAFETY: The host promises callback/context validity through callback return, no unwind,
        // and no F16 API re-entry. `close_and_wait` waits for this invocation before destroy returns.
        unsafe { callback(context as *mut c_void) };
        let mut state = self.lock();
        state.in_flight -= 1;
        if state.in_flight == 0 {
            state.callback = None;
            state.context = 0;
            self.callback_finished.notify_all();
        }
    }

    fn lock(&self) -> MutexGuard<'_, ReadinessState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(target_os = "ios")]
impl Wake for ReadinessSignal {
    fn wake(self: Arc<Self>) {
        self.signal();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.signal();
    }
}

#[cfg(target_os = "ios")]
impl FrameworkIosFileProviderOperation {
    fn poll_future(&mut self) {
        if self.result.is_some() {
            return;
        }
        let Some(future) = self.future.as_mut() else {
            return;
        };
        let waker = Waker::from(Arc::clone(&self.signal));
        let mut context = Context::from_waker(&waker);
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            self.future.take();
            self.result = Some(result);
        }
    }
}

#[cfg(target_os = "ios")]
impl Drop for FrameworkIosFileProviderOperation {
    fn drop(&mut self) {
        self.signal.close_and_wait();
        self.future.take();
    }
}

fn empty_result() -> FrameworkIosFileProviderResultV1 {
    FrameworkIosFileProviderResultV1 {
        struct_size: size_of::<FrameworkIosFileProviderResultV1>() as u32,
        abi_version: ABI_VERSION_MAJOR,
        status: FrameworkStatus::OK,
        has_registered_domains: 0,
        reserved: [0; 3],
        native_error_domain: FrameworkOwnedBuffer::default(),
        native_error_code: 0,
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
fn result_record(
    value: &Result<RegisteredDomainPresence, FileProviderQueryError>,
) -> FrameworkIosFileProviderResultV1 {
    let mut result = empty_result();
    match value {
        Ok(presence) => {
            result.has_registered_domains = u8::from(presence.has_registered_domains());
        }
        Err(FileProviderQueryError::UnsupportedPlatform) => {
            result.status = FrameworkStatus::UNSUPPORTED;
        }
        Err(FileProviderQueryError::ApiUnavailable) => {
            result.status = FrameworkStatus::UNAVAILABLE;
        }
        Err(FileProviderQueryError::CountOutOfRange) => {
            result.status = FrameworkStatus::INTERNAL_ERROR;
        }
        Err(FileProviderQueryError::CallbackPanicked) => {
            result.status = FrameworkStatus::PANIC;
        }
        Err(FileProviderQueryError::Native(error)) => {
            result.status = FrameworkStatus::PLATFORM_ERROR;
            result.native_error_code = error.code() as i64;
            let bytes = error.domain().as_bytes().to_vec();
            match FrameworkOwnedBuffer::try_from_vec(bytes) {
                Ok(domain) => result.native_error_domain = domain,
                Err(_) => result.status = FrameworkStatus::RESOURCE_EXHAUSTED,
            }
        }
    }
    result
}

/// Starts the app's own registered-domain presence query and returns one opaque operation handle.
///
/// The Apple request starts during this call and has no cancellation API. The readiness callback
/// may run inline before this function returns or later on Apple's unspecified callback queue.
///
/// # Safety
/// `completion` must be non-null and must not unwind or re-enter F16. `out_operation` must point
/// to an aligned writable handle slot that is not live on entry.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_file_provider_registered_domain_presence_start(
    completion: FrameworkIosFileProviderReady,
    context: *mut c_void,
    out_operation: *mut *mut FrameworkIosFileProviderOperation,
) -> FrameworkStatus {
    if out_operation.is_null() || !pointer_aligned(out_operation) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises an aligned writable handle slot.
    unsafe { out_operation.write(ptr::null_mut()) };
    let Some(completion) = completion else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    #[cfg(target_os = "ios")]
    {
        catch_unwind_status(AssertUnwindSafe(|| {
            let signal = Arc::new(ReadinessSignal::new(completion, context));
            let future: FileProviderFuture =
                Box::pin(::ios_file_provider::request_registered_domain_presence());
            let mut operation = Box::new(FrameworkIosFileProviderOperation {
                future: Some(future),
                result: None,
                signal: Arc::clone(&signal),
                consumed: false,
            });
            operation.poll_future();
            let ready = operation.result.is_some();
            let raw = Box::into_raw(operation);
            // SAFETY: This start validated and initialized the unique output slot.
            unsafe { out_operation.write(raw) };
            signal.activate();
            if ready {
                signal.signal();
            }
            FrameworkStatus::OK
        }))
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (completion, context);
        FrameworkStatus::UNSUPPORTED
    }
}

/// Polls one operation and consumes its terminal result at most once.
///
/// # Safety
/// On iOS, `operation` must be a live unique handle. Calls on one handle must be serialized and
/// must not occur from its readiness callback. Both outputs must be aligned, writable, disjoint
/// from each other, and disjoint from the handle; `out_result` must contain no live owned buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_file_provider_operation_poll(
    operation: *mut FrameworkIosFileProviderOperation,
    out_ready: *mut u8,
    out_result: *mut FrameworkIosFileProviderResultV1,
) -> FrameworkStatus {
    let ready_valid = !out_ready.is_null() && pointer_aligned(out_ready);
    let result_valid = !out_result.is_null() && pointer_aligned(out_result);
    if ready_valid {
        // SAFETY: The caller promises writable output storage.
        unsafe { out_ready.write(0) };
    }
    if result_valid {
        // SAFETY: The caller promises an empty output record and writable storage.
        unsafe { out_result.write(empty_result()) };
    }
    if !ready_valid || !result_valid {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if byte_ranges_overlap(
        out_ready.cast(),
        size_of::<u8>(),
        out_result.cast(),
        size_of::<FrameworkIosFileProviderResultV1>(),
    ) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if operation.is_null() || !pointer_aligned(operation) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    #[cfg(target_os = "ios")]
    {
        if byte_ranges_overlap(
            operation.cast(),
            size_of::<FrameworkIosFileProviderOperation>(),
            out_ready.cast(),
            size_of::<u8>(),
        ) || byte_ranges_overlap(
            operation.cast(),
            size_of::<FrameworkIosFileProviderOperation>(),
            out_result.cast(),
            size_of::<FrameworkIosFileProviderResultV1>(),
        ) {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        catch_unwind_status(AssertUnwindSafe(|| {
            // SAFETY: The caller promises one live unique handle and serialized calls.
            let operation = unsafe { &mut *operation };
            if operation.consumed {
                return FrameworkStatus::NOT_FOUND;
            }
            operation.poll_future();
            if let Some(value) = operation.result.as_ref() {
                operation.signal.close_and_wait();
                let result = result_record(value);
                operation.result.take();
                operation.consumed = true;
                // SAFETY: The caller supplied aligned, writable, non-overlapping outputs.
                unsafe {
                    out_result.write(result);
                    out_ready.write(1);
                }
            }
            FrameworkStatus::OK
        }))
    }
    #[cfg(not(target_os = "ios"))]
    {
        FrameworkStatus::UNSUPPORTED
    }
}

/// Destroys an operation and clears its original handle slot.
///
/// Destroy detaches Rust interest and suppresses future readiness notification; it cannot cancel
/// Apple's native request. It may wait for a readiness callback already in progress to return.
///
/// # Safety
/// `operation` must be null or the original aligned writable slot returned by start. On iOS, the
/// slot must contain null or the live unique handle; do not copy or race the slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_file_provider_operation_destroy(
    operation: *mut *mut FrameworkIosFileProviderOperation,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        if operation.is_null() {
            return FrameworkStatus::OK;
        }
        if !pointer_aligned(operation) {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        // SAFETY: The caller promises a valid original writable pointer slot.
        let value = unsafe { operation.read() };
        if value.is_null() {
            return FrameworkStatus::OK;
        }
        #[cfg(target_os = "ios")]
        {
            if !pointer_aligned(value) {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: Clear the unique original slot before dropping its Box allocation.
            unsafe { operation.write(ptr::null_mut()) };
            // SAFETY: A successful iOS start returned this unique Box allocation.
            unsafe { drop(Box::from_raw(value)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
