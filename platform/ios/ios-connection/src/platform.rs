use crate::listener_queue::{PendingAcceptQueue, QueuePushError};
use core::ffi::{c_char, c_void};
use core::future::Future;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use framework_connection::{
    AcceptedConnectionBackend, ConnectionBackend, ConnectionError, Endpoint, MAX_CHUNK_BYTES,
    MAX_QUEUED_ACCEPTS, NativeConnectionError, ReadChunk, ReadTerminal, TcpListenerBackend,
};
use std::ffi::CString;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

#[repr(C)]
struct NativeConnection {
    _private: [u8; 0],
}

#[repr(C)]
struct NativeListener {
    _private: [u8; 0],
}

#[derive(Clone, Copy)]
struct NativeEvent {
    kind: i32,
    operation_id: u64,
    state: i32,
    bytes: *const u8,
    bytes_length: usize,
    is_complete: bool,
    is_final_context: bool,
    has_error: bool,
    resource_exhausted: bool,
    error_domain: i32,
    error_code: i32,
}

type NativeEventCallback = unsafe extern "C" fn(
    *mut c_void,
    i32,
    u64,
    i32,
    *const u8,
    usize,
    bool,
    bool,
    bool,
    bool,
    i32,
    i32,
);

type NativeListenerEventCallback = unsafe extern "C" fn(*mut c_void, i32, u16, bool, i32, i32);
type NativeListenerAcceptCallback = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32;

const EVENT_STATE: i32 = 1;
const EVENT_SEND: i32 = 2;
const EVENT_RECEIVE: i32 = 3;
const EVENT_CLOSED: i32 = 4;

const STATE_INVALID: i32 = 0;
const STATE_WAITING: i32 = 1;
const STATE_PREPARING: i32 = 2;
const STATE_READY: i32 = 3;
const STATE_FAILED: i32 = 4;

const STATUS_OK: i32 = 0;
const STATUS_INVALID: i32 = 1;
const STATUS_RESOURCE_EXHAUSTED: i32 = 2;
const LISTENER_EVENT_READY: i32 = 1;
const LISTENER_EVENT_FAILED: i32 = 2;
const LISTENER_EVENT_CLOSED: i32 = 3;

