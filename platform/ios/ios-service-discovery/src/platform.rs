use core::ffi::{c_char, c_void};
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use framework_service_discovery::{
    BonjourBrowseError, BonjourBrowserBackend, BonjourBrowserEvent, BonjourBrowserState,
    BonjourQuery, BonjourService, MAX_BONJOUR_COMPONENT_BYTES, NativeBonjourError,
};

use crate::state::{QueueServiceError, Shared};

const STATUS_OK: i32 = 0;
const STATUS_INVALID: i32 = 1;
const STATUS_RESOURCE_EXHAUSTED: i32 = 2;
const CALLBACK_OK: i32 = 0;
const CALLBACK_STOP: i32 = 1;
const EVENT_STATE: i32 = 1;
const EVENT_SERVICE_ADDED: i32 = 2;
const EVENT_SERVICE_REMOVED: i32 = 3;
const EVENT_INVALID_RESULT: i32 = 4;
const STATE_READY: i32 = 1;
const STATE_FAILED: i32 = 2;
const STATE_CLOSED: i32 = 3;
const STATE_WAITING: i32 = 4;

type NativeBrowser = c_void;
type NativeBonjourCallback = unsafe extern "C" fn(
    context: *mut c_void,
    event_kind: i32,
    browser_state: i32,
    service_name: *const u8,
    service_name_length: usize,
    service_type: *const u8,
    service_type_length: usize,
    service_domain: *const u8,
    service_domain_length: usize,
    has_error: bool,
    error_domain: i32,
    error_code: i32,
) -> i32;

unsafe extern "C" {
    fn framework_bonjour_browser_start(
        context: *mut c_void,
        event_callback: NativeBonjourCallback,
        service_type: *const c_char,
        service_domain: *const c_char,
        browser: *mut *mut NativeBrowser,
    ) -> i32;
    fn framework_bonjour_browser_cancel(browser: *mut NativeBrowser);
    fn framework_bonjour_browser_release(browser: *mut NativeBrowser);
}

/// An idle, caller-owned iOS Bonjour browser backend.
///
/// The backend uses the public Network.framework `NWBrowser` C API and starts only when
/// [`BonjourBrowserBackend::start`] is first polled. The host application must own the required
/// Local Network usage description and Bonjour service-type declarations.
pub struct IosBonjourBrowserBackend {
    shared: Arc<Shared>,
    native: Option<NonNull<NativeBrowser>>,
}

impl IosBonjourBrowserBackend {
    /// Creates an idle backend without allocating native browser state.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared::new()),
            native: None,
        }
    }

    fn begin_start(&mut self, query: BonjourQuery) -> Result<(), BonjourBrowseError> {
        self.shared.begin_start()?;
        let context = Arc::into_raw(Arc::clone(&self.shared))
            .cast_mut()
            .cast::<c_void>();
        let mut service_type = [0; MAX_BONJOUR_COMPONENT_BYTES + 1];
        let service_type_value = query.service_type().as_bytes();
        service_type[..service_type_value.len()].copy_from_slice(service_type_value);
        let mut service_domain = [0; MAX_BONJOUR_COMPONENT_BYTES + 1];
        let domain = query.domain().map_or(core::ptr::null(), |value| {
            let bytes = value.as_bytes();
            service_domain[..bytes.len()].copy_from_slice(bytes);
            service_domain.as_ptr().cast::<c_char>()
        });
        let mut native = core::ptr::null_mut();
        // SAFETY: The query owns validated NUL-free UTF-8 bytes for the full synchronous call.
        // The C shim creates its Network.framework descriptor before returning and retains the
        // transferred Arc callback context until its final cancelled-state callback.
        let status = unsafe {
            framework_bonjour_browser_start(
                context,
                native_bonjour_callback,
                service_type.as_ptr().cast::<c_char>(),
                domain,
                &mut native,
            )
        };
        if status != STATUS_OK {
            // SAFETY: A failed C start does not retain the callback context.
            drop(unsafe { Arc::from_raw(context.cast::<Shared>()) });
            let error = status_error(status);
            self.shared.fail_start(error);
            return Err(error);
        }
        self.native = NonNull::new(native);
        if self.native.is_none() {
            // The C shim's success contract always returns its owned browser handle.
            self.shared.fail_start(BonjourBrowseError::BackendFailure);
            return Err(BonjourBrowseError::BackendFailure);
        }
        Ok(())
    }

    fn request_cancel(&mut self) {
        if self.shared.request_cancel() {
            if let Some(native) = self.native {
                // SAFETY: The C shim serializes cancellation with the final callback and retains
                // its callback context and native browser through asynchronous cleanup.
                unsafe { framework_bonjour_browser_cancel(native.as_ptr()) };
            }
        }
    }
}

