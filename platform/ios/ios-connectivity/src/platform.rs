use core::ffi::c_void;
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use framework_connectivity::{
    NetworkPathBackend, NetworkPathError, NetworkPathSnapshot, NetworkPathStatus,
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
        let status = match status {
            FRAMEWORK_PATH_STATUS_SATISFIED => NetworkPathStatus::Satisfied,
            FRAMEWORK_PATH_STATUS_UNSATISFIED => NetworkPathStatus::Unsatisfied,
            FRAMEWORK_PATH_STATUS_SATISFIABLE => NetworkPathStatus::Satisfiable,
            _ => NetworkPathStatus::Unknown,
        };
        cell.complete(Ok(NetworkPathSnapshot::new(status)));
    }));
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