unsafe extern "C" {
    fn framework_connection_start(
        context: *mut c_void,
        event_callback: NativeEventCallback,
        hostname: *const c_char,
        port: u16,
        out_connection: *mut *mut NativeConnection,
    ) -> i32;
    fn framework_connection_adopt(
        context: *mut c_void,
        event_callback: NativeEventCallback,
        native_connection: *mut c_void,
        out_connection: *mut *mut NativeConnection,
    ) -> i32;
    fn framework_connection_send(
        connection: *mut NativeConnection,
        operation_id: u64,
        bytes: *const u8,
        bytes_length: usize,
    ) -> i32;
    fn framework_connection_receive(
        connection: *mut NativeConnection,
        operation_id: u64,
        maximum_length: u32,
    ) -> i32;
    fn framework_connection_cancel(connection: *mut NativeConnection);
    fn framework_connection_release(connection: *mut NativeConnection);
    fn framework_listener_start(
        context: *mut c_void,
        event_callback: NativeListenerEventCallback,
        accept_callback: NativeListenerAcceptCallback,
        port: u16,
        out_listener: *mut *mut NativeListener,
    ) -> i32;
    fn framework_listener_cancel(listener: *mut NativeListener);
    fn framework_listener_release(listener: *mut NativeListener);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Idle,
    Connecting,
    Ready,
    Failed(ConnectionError),
    Cancelling,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OperationKind {
    Send,
    Receive,
}

enum OperationResult {
    Send(Result<(), ConnectionError>),
    Receive(Result<ReadChunk, ConnectionError>),
}

struct ActiveOperation {
    id: u64,
    kind: OperationKind,
    maximum_length: Option<usize>,
    result: Option<OperationResult>,
    waker: Option<Waker>,
}

struct SharedState {
    phase: Phase,
    connect_waker: Option<Waker>,
    close_result: Option<Result<(), ConnectionError>>,
    close_waker: Option<Waker>,
    active_operation: Option<ActiveOperation>,
}

struct ConnectionShared {
    state: Mutex<SharedState>,
}

impl ConnectionShared {
    fn new() -> Self {
        Self {
            state: Mutex::new(SharedState {
                phase: Phase::Idle,
                connect_waker: None,
                close_result: None,
                close_waker: None,
                active_operation: None,
            }),
        }
    }

    fn ready() -> Self {
        let shared = Self::new();
        shared.lock().phase = Phase::Ready;
        shared
    }

    fn lock(&self) -> MutexGuard<'_, SharedState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn poll_connect(&self, context: &mut Context<'_>) -> Poll<Result<(), ConnectionError>> {
        let mut state = self.lock();
        match state.phase {
            Phase::Ready => Poll::Ready(Ok(())),
            Phase::Failed(error) => Poll::Ready(Err(error)),
            Phase::Closed | Phase::Cancelling => Poll::Ready(Err(ConnectionError::Cancelled)),
            Phase::Idle | Phase::Connecting => {
                store_waker(&mut state.connect_waker, context.waker());
                Poll::Pending
            }
        }
    }

    fn poll_close(&self, context: &mut Context<'_>) -> Poll<Result<(), ConnectionError>> {
        let mut state = self.lock();
        if let Some(result) = state.close_result {
            return Poll::Ready(result);
        }
        store_waker(&mut state.close_waker, context.waker());
        Poll::Pending
    }

    fn set_close_waker(&self, waker: &Waker) {
        let mut state = self.lock();
        if state.close_result.is_none() {
            store_waker(&mut state.close_waker, waker);
        }
    }

    fn fail_start(&self, error: ConnectionError) {
        let waker = {
            let mut state = self.lock();
            if matches!(state.phase, Phase::Connecting | Phase::Idle) {
                state.phase = Phase::Failed(error);
                state.connect_waker.take()
            } else {
                None
            }
        };
        wake(waker);
    }

    fn receive_limit(&self, id: u64) -> Option<usize> {
        self.lock().active_operation.as_ref().and_then(|operation| {
            (operation.id == id && operation.kind == OperationKind::Receive)
                .then_some(operation.maximum_length)
                .flatten()
        })
    }

    fn complete_operation(&self, id: u64, result: OperationResult) {
        let waker = {
            let mut state = self.lock();
            let Some(operation) = state
                .active_operation
                .as_mut()
                .filter(|operation| operation.id == id)
            else {
                return;
            };
            if operation.result.is_some() {
                return;
            }
            operation.result = Some(result);
            operation.waker.take()
        };
        wake(waker);
    }

    fn poll_operation(
        &self,
        id: u64,
        kind: OperationKind,
        context: &mut Context<'_>,
    ) -> Poll<Result<OperationResult, ConnectionError>> {
        let mut state = self.lock();
        let Some(operation) = state
            .active_operation
            .as_mut()
            .filter(|operation| operation.id == id && operation.kind == kind)
        else {
            return Poll::Ready(Err(phase_error(state.phase)));
        };
        if let Some(result) = operation.result.take() {
            state.active_operation = None;
            Poll::Ready(Ok(result))
        } else {
            store_waker(&mut operation.waker, context.waker());
            Poll::Pending
        }
    }

    fn detach_connect(&self) {
        self.lock().connect_waker = None;
    }

    fn detach_close(&self) {
        self.lock().close_waker = None;
    }

    fn detach_operation(&self, id: u64) {
        let mut state = self.lock();
        if state
            .active_operation
            .as_ref()
            .is_some_and(|operation| operation.id == id)
        {
            state.active_operation = None;
        }
    }

    fn start_operation(
        &self,
        id: u64,
        kind: OperationKind,
        maximum_length: Option<usize>,
        waker: &Waker,
    ) -> Result<(), ConnectionError> {
        let mut state = self.lock();
        match state.phase {
            Phase::Ready => {}
            phase => return Err(phase_error(phase)),
        }
        if state.active_operation.is_some() {
            return Err(ConnectionError::InvalidState);
        }
        state.active_operation = Some(ActiveOperation {
            id,
            kind,
            maximum_length,
            result: None,
            waker: Some(waker.clone()),
        });
        Ok(())
    }

    fn native_event(&self, event: NativeEvent) {
        match event.kind {
            EVENT_STATE => self.state_event(
                event.state,
                event.has_error,
                event.error_domain,
                event.error_code,
            ),
            EVENT_SEND => self.complete_operation(
                event.operation_id,
                OperationResult::Send(native_result(
                    event.has_error,
                    event.error_domain,
                    event.error_code,
                )),
            ),
            EVENT_RECEIVE => self.receive_event(event),
            EVENT_CLOSED => {
                self.closed_event(event.has_error, event.error_domain, event.error_code)
            }
            _ => {}
        }
    }

    fn state_event(&self, native_state: i32, has_error: bool, error_domain: i32, error_code: i32) {
        let waker = {
            let mut state = self.lock();
            if matches!(state.phase, Phase::Cancelling | Phase::Closed) {
                return;
            }
            match native_state {
                STATE_WAITING | STATE_PREPARING => return,
                STATE_READY => {
                    state.phase = Phase::Ready;
                    state.connect_waker.take()
                }
                STATE_FAILED => {
                    state.phase = Phase::Failed(native_error(has_error, error_domain, error_code));
                    state.connect_waker.take()
                }
                STATE_INVALID => {
                    state.phase = Phase::Failed(ConnectionError::BackendFailure);
                    state.connect_waker.take()
                }
                _ => {
                    state.phase = Phase::Failed(ConnectionError::BackendFailure);
                    state.connect_waker.take()
                }
            }
        };
        wake(waker);
    }

    fn receive_event(&self, event: NativeEvent) {
        let Some(maximum_length) = self.receive_limit(event.operation_id) else {
            return;
        };
        if event.resource_exhausted {
            self.complete_operation(
                event.operation_id,
                OperationResult::Receive(Err(ConnectionError::ReceiveDataUnavailable(
                    event
                        .has_error
                        .then(|| NativeConnectionError::new(event.error_domain, event.error_code)),
                ))),
            );
            return;
        }
        if event.bytes_length > maximum_length || (event.bytes_length > 0 && event.bytes.is_null())
        {
            self.complete_operation(
                event.operation_id,
                OperationResult::Receive(Err(ConnectionError::BackendFailure)),
            );
            return;
        }

        let mut owned = Vec::new();
        if owned.try_reserve_exact(event.bytes_length).is_err() {
            self.complete_operation(
                event.operation_id,
                OperationResult::Receive(Err(ConnectionError::ReceiveDataUnavailable(
                    event
                        .has_error
                        .then(|| NativeConnectionError::new(event.error_domain, event.error_code)),
                ))),
            );
            return;
        }
        if event.bytes_length > 0 {
            // SAFETY: The C shim keeps mapped dispatch data alive for the callback duration and
            // reports this exact length; the null check above rejects an invalid nonempty span.
            owned.extend_from_slice(unsafe {
                core::slice::from_raw_parts(event.bytes, event.bytes_length)
            });
        }
        let terminal = if event.has_error {
            ReadTerminal::Error(native_error(true, event.error_domain, event.error_code))
        } else if event.is_complete && event.is_final_context {
            ReadTerminal::EndOfStream
        } else if event.bytes_length > 0 {
            ReadTerminal::More
        } else {
            ReadTerminal::Error(ConnectionError::BackendFailure)
        };
        self.complete_operation(
            event.operation_id,
            OperationResult::Receive(ReadChunk::new(owned, terminal)),
        );
    }

    fn closed_event(&self, has_error: bool, error_domain: i32, error_code: i32) {
        let (connect_waker, close_waker, operation_waker) = {
            let mut state = self.lock();
            state.phase = Phase::Closed;
            let close_result = if has_error {
                Err(native_error(true, error_domain, error_code))
            } else {
                Ok(())
            };
            state.close_result = Some(close_result);
            let operation_waker = if let Some(operation) = state.active_operation.as_mut() {
                if operation.result.is_none() {
                    let error = if has_error {
                        native_error(true, error_domain, error_code)
                    } else {
                        ConnectionError::Cancelled
                    };
                    operation.result = Some(match operation.kind {
                        OperationKind::Send => OperationResult::Send(Err(error)),
                        OperationKind::Receive => OperationResult::Receive(Err(error)),
                    });
                }
                operation.waker.take()
            } else {
                None
            };
            (
                state.connect_waker.take(),
                state.close_waker.take(),
                operation_waker,
            )
        };
        wake(connect_waker);
        wake(close_waker);
        wake(operation_waker);
    }
}

