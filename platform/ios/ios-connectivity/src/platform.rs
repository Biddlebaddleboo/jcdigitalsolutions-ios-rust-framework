use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use framework_connectivity::{
    NetworkPathBackend, NetworkPathError, NetworkPathMonitorBackend, NetworkPathSnapshot,
    NetworkPathStatus,
};
use framework_core::{Error, ErrorKind};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

#[repr(C)]
struct NativePathMonitor {
    _private: [u8; 0],
}

type PathUpdate = unsafe extern "C" fn(*mut c_void, i32);
type PathCancelled = unsafe extern "C" fn(*mut c_void);

const FRAMEWORK_PATH_STATUS_SATISFIED: i32 = 1;
const FRAMEWORK_PATH_STATUS_UNSATISFIED: i32 = 2;
const FRAMEWORK_PATH_STATUS_SATISFIABLE: i32 = 3;

unsafe extern "C" {
    fn framework_path_monitor_create(
        context: *mut c_void,
        update_callback: PathUpdate,
        cancel_callback: PathCancelled,
    ) -> *mut NativePathMonitor;
    fn framework_path_monitor_create_continuous(
        context: *mut c_void,
        update_callback: PathUpdate,
        cancel_callback: PathCancelled,
    ) -> *mut NativePathMonitor;
    fn framework_path_monitor_cancel(monitor: *mut NativePathMonitor);
    fn framework_path_monitor_release(monitor: *mut NativePathMonitor);
}

struct CompletionCell {
    state: Mutex<State>,
}

struct State {
    completed: bool,
    detached: bool,
    result: Option<Result<NetworkPathSnapshot, NetworkPathError>>,
    waker: Option<Waker>,
}