impl Default for IosBonjourBrowserBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl BonjourBrowserBackend for IosBonjourBrowserBackend {
    type StartFuture<'a> = IosBonjourStartFuture<'a>;
    type NextEventFuture<'a> = IosBonjourNextEventFuture<'a>;
    type CloseFuture<'a> = IosBonjourCloseFuture<'a>;

    fn start<'a>(&'a mut self, query: BonjourQuery) -> Self::StartFuture<'a> {
        IosBonjourStartFuture {
            backend: self,
            query: Some(query),
            finished: false,
        }
    }

    fn next_event<'a>(&'a mut self) -> Self::NextEventFuture<'a> {
        IosBonjourNextEventFuture {
            backend: self,
            finished: false,
        }
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        IosBonjourCloseFuture {
            backend: self,
            started: false,
            finished: false,
        }
    }

    fn cancel(&mut self) {
        self.request_cancel();
    }
}

impl Drop for IosBonjourBrowserBackend {
    fn drop(&mut self) {
        self.request_cancel();
        if let Some(native) = self.native.take() {
            // SAFETY: This is the client reference returned by the C shim; callback ownership
            // remains until the final NWBrowser cancelled-state event.
            unsafe { framework_bonjour_browser_release(native.as_ptr()) };
        }
    }
}

/// A future that lazily creates and starts an iOS Bonjour browser.
pub struct IosBonjourStartFuture<'a> {
    backend: &'a mut IosBonjourBrowserBackend,
    query: Option<BonjourQuery>,
    finished: bool,
}

impl Future for IosBonjourStartFuture<'_> {
    type Output = Result<(), BonjourBrowseError>;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Ready(Err(BonjourBrowseError::InvalidState));
        }
        this.finished = true;
        let query = this
            .query
            .take()
            .expect("unpolled start future owns its query");
        Poll::Ready(this.backend.begin_start(query))
    }
}

/// A future that yields one iOS Bonjour service or browser-state event.
pub struct IosBonjourNextEventFuture<'a> {
    backend: &'a mut IosBonjourBrowserBackend,
    finished: bool,
}

impl Future for IosBonjourNextEventFuture<'_> {
    type Output = Result<BonjourBrowserEvent, BonjourBrowseError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Ready(Err(BonjourBrowseError::InvalidState));
        }
        match this.backend.shared.poll_next_event(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosBonjourNextEventFuture<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.backend.shared.detach_next();
        }
    }
}

/// A future that resolves after the iOS browser's final cancellation callback.
pub struct IosBonjourCloseFuture<'a> {
    backend: &'a mut IosBonjourBrowserBackend,
    started: bool,
    finished: bool,
}

impl Future for IosBonjourCloseFuture<'_> {
    type Output = Result<(), BonjourBrowseError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Ready(Err(BonjourBrowseError::InvalidState));
        }
        if !this.started {
            this.started = true;
            this.backend.request_cancel();
        }
        match this.backend.shared.poll_close(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosBonjourCloseFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_close();
        }
    }
}

unsafe extern "C" fn native_bonjour_callback(
    context: *mut c_void,
    event_kind: i32,
    browser_state: i32,
    service_name: *const u8,
    service_name_length: usize,
    service_type: *const u8,
    service_type_length: usize,
    service_domain: *const u8,
    service_domain_length: usize,
    has_error: bool,
    error_domain: i32,
    error_code: i32,
) -> i32 {
    if context.is_null() {
        return CALLBACK_STOP;
    }
    let shared = context.cast::<Shared>();
    // SAFETY: The C shim owns one raw Arc strong reference until the final cancelled callback.
    let shared_ref = unsafe { &*shared };
    let final_callback = event_kind == EVENT_STATE && browser_state == STATE_CLOSED;
    let result = catch_unwind(AssertUnwindSafe(|| {
        process_native_event(
            shared_ref,
            event_kind,
            browser_state,
            service_name,
            service_name_length,
            service_type,
            service_type_length,
            service_domain,
            service_domain_length,
            has_error,
            error_domain,
            error_code,
        )
    }));
    let callback_status = match result {
        Ok(status) => status,
        Err(_) => {
            let _ = catch_unwind(AssertUnwindSafe(|| shared_ref.callback_panic()));
            CALLBACK_STOP
        }
    };
    if final_callback {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: The final native callback consumes the single raw Arc owner transferred at
            // start. Apple documents the cancelled state as the browser's final callback.
            drop(unsafe { Arc::from_raw(shared) });
        }));
    }
    callback_status
}