fn store_waker(slot: &mut Option<Waker>, waker: &Waker) {
    if slot.as_ref().is_none_or(|stored| !stored.will_wake(waker)) {
        *slot = Some(waker.clone());
    }
}

fn wake(waker: Option<Waker>) {
    if let Some(waker) = waker {
        let mut waker = ManuallyDrop::new(waker);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: The ManuallyDrop remains live and is not moved while wake_by_ref runs.
            waker.wake_by_ref();
        }));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: This is the single explicit drop for the ManuallyDrop-wrapped Waker.
            unsafe { ManuallyDrop::drop(&mut waker) };
        }));
    }
}

fn native_error(has_error: bool, domain: i32, code: i32) -> ConnectionError {
    if has_error {
        ConnectionError::Native(NativeConnectionError::new(domain, code))
    } else {
        ConnectionError::BackendFailure
    }
}

fn native_result(has_error: bool, domain: i32, code: i32) -> Result<(), ConnectionError> {
    if has_error {
        Err(native_error(true, domain, code))
    } else {
        Ok(())
    }
}

fn phase_error(phase: Phase) -> ConnectionError {
    match phase {
        Phase::Failed(error) => error,
        Phase::Idle | Phase::Connecting | Phase::Ready => ConnectionError::InvalidState,
        Phase::Cancelling | Phase::Closed => ConnectionError::Cancelled,
    }
}

fn status_error(status: i32) -> ConnectionError {
    match status {
        STATUS_RESOURCE_EXHAUSTED => ConnectionError::ResourceExhausted,
        STATUS_INVALID => ConnectionError::InvalidState,
        _ => ConnectionError::BackendFailure,
    }
}

unsafe extern "C" fn native_event(
    context: *mut c_void,
    event_kind: i32,
    operation_id: u64,
    connection_state: i32,
    bytes: *const u8,
    bytes_length: usize,
    is_complete: bool,
    is_final_context: bool,
    has_error: bool,
    resource_exhausted: bool,
    error_domain: i32,
    error_code: i32,
) {
    if context.is_null() {
        return;
    }
    let pointer = context.cast::<ConnectionShared>();
    // SAFETY: The C shim owns one raw `Arc` reference until the final cancelled callback.
    unsafe { Arc::increment_strong_count(pointer) };
    // SAFETY: The increment above adds exactly the reference consumed by this callback.
    let shared = unsafe { Arc::from_raw(pointer) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        shared.native_event(NativeEvent {
            kind: event_kind,
            operation_id,
            state: connection_state,
            bytes,
            bytes_length,
            is_complete,
            is_final_context,
            has_error,
            resource_exhausted,
            error_domain,
            error_code,
        });
    }));
    let temporary_context = Arc::into_raw(shared);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `temporary_context` is the temporary Arc ownership created above.
        drop(unsafe { Arc::from_raw(temporary_context) });
    }));
    if event_kind == EVENT_CLOSED {
        // SAFETY: C sends one final closed event and releases its persistent callback context
        // exactly once after native cancellation completes.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: This is the persistent raw Arc ownership transferred to C at start.
            drop(unsafe { Arc::from_raw(pointer) });
        }));
    }
}

/// A lazy iOS Network.framework backend for one TCP stream.
///
/// Outbound connections use TLS over TCP. Streams adopted from [`IosTcpListenerBackend`] use
/// plaintext TCP; this backend does not add TLS to accepted streams.
pub struct IosConnectionBackend {
    shared: Arc<ConnectionShared>,
    native: Option<NonNull<NativeConnection>>,
    next_operation_id: Option<u64>,
}

