use core::ffi::{c_char, c_void};
use core::future::Future;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use framework_connection::{
    ConnectionBackend, ConnectionError, Endpoint, MAX_CHUNK_BYTES, NativeConnectionError,
    ReadChunk, ReadTerminal,
};
use std::ffi::CString;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

#[repr(C)]
struct NativeConnection {
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

unsafe extern "C" {
    fn framework_connection_start(
        context: *mut c_void,
        event_callback: NativeEventCallback,
        hostname: *const c_char,
        port: u16,
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

/// A lazy iOS Network.framework backend for one outbound TLS-over-TCP stream.
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