#[allow(clippy::too_many_arguments)]
fn process_native_event(
    shared: &Shared,
    event_kind: i32,
    browser_state: i32,
    service_name: *const u8,
    service_name_length: usize,
    service_type: *const u8,
    service_type_length: usize,
    service_domain: *const u8,
    service_domain_length: usize,
    has_error: bool,
    error_domain: i32,
    error_code: i32,
) -> i32 {
    let native_error = has_error.then_some(NativeBonjourError::new(error_domain, error_code));
    if event_kind == EVENT_STATE {
        let (state, terminal) = match browser_state {
            STATE_READY => (BonjourBrowserState::Ready, None),
            STATE_WAITING => (BonjourBrowserState::Waiting(native_error), None),
            STATE_FAILED => (
                BonjourBrowserState::Failed(native_error),
                Some(native_error.map_or(
                    BonjourBrowseError::BackendFailure,
                    BonjourBrowseError::Native,
                )),
            ),
            STATE_CLOSED => (BonjourBrowserState::Closed, None),
            _ => (
                BonjourBrowserState::Failed(native_error),
                Some(native_error.map_or(
                    BonjourBrowseError::BackendFailure,
                    BonjourBrowseError::Native,
                )),
            ),
        };
        shared.set_native_state(state, terminal);
        return if browser_state == STATE_FAILED || browser_state == 0 {
            CALLBACK_STOP
        } else {
            CALLBACK_OK
        };
    }
    if event_kind == EVENT_INVALID_RESULT {
        shared.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::InvalidNativeService),
        );
        return CALLBACK_STOP;
    }
    if event_kind != EVENT_SERVICE_ADDED && event_kind != EVENT_SERVICE_REMOVED {
        shared.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::BackendFailure),
        );
        return CALLBACK_STOP;
    }
    // SAFETY: This callback receives these slices only from the C shim, which passes bounded
    // endpoint strings retained for the duration of the synchronous callback.
    let Some(name) = (unsafe { native_bytes(service_name, service_name_length) }) else {
        shared.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::InvalidNativeService),
        );
        return CALLBACK_STOP;
    };
    // SAFETY: As above, the C shim supplies a bounded endpoint string for this callback.
    let Some(service_type) = (unsafe { native_bytes(service_type, service_type_length) }) else {
        shared.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::InvalidNativeService),
        );
        return CALLBACK_STOP;
    };
    // SAFETY: As above, the C shim supplies a bounded endpoint string for this callback.
    let Some(domain) = (unsafe { native_bytes(service_domain, service_domain_length) }) else {
        shared.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::InvalidNativeService),
        );
        return CALLBACK_STOP;
    };
    let service = match BonjourService::from_utf8_components(name, service_type, domain) {
        Ok(service) => service,
        Err(error) => {
            shared.set_native_state(BonjourBrowserState::Failed(None), Some(error));
            return CALLBACK_STOP;
        }
    };
    match shared.push_service(event_kind == EVENT_SERVICE_ADDED, service) {
        Ok(()) => CALLBACK_OK,
        Err(QueueServiceError::Full | QueueServiceError::Allocation) => CALLBACK_STOP,
    }
}

unsafe fn native_bytes<'a>(value: *const u8, length: usize) -> Option<&'a [u8]> {
    if value.is_null() || length == 0 || length > MAX_BONJOUR_COMPONENT_BYTES {
        return None;
    }
    // SAFETY: The C shim validates each native C string's bounded byte length and keeps its
    // endpoint retained until this synchronous callback returns.
    let bytes = unsafe { core::slice::from_raw_parts(value, length) };
    Some(bytes)
}

fn status_error(status: i32) -> BonjourBrowseError {
    match status {
        STATUS_INVALID => BonjourBrowseError::InvalidState,
        STATUS_RESOURCE_EXHAUSTED => BonjourBrowseError::ResourceExhausted,
        _ => BonjourBrowseError::BackendFailure,
    }
}