impl IosConnectionBackend {
    /// Creates an idle backend without allocating native state or starting a connection.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(ConnectionShared::new()),
            native: None,
            next_operation_id: Some(1),
        }
    }

    fn from_accepted(native_connection: *mut c_void) -> Result<Self, ConnectionError> {
        if native_connection.is_null() {
            return Err(ConnectionError::BackendFailure);
        }
        let shared = Arc::new(ConnectionShared::ready());
        let context = Arc::into_raw(Arc::clone(&shared))
            .cast_mut()
            .cast::<c_void>();
        let mut native = core::ptr::null_mut();
        // SAFETY: The listener callback lends an unstarted Network.framework connection for this
        // call. The C shim retains it, installs this backend's callbacks, and takes the context
        // reference only when it returns success.
        let status = unsafe {
            framework_connection_adopt(context, native_event, native_connection, &mut native)
        };
        if status != STATUS_OK {
            // SAFETY: A non-success status means the C shim did not retain the Rust callback
            // context.
            drop(unsafe { Arc::from_raw(context.cast::<ConnectionShared>()) });
            return Err(status_error(status));
        }
        let native = NonNull::new(native)
            .expect("successful accepted connection adoption returns a native handle");
        Ok(Self {
            shared,
            native: Some(native),
            next_operation_id: Some(1),
        })
    }

    fn begin_connect(&mut self, endpoint: Endpoint) -> Result<(), ConnectionError> {
        let can_start = {
            let mut state = self.shared.lock();
            if state.phase == Phase::Idle {
                state.phase = Phase::Connecting;
                true
            } else {
                false
            }
        };
        if !can_start {
            return Err(phase_error(self.shared.lock().phase));
        }

        let hostname = match CString::new(endpoint.host()) {
            Ok(hostname) => hostname,
            Err(_) => {
                self.shared.fail_start(ConnectionError::InvalidEndpoint);
                return Err(ConnectionError::InvalidEndpoint);
            }
        };
        let context = Arc::into_raw(Arc::clone(&self.shared))
            .cast_mut()
            .cast::<c_void>();
        let mut native = core::ptr::null_mut();
        // SAFETY: The C shim copies its state block, starts one private serial queue, and assumes
        // ownership of this Arc reference only when it returns a non-null native handle.
        let status = unsafe {
            framework_connection_start(
                context,
                native_event,
                hostname.as_ptr(),
                endpoint.port().get(),
                &mut native,
            )
        };
        if status != STATUS_OK {
            // SAFETY: A non-success C status means it did not transfer the callback context.
            drop(unsafe { Arc::from_raw(context.cast::<ConnectionShared>()) });
            self.shared.fail_start(status_error(status));
            return Err(status_error(status));
        }
        self.native = NonNull::new(native);
        if self.native.is_none() {
            // A successful shim start must return its client handle; callbacks retain the context
            // until cancellation even if this defensive contract check fails.
            self.shared.fail_start(ConnectionError::BackendFailure);
            self.request_cancel();
            return Err(ConnectionError::BackendFailure);
        }
        Ok(())
    }

    fn allocate_operation_id(&mut self) -> Result<u64, ConnectionError> {
        let id = self
            .next_operation_id
            .take()
            .ok_or(ConnectionError::ResourceExhausted)?;
        self.next_operation_id = id.checked_add(1);
        Ok(id)
    }

    fn begin_send(&mut self, bytes: &[u8], waker: &Waker) -> Result<u64, ConnectionError> {
        if bytes.len() > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        let native = self
            .native
            .ok_or_else(|| phase_error(self.shared.lock().phase))?;
        let id = self.allocate_operation_id()?;
        self.shared
            .start_operation(id, OperationKind::Send, None, waker)?;
        // SAFETY: The C shim copies this bounded slice before returning and retains its send block
        // until Network.framework invokes the one-shot completion callback.
        let status =
            unsafe { framework_connection_send(native.as_ptr(), id, bytes.as_ptr(), bytes.len()) };
        if status != STATUS_OK {
            self.shared
                .complete_operation(id, OperationResult::Send(Err(status_error(status))));
        }
        Ok(id)
    }

    fn begin_receive(
        &mut self,
        maximum_length: u32,
        waker: &Waker,
    ) -> Result<u64, ConnectionError> {
        if maximum_length as usize > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        let native = self
            .native
            .ok_or_else(|| phase_error(self.shared.lock().phase))?;
        let id = self.allocate_operation_id()?;
        self.shared.start_operation(
            id,
            OperationKind::Receive,
            Some(maximum_length as usize),
            waker,
        )?;
        // SAFETY: The C shim asks Network.framework for at most this nonzero, capped length and
        // copies mapped dispatch data only during its one-shot receive callback.
        let status = unsafe { framework_connection_receive(native.as_ptr(), id, maximum_length) };
        if status != STATUS_OK {
            self.shared
                .complete_operation(id, OperationResult::Receive(Err(status_error(status))));
        }
        Ok(id)
    }

    fn request_cancel(&mut self) {
        let should_cancel = {
            let mut state = self.shared.lock();
            match state.phase {
                Phase::Closed | Phase::Cancelling => false,
                Phase::Idle => {
                    state.phase = Phase::Closed;
                    state.close_result = Some(Ok(()));
                    let connect_waker = state.connect_waker.take();
                    let close_waker = state.close_waker.take();
                    drop(state);
                    wake(connect_waker);
                    wake(close_waker);
                    false
                }
                _ if self.native.is_none() => {
                    state.phase = Phase::Closed;
                    state.close_result = Some(Ok(()));
                    let connect_waker = state.connect_waker.take();
                    let close_waker = state.close_waker.take();
                    drop(state);
                    wake(connect_waker);
                    wake(close_waker);
                    false
                }
                _ => {
                    state.phase = Phase::Cancelling;
                    state.connect_waker = None;
                    true
                }
            }
        };
        if should_cancel && let Some(native) = self.native {
            // SAFETY: The C shim makes cancellation idempotent and retains its connection state
            // until the asynchronous final cancelled callback releases native resources.
            unsafe { framework_connection_cancel(native.as_ptr()) };
        }
    }

    fn close_without_native(&mut self) {
        let (connect_waker, close_waker) = {
            let mut state = self.shared.lock();
            state.phase = Phase::Closed;
            state.close_result = Some(Ok(()));
            (state.connect_waker.take(), state.close_waker.take())
        };
        wake(connect_waker);
        wake(close_waker);
    }
}