impl CompletionCell {
    fn new() -> Self {
        Self {
            state: Mutex::new(State {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    fn complete(&self, result: Result<NetworkPathSnapshot, NetworkPathError>) {
        let waker = {
            let mut state = self.lock();
            if state.completed || state.detached {
                return;
            }
            // `completed` claims the event under the same lock as `detach`; a later drop may
            // clear the result but cannot revoke the Waker already claimed by this completion
            state.completed = true;
            state.result = Some(result);
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    fn poll(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Result<NetworkPathSnapshot, NetworkPathError>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
            state.detached = true;
            state.waker = None;
            return Poll::Ready(result);
        }
        if !state.completed
            && !state.detached
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    fn detach(&self) {
        let mut state = self.lock();
        if state.completed {
            state.result = None;
            state.waker = None;
            return;
        }
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

unsafe extern "C" fn path_updated(context: *mut c_void, status: i32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if context.is_null() {
            return;
        }
        let pointer = context.cast::<CompletionCell>();
        // SAFETY: The C shim owns one raw `Arc` reference until its cancel callback. This
        // temporary increment keeps the cell alive for this update callback.
        unsafe { Arc::increment_strong_count(pointer) };
        // SAFETY: The increment above adds the ownership consumed by this temporary `Arc`.
        let cell = unsafe { Arc::from_raw(pointer) };
        cell.complete(Ok(snapshot_from_tag(status)));
    }));
}

fn snapshot_from_tag(status: i32) -> NetworkPathSnapshot {
    let status = match status {
        FRAMEWORK_PATH_STATUS_SATISFIED => NetworkPathStatus::Satisfied,
        FRAMEWORK_PATH_STATUS_UNSATISFIED => NetworkPathStatus::Unsatisfied,
        FRAMEWORK_PATH_STATUS_SATISFIABLE => NetworkPathStatus::Satisfiable,
        _ => NetworkPathStatus::Unknown,
    };
    NetworkPathSnapshot::new(status)
}

struct PathEventCell {
    state: Mutex<PathEventState>,
}

struct PathEventState {
    started: bool,
    cancelled: bool,
    waiter_active: bool,
    latest: Option<NetworkPathSnapshot>,
    waker: Option<Waker>,
}

impl PathEventCell {
    fn new() -> Self {
        Self {
            state: Mutex::new(PathEventState {
                started: false,
                cancelled: false,
                waiter_active: false,
                latest: None,
                waker: None,
            }),
        }
    }

    fn start(&self) -> Result<(), NetworkPathError> {
        let mut state = self.lock();
        if state.cancelled {
            return Err(NetworkPathError::Backend(Error::new(ErrorKind::Cancelled)));
        }
        if state.started {
            return Err(NetworkPathError::Backend(Error::new(
                ErrorKind::AlreadyExists,
            )));
        }
        state.started = true;
        Ok(())
    }

    fn start_failed(&self) {
        self.lock().started = false;
    }

    fn reserve_waiter(&self) -> Result<(), NetworkPathError> {
        let mut state = self.lock();
        if state.cancelled {
            return Err(NetworkPathError::Backend(Error::new(ErrorKind::Cancelled)));
        }
        if !state.started {
            return Err(NetworkPathError::Backend(Error::new(
                ErrorKind::InvalidInput,
            )));
        }
        if state.waiter_active {
            return Err(NetworkPathError::Backend(Error::new(
                ErrorKind::AlreadyExists,
            )));
        }
        state.waiter_active = true;
        Ok(())
    }

    fn publish(&self, snapshot: NetworkPathSnapshot) {
        let waker = {
            let mut state = self.lock();
            if !state.started || state.cancelled {
                return;
            }
            state.latest = Some(snapshot);
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }

    fn cancel(&self) -> Option<Waker> {
        let mut state = self.lock();
        if state.cancelled {
            return None;
        }
        state.cancelled = true;
        state.latest = None;
        state.waker.take()
    }

    fn poll(
        &self,
        context: &mut Context<'_>,
    ) -> Poll<Result<NetworkPathSnapshot, NetworkPathError>> {
        let mut state = self.lock();
        if state.cancelled {
            state.waiter_active = false;
            state.waker = None;
            return Poll::Ready(Err(NetworkPathError::Backend(Error::new(
                ErrorKind::Cancelled,
            ))));
        }
        if let Some(snapshot) = state.latest.take() {
            state.waiter_active = false;
            state.waker = None;
            return Poll::Ready(Ok(snapshot));
        }
        if state
            .waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    fn release_waiter(&self, waker: Option<&Waker>) {
        let mut state = self.lock();
        state.waiter_active = false;
        if waker.is_some_and(|waker| {
            state
                .waker
                .as_ref()
                .is_some_and(|current| current.will_wake(waker))
        }) {
            state.waker = None;
        }
    }

    fn lock(&self) -> MutexGuard<'_, PathEventState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

unsafe extern "C" fn path_update_published(context: *mut c_void, status: i32) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if context.is_null() {
            return;
        }
        let pointer = context.cast::<PathEventCell>();
        // SAFETY: The continuous C monitor owns one raw `Arc` reference until its cancel callback.
        unsafe { Arc::increment_strong_count(pointer) };
        // SAFETY: The increment above adds the ownership consumed by this temporary `Arc`.
        let cell = unsafe { Arc::from_raw(pointer) };
        cell.publish(snapshot_from_tag(status));
    }));
}

unsafe extern "C" fn path_monitor_cancelled(context: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if context.is_null() {
            return;
        }
        // SAFETY: The C shim invokes this once after cancellation completes and no update callback
        // can run again. It releases the raw `Arc` reference transferred at monitor creation.
        drop(unsafe { Arc::from_raw(context.cast::<PathEventCell>()) });
    }));
}

/// A caller-owned iOS backend for coalesced path-change observation.
pub struct IosNetworkPathMonitorBackend {
    cell: Arc<PathEventCell>,
    monitor: Mutex<Option<NonNull<NativePathMonitor>>>,
}

impl IosNetworkPathMonitorBackend {
    /// Creates an inert backend; native observation starts only after `NetworkPathMonitor::start`.
    pub fn new() -> Self {
        Self {
            cell: Arc::new(PathEventCell::new()),
            monitor: Mutex::new(None),
        }
    }

    fn lock_monitor(&self) -> MutexGuard<'_, Option<NonNull<NativePathMonitor>>> {
        self.monitor
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

impl Default for IosNetworkPathMonitorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkPathMonitorBackend for IosNetworkPathMonitorBackend {
    type NextUpdateFuture<'a>
        = IosNextPathUpdateFuture<'a>
    where
        Self: 'a;

    fn start(&self) -> Result<(), NetworkPathError> {
        self.cell.start()?;
        let context = Arc::into_raw(Arc::clone(&self.cell))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: The C shim starts one private serial queue and retains `context` until its
        // once-only cancellation handler. It never calls Rust after that boundary.
        let monitor = unsafe {
            framework_path_monitor_create_continuous(
                context,
                path_update_published,
                path_monitor_cancelled,
            )
        };
        match NonNull::new(monitor) {
            Some(monitor) => {
                *self.lock_monitor() = Some(monitor);
                Ok(())
            }
            None => {
                // SAFETY: A null return means the C shim did not take ownership of `context`.
                drop(unsafe { Arc::from_raw(context.cast::<PathEventCell>()) });
                self.cell.start_failed();
                Err(NetworkPathError::Backend(Error::new(
                    ErrorKind::ResourceExhausted,
                )))
            }
        }
    }

    fn next_update(&self) -> Self::NextUpdateFuture<'_> {
        let (reserved, error) = match self.cell.reserve_waiter() {
            Ok(()) => (true, None),
            Err(error) => (false, Some(error)),
        };
        IosNextPathUpdateFuture {
            cell: &self.cell,
            reserved,
            error,
            last_waker: None,
            result: None,
        }
    }