impl Default for IosConnectionBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AcceptedConnectionBackend for IosConnectionBackend {}

impl ConnectionBackend for IosConnectionBackend {
    type ConnectFuture<'a> = IosConnectFuture<'a>;
    type SendFuture<'a> = IosSendFuture<'a>;
    type ReceiveFuture<'a> = IosReceiveFuture<'a>;
    type CloseFuture<'a> = IosCloseFuture<'a>;

    fn connect<'a>(&'a mut self, endpoint: Endpoint) -> Self::ConnectFuture<'a> {
        IosConnectFuture {
            backend: self,
            endpoint: Some(endpoint),
            started: false,
            finished: false,
        }
    }

    fn send<'a>(&'a mut self, bytes: &'a [u8]) -> Self::SendFuture<'a> {
        IosSendFuture {
            backend: self,
            bytes,
            operation_id: None,
            started: false,
            finished: false,
        }
    }

    fn receive<'a>(&'a mut self, max_bytes: core::num::NonZeroU32) -> Self::ReceiveFuture<'a> {
        IosReceiveFuture {
            backend: self,
            maximum_length: max_bytes.get(),
            operation_id: None,
            started: false,
            finished: false,
        }
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        IosCloseFuture {
            backend: self,
            started: false,
            finished: false,
        }
    }

    fn cancel(&mut self) {
        self.request_cancel();
    }
}

impl Drop for IosConnectionBackend {
    fn drop(&mut self) {
        self.request_cancel();
        if let Some(native) = self.native.take() {
            // SAFETY: The backend owns exactly the client reference returned by the C shim; the
            // shim keeps its separate callback reference alive until cancellation completes.
            unsafe { framework_connection_release(native.as_ptr()) };
        }
    }
}

/// A lazy future that resolves when Network.framework reports a ready stream or failure.
pub struct IosConnectFuture<'a> {
    backend: &'a mut IosConnectionBackend,
    endpoint: Option<Endpoint>,
    started: bool,
    finished: bool,
}

impl Future for IosConnectFuture<'_> {
    type Output = Result<(), ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            let Some(endpoint) = this.endpoint.take() else {
                this.finished = true;
                return Poll::Ready(Err(ConnectionError::InvalidState));
            };
            if let Err(error) = this.backend.begin_connect(endpoint) {
                this.finished = true;
                return Poll::Ready(Err(error));
            }
        }
        match this.backend.shared.poll_connect(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosConnectFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_connect();
            self.backend.request_cancel();
        }
    }
}

/// A lazy future for one copied send chunk and its native send completion.
pub struct IosSendFuture<'a> {
    backend: &'a mut IosConnectionBackend,
    bytes: &'a [u8],
    operation_id: Option<u64>,
    started: bool,
    finished: bool,
}

impl Future for IosSendFuture<'_> {
    type Output = Result<(), ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            if this.bytes.len() > MAX_CHUNK_BYTES {
                this.finished = true;
                return Poll::Ready(Err(ConnectionError::ChunkTooLarge));
            }
            if this.bytes.is_empty() {
                this.finished = true;
                return Poll::Ready(Ok(()));
            }
            match this.backend.begin_send(this.bytes, context.waker()) {
                Ok(id) => this.operation_id = Some(id),
                Err(error) => {
                    this.finished = true;
                    return Poll::Ready(Err(error));
                }
            }
        }
        let id = this.operation_id.expect("started send has an operation id");
        match this
            .backend
            .shared
            .poll_operation(id, OperationKind::Send, context)
        {
            Poll::Ready(Ok(OperationResult::Send(result))) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Ready(Ok(OperationResult::Receive(_))) => {
                this.finished = true;
                Poll::Ready(Err(ConnectionError::BackendFailure))
            }
            Poll::Ready(Err(error)) => {
                this.finished = true;
                Poll::Ready(Err(error))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosSendFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            if let Some(id) = self.operation_id {
                self.backend.shared.detach_operation(id);
            }
            self.backend.request_cancel();
        }
    }
}

/// A lazy future for one owned receive result of at most the requested byte limit.
pub struct IosReceiveFuture<'a> {
    backend: &'a mut IosConnectionBackend,
    maximum_length: u32,
    operation_id: Option<u64>,
    started: bool,
    finished: bool,
}

impl Future for IosReceiveFuture<'_> {
    type Output = Result<ReadChunk, ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            if this.maximum_length as usize > MAX_CHUNK_BYTES {
                this.finished = true;
                return Poll::Ready(Err(ConnectionError::ChunkTooLarge));
            }
            match this
                .backend
                .begin_receive(this.maximum_length, context.waker())
            {
                Ok(id) => this.operation_id = Some(id),
                Err(error) => {
                    this.finished = true;
                    return Poll::Ready(Err(error));
                }
            }
        }
        let id = this
            .operation_id
            .expect("started receive has an operation id");
        match this
            .backend
            .shared
            .poll_operation(id, OperationKind::Receive, context)
        {
            Poll::Ready(Ok(OperationResult::Receive(result))) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Ready(Ok(OperationResult::Send(_))) => {
                this.finished = true;
                Poll::Ready(Err(ConnectionError::BackendFailure))
            }
            Poll::Ready(Err(error)) => {
                this.finished = true;
                Poll::Ready(Err(error))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosReceiveFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            if let Some(id) = self.operation_id {
                self.backend.shared.detach_operation(id);
            }
            self.backend.request_cancel();
        }
    }
}

/// A future that waits for asynchronous Network.framework connection cancellation.
pub struct IosCloseFuture<'a> {
    backend: &'a mut IosConnectionBackend,
    started: bool,
    finished: bool,
}

impl Future for IosCloseFuture<'_> {
    type Output = Result<(), ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            if this.backend.native.is_none() {
                this.backend.close_without_native();
            } else {
                this.backend.shared.set_close_waker(context.waker());
                this.backend.request_cancel();
            }
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

impl Drop for IosCloseFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_close();
            self.backend.request_cancel();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListenerPhase {
    Idle,
    Binding,
    Ready(core::num::NonZeroU16),
    Failed(ConnectionError),
    Cancelling,
    Closed,
}

struct PendingAccepted(IosConnectionBackend);

// SAFETY: Network.framework connections are thread-safe native objects. The C connection shim
// serializes cancellation/resource teardown with a pthread mutex, and all Rust callback state is
// behind `ConnectionShared`'s mutex. This wrapper only permits the listener callback queue to move
// an owned accepted backend to the caller's accept future; it does not permit concurrent Rust
// access to that backend.
unsafe impl Send for PendingAccepted {}

impl PendingAccepted {
    fn into_backend(self) -> IosConnectionBackend {
        self.0
    }
}

struct ListenerState {
    phase: ListenerPhase,
    cancel_requested: bool,
    bind_waker: Option<Waker>,
    accept_waker: Option<Waker>,
    close_result: Option<Result<(), ConnectionError>>,
    close_waker: Option<Waker>,
    accepted: PendingAcceptQueue<PendingAccepted>,
}

struct ListenerShared {
    state: Mutex<ListenerState>,
}