    fn cancel(&self) {
        let waker = self.cell.cancel();
        if let Some(monitor) = self.lock_monitor().take() {
            // SAFETY: The C shim retains its cancellation-completion reference while this client
            // reference is released after requesting native cancellation.
            unsafe { framework_path_monitor_cancel(monitor.as_ptr()) };
            // SAFETY: This backend owns the one client reference returned by the C shim.
            unsafe { framework_path_monitor_release(monitor.as_ptr()) };
        }
        if let Some(waker) = waker {
            let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
        }
    }
}

impl Drop for IosNetworkPathMonitorBackend {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// A future for one coalesced path update from an iOS monitor.
pub struct IosNextPathUpdateFuture<'a> {
    cell: &'a PathEventCell,
    reserved: bool,
    error: Option<NetworkPathError>,
    last_waker: Option<Waker>,
    result: Option<Result<NetworkPathSnapshot, NetworkPathError>>,
}

impl Future for IosNextPathUpdateFuture<'_> {
    type Output = Result<NetworkPathSnapshot, NetworkPathError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if let Some(result) = this.result {
            return Poll::Ready(result);
        }
        if let Some(error) = this.error.take() {
            let result = Err(error);
            this.result = Some(result);
            return Poll::Ready(result);
        }
        match this.cell.poll(context) {
            Poll::Ready(result) => {
                this.cell.release_waiter(this.last_waker.as_ref());
                this.reserved = false;
                this.result = Some(result);
                Poll::Ready(result)
            }
            Poll::Pending => {
                this.last_waker = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl Drop for IosNextPathUpdateFuture<'_> {
    fn drop(&mut self) {
        if self.reserved {
            self.cell.release_waiter(self.last_waker.as_ref());
        }
    }
}

unsafe extern "C" fn path_cancelled(context: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if context.is_null() {
            return;
        }
        // SAFETY: The C shim invokes this once after cancellation completes and no update
        // callback can run again. It releases the raw `Arc` reference transferred at create.
        drop(unsafe { Arc::from_raw(context.cast::<CompletionCell>()) });
    }));
}

/// A statically selected iOS `NWPathMonitor` backend with no global monitor or registry.
pub struct IosConnectivityBackend;

impl IosConnectivityBackend {
    /// Creates a backend with no native state; each request owns a separate monitor.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for IosConnectivityBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkPathBackend for IosConnectivityBackend {
    type CurrentPathFuture<'a>
        = IosCurrentPathFuture<'a>
    where
        Self: 'a;

    fn current_path<'a>(&'a mut self) -> Self::CurrentPathFuture<'a> {
        IosCurrentPathFuture {
            cell: Arc::new(CompletionCell::new()),
            monitor: None,
            started: false,
            finished: false,
            _backend: self,
        }
    }
}

/// A lazy, one-shot future for the first native path update.
pub struct IosCurrentPathFuture<'a> {
    cell: Arc<CompletionCell>,
    monitor: Option<NonNull<NativePathMonitor>>,
    started: bool,
    finished: bool,
    _backend: &'a mut IosConnectivityBackend,
}

impl IosCurrentPathFuture<'_> {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let context = Arc::into_raw(Arc::clone(&self.cell))
            .cast_mut()
            .cast::<c_void>();
        // SAFETY: The C shim copies its callbacks, starts a private serial dispatch queue, and
        // retains `context` until its once-only cancellation handler. The callbacks use only the
        // Arc-backed completion cell and do not retain this future.
        let monitor =
            unsafe { framework_path_monitor_create(context, path_updated, path_cancelled) };
        match NonNull::new(monitor) {
            Some(monitor) => self.monitor = Some(monitor),
            None => {
                // SAFETY: A null return means the C shim did not take ownership of `context`.
                drop(unsafe { Arc::from_raw(context.cast::<CompletionCell>()) });
                self.cell.complete(Err(NetworkPathError::Backend(Error::new(
                    ErrorKind::ResourceExhausted,
                ))));
            }
        }
    }
}

impl Future for IosCurrentPathFuture<'_> {
    type Output = Result<NetworkPathSnapshot, NetworkPathError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        this.start();
        match this.cell.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                if let Some(monitor) = this.monitor.take() {
                    // SAFETY: This future owns the one client reference returned by the C shim.
                    unsafe { framework_path_monitor_release(monitor.as_ptr()) };
                }
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosCurrentPathFuture<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.cell.detach();
        if let Some(monitor) = self.monitor.take() {
            // SAFETY: The C shim has a separate cancellation-completion reference; it keeps the
            // monitor and callback context alive after this future releases its client reference.
            unsafe { framework_path_monitor_cancel(monitor.as_ptr()) };
            // SAFETY: The future releases its client reference once after requesting cancellation.
            unsafe { framework_path_monitor_release(monitor.as_ptr()) };
        }
    }
}