impl ListenerShared {
    fn new() -> Self {
        Self {
            state: Mutex::new(ListenerState {
                phase: ListenerPhase::Idle,
                cancel_requested: false,
                bind_waker: None,
                accept_waker: None,
                close_result: None,
                close_waker: None,
                accepted: PendingAcceptQueue::new(),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, ListenerState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn begin_bind(&self) -> Result<(), ConnectionError> {
        let mut state = self.lock();
        if state.phase == ListenerPhase::Idle {
            state.phase = ListenerPhase::Binding;
            Ok(())
        } else {
            Err(listener_phase_error(state.phase))
        }
    }

    fn fail_start(&self, error: ConnectionError) {
        let waker = {
            let mut state = self.lock();
            if matches!(state.phase, ListenerPhase::Binding | ListenerPhase::Idle) {
                state.phase = ListenerPhase::Failed(error);
                state.bind_waker.take()
            } else {
                None
            }
        };
        wake(waker);
    }

    fn poll_bind(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Result<core::num::NonZeroU16, ConnectionError>> {
        let mut state = self.lock();
        match state.phase {
            ListenerPhase::Ready(port) => Poll::Ready(Ok(port)),
            ListenerPhase::Failed(error) => Poll::Ready(Err(error)),
            ListenerPhase::Cancelling | ListenerPhase::Closed => {
                Poll::Ready(Err(ConnectionError::Cancelled))
            }
            ListenerPhase::Idle | ListenerPhase::Binding => {
                store_waker(&mut state.bind_waker, context.waker());
                Poll::Pending
            }
        }
    }

    fn poll_accept(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Result<IosConnectionBackend, ConnectionError>> {
        let mut state = self.lock();
        if let Some(accepted) = state.accepted.pop() {
            return Poll::Ready(Ok(accepted.into_backend()));
        }
        match state.phase {
            ListenerPhase::Ready(_) => {
                store_waker(&mut state.accept_waker, context.waker());
                Poll::Pending
            }
            ListenerPhase::Failed(error) => Poll::Ready(Err(error)),
            ListenerPhase::Idle | ListenerPhase::Binding => {
                Poll::Ready(Err(ConnectionError::InvalidState))
            }
            ListenerPhase::Cancelling | ListenerPhase::Closed => {
                Poll::Ready(Err(ConnectionError::Cancelled))
            }
        }
    }

    fn poll_close(&self, context: &mut Context<'_>) -> Poll<Result<(), ConnectionError>> {
        let mut state = self.lock();
        if let Some(result) = state.close_result {
            return Poll::Ready(result);
        }
        store_waker(&mut state.close_waker, context.waker());
        Poll::Pending
    }

    fn set_close_waker(&self, waker: &Waker) {
        let mut state = self.lock();
        if state.close_result.is_none() {
            store_waker(&mut state.close_waker, waker);
        }
    }

    fn detach_bind(&self) {
        self.lock().bind_waker = None;
    }

    fn detach_accept(&self) {
        self.lock().accept_waker = None;
    }

    fn detach_close(&self) {
        self.lock().close_waker = None;
    }

    fn has_accept_capacity(&self) -> bool {
        let state = self.lock();
        matches!(state.phase, ListenerPhase::Ready(_)) && state.accepted.len() < MAX_QUEUED_ACCEPTS
    }

    fn enqueue_accepted(
        &self,
        backend: IosConnectionBackend,
    ) -> Result<Option<Waker>, (i32, PendingAccepted)> {
        let mut state = self.lock();
        let accepted = PendingAccepted(backend);
        if !matches!(state.phase, ListenerPhase::Ready(_)) {
            return Err((STATUS_INVALID, accepted));
        }
        match state.accepted.push(accepted) {
            Ok(()) => Ok(state.accept_waker.take()),
            Err(QueuePushError::Full(accepted)) => Err((STATUS_INVALID, accepted)),
            Err(QueuePushError::ResourceExhausted(accepted)) => {
                Err((STATUS_RESOURCE_EXHAUSTED, accepted))
            }
        }
    }

    fn accept_native(&self, native_connection: *mut c_void) -> i32 {
        if !self.has_accept_capacity() {
            return STATUS_INVALID;
        }
        let backend = match IosConnectionBackend::from_accepted(native_connection) {
            Ok(backend) => backend,
            Err(ConnectionError::ResourceExhausted) => return STATUS_RESOURCE_EXHAUSTED,
            Err(_) => return STATUS_INVALID,
        };
        match self.enqueue_accepted(backend) {
            Ok(waker) => {
                wake(waker);
                STATUS_OK
            }
            Err((status, accepted)) => {
                drop(accepted);
                status
            }
        }
    }

    fn native_event(
        &self,
        event_kind: i32,
        port: u16,
        has_error: bool,
        error_domain: i32,
        error_code: i32,
    ) {
        let (bind_waker, accept_waker, close_waker, abandoned) = {
            let mut state = self.lock();
            match event_kind {
                LISTENER_EVENT_READY if state.phase == ListenerPhase::Binding => {
                    match core::num::NonZeroU16::new(port) {
                        Some(port) => state.phase = ListenerPhase::Ready(port),
                        None => {
                            state.phase = ListenerPhase::Failed(ConnectionError::BackendFailure)
                        }
                    }
                    (state.bind_waker.take(), None, None, None)
                }
                LISTENER_EVENT_FAILED => {
                    state.phase =
                        ListenerPhase::Failed(native_error(has_error, error_domain, error_code));
                    (
                        state.bind_waker.take(),
                        state.accept_waker.take(),
                        None,
                        None,
                    )
                }
                LISTENER_EVENT_CLOSED => {
                    state.phase = ListenerPhase::Closed;
                    state.close_result = Some(if has_error {
                        Err(native_error(true, error_domain, error_code))
                    } else {
                        Ok(())
                    });
                    (
                        state.bind_waker.take(),
                        state.accept_waker.take(),
                        state.close_waker.take(),
                        Some(state.accepted.take_all()),
                    )
                }
                _ => (None, None, None, None),
            }
        };
        drop(abandoned);
        wake(bind_waker);
        wake(accept_waker);
        wake(close_waker);
    }

    fn close_without_native(&self) {
        let (bind_waker, accept_waker, close_waker, abandoned) = {
            let mut state = self.lock();
            state.phase = ListenerPhase::Closed;
            state.close_result = Some(Ok(()));
            (
                state.bind_waker.take(),
                state.accept_waker.take(),
                state.close_waker.take(),
                state.accepted.take_all(),
            )
        };
        drop(abandoned);
        wake(bind_waker);
        wake(accept_waker);
        wake(close_waker);
    }
}

fn listener_phase_error(phase: ListenerPhase) -> ConnectionError {
    match phase {
        ListenerPhase::Failed(error) => error,
        ListenerPhase::Idle | ListenerPhase::Binding | ListenerPhase::Ready(_) => {
            ConnectionError::InvalidState
        }
        ListenerPhase::Cancelling | ListenerPhase::Closed => ConnectionError::Cancelled,
    }
}

unsafe extern "C" fn native_listener_event(
    context: *mut c_void,
    event_kind: i32,
    port: u16,
    has_error: bool,
    error_domain: i32,
    error_code: i32,
) {
    if context.is_null() {
        return;
    }
    let pointer = context.cast::<ListenerShared>();
    // SAFETY: The C shim owns one raw `Arc` reference until its final cancelled callback.
    unsafe { Arc::increment_strong_count(pointer) };
    // SAFETY: The increment above adds exactly the reference consumed by this callback.
    let shared = unsafe { Arc::from_raw(pointer) };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        shared.native_event(event_kind, port, has_error, error_domain, error_code);
    }));
    let temporary_context = Arc::into_raw(shared);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: This is the temporary Arc ownership created above.
        drop(unsafe { Arc::from_raw(temporary_context) });
    }));
    if event_kind == LISTENER_EVENT_CLOSED {
        // SAFETY: The C shim sends one final close event and releases this persistent context.
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: This is the persistent raw Arc ownership transferred to the C shim.
            drop(unsafe { Arc::from_raw(pointer) });
        }));
    }
}

unsafe extern "C" fn native_listener_accept(
    context: *mut c_void,
    native_connection: *mut c_void,
) -> i32 {
    if context.is_null() || native_connection.is_null() {
        return STATUS_INVALID;
    }
    let pointer = context.cast::<ListenerShared>();
    // SAFETY: The C shim owns one raw `Arc` reference until its final cancelled callback.
    unsafe { Arc::increment_strong_count(pointer) };
    // SAFETY: The increment above adds exactly the reference consumed by this callback.
    let shared = unsafe { Arc::from_raw(pointer) };
    let status = catch_unwind(AssertUnwindSafe(|| shared.accept_native(native_connection)))
        .unwrap_or(STATUS_RESOURCE_EXHAUSTED);
    let temporary_context = Arc::into_raw(shared);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: This is the temporary Arc ownership created above.
        drop(unsafe { Arc::from_raw(temporary_context) });
    }));
    status
}

/// An iOS TCP listener backed by public Network.framework APIs.
///
/// Accepted streams use plaintext TCP with no TLS configuration.
pub struct IosTcpListenerBackend {
    shared: Arc<ListenerShared>,
    native: Option<NonNull<NativeListener>>,
}

impl IosTcpListenerBackend {
    /// Creates an idle listener backend without binding or allocating native state.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(ListenerShared::new()),
            native: None,
        }
    }

    fn begin_bind(&mut self, port: u16) -> Result<(), ConnectionError> {
        self.shared.begin_bind()?;
        let context = Arc::into_raw(Arc::clone(&self.shared))
            .cast_mut()
            .cast::<c_void>();
        let mut native = core::ptr::null_mut();
        // SAFETY: The C shim starts one public NWListener and takes ownership of this callback
        // context only when it returns success. Port zero requests a system-selected port.
        let status = unsafe {
            framework_listener_start(
                context,
                native_listener_event,
                native_listener_accept,
                port,
                &mut native,
            )
        };
        if status != STATUS_OK {
            // SAFETY: A non-success status means the C shim did not transfer this context.
            drop(unsafe { Arc::from_raw(context.cast::<ListenerShared>()) });
            self.shared.fail_start(status_error(status));
            return Err(status_error(status));
        }
        self.native = NonNull::new(native);
        if self.native.is_none() {
            self.shared.fail_start(ConnectionError::BackendFailure);
            return Err(ConnectionError::BackendFailure);
        }
        Ok(())
    }

    fn request_cancel(&mut self) {
        let native = self.native;
        let (should_cancel, bind_waker, accept_waker, abandoned) = {
            let mut state = self.shared.lock();
            match state.phase {
                ListenerPhase::Closed => (false, None, None, None),
                ListenerPhase::Idle => {
                    state.phase = ListenerPhase::Closed;
                    state.close_result = Some(Ok(()));
                    (
                        false,
                        state.bind_waker.take(),
                        state.accept_waker.take(),
                        Some(state.accepted.take_all()),
                    )
                }
                _ if state.cancel_requested => (false, None, None, None),
                phase => {
                    state.cancel_requested = true;
                    if !matches!(phase, ListenerPhase::Failed(_)) {
                        state.phase = ListenerPhase::Cancelling;
                    }
                    (
                        native.is_some(),
                        state.bind_waker.take(),
                        state.accept_waker.take(),
                        Some(state.accepted.take_all()),
                    )
                }
            }
        };
        drop(abandoned);
        wake(bind_waker);
        wake(accept_waker);
        if should_cancel {
            if let Some(native) = native {
                // SAFETY: The C shim serializes cancel requests and retains its native listener
                // and callback context until the asynchronous cancelled event.
                unsafe { framework_listener_cancel(native.as_ptr()) };
            }
        } else if native.is_none() {
            self.shared.close_without_native();
        }
    }
}

impl Default for IosTcpListenerBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpListenerBackend for IosTcpListenerBackend {
    type AcceptedBackend = IosConnectionBackend;
    type BindFuture<'a> = IosTcpListenerBindFuture<'a>;
    type AcceptFuture<'a> = IosTcpListenerAcceptFuture<'a>;
    type CloseFuture<'a> = IosTcpListenerCloseFuture<'a>;

    fn bind<'a>(&'a mut self, port: u16) -> Self::BindFuture<'a> {
        IosTcpListenerBindFuture {
            backend: self,
            port,
            started: false,
            finished: false,
        }
    }

    fn accept<'a>(&'a mut self) -> Self::AcceptFuture<'a> {
        IosTcpListenerAcceptFuture {
            backend: self,
            started: false,
            finished: false,
        }
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        IosTcpListenerCloseFuture {
            backend: self,
            started: false,
            finished: false,
        }
    }

    fn cancel(&mut self) {
        self.request_cancel();
    }
}

impl Drop for IosTcpListenerBackend {
    fn drop(&mut self) {
        self.request_cancel();
        if let Some(native) = self.native.take() {
            // SAFETY: This backend owns the client reference returned by the C shim; the shim
            // retains a separate callback reference through asynchronous listener cancellation.
            unsafe { framework_listener_release(native.as_ptr()) };
        }
    }
}

/// A lazy future that resolves when the iOS TCP listener is ready and its port is known.
pub struct IosTcpListenerBindFuture<'a> {
    backend: &'a mut IosTcpListenerBackend,
    port: u16,
    started: bool,
    finished: bool,
}

impl Future for IosTcpListenerBindFuture<'_> {
    type Output = Result<core::num::NonZeroU16, ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            if let Err(error) = this.backend.begin_bind(this.port) {
                this.finished = true;
                return Poll::Ready(Err(error));
            }
        }
        match this.backend.shared.poll_bind(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosTcpListenerBindFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_bind();
            self.backend.request_cancel();
        }
    }
}

/// A future that yields the next ready iOS TCP stream or a listener error.
pub struct IosTcpListenerAcceptFuture<'a> {
    backend: &'a mut IosTcpListenerBackend,
    started: bool,
    finished: bool,
}

impl Future for IosTcpListenerAcceptFuture<'_> {
    type Output = Result<IosConnectionBackend, ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        this.started = true;
        match this.backend.shared.poll_accept(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosTcpListenerAcceptFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_accept();
        }
    }
}

/// A future that waits for asynchronous iOS TCP listener cancellation.
pub struct IosTcpListenerCloseFuture<'a> {
    backend: &'a mut IosTcpListenerBackend,
    started: bool,
    finished: bool,
}

impl Future for IosTcpListenerCloseFuture<'_> {
    type Output = Result<(), ConnectionError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.started = true;
            this.backend.shared.set_close_waker(context.waker());
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

impl Drop for IosTcpListenerCloseFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.finished {
            self.backend.shared.detach_close();
            self.backend.request_cancel();
        }
    }
}
