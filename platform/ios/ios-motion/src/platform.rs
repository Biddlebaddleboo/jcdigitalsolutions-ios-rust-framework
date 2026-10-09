use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use framework_motion::{
    Acceleration, AccelerationSample, MotionBackend, MotionError, MotionTimestamp,
};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_core_motion::{CMAccelerometerData, CMMotionManager};
use objc2_foundation::{NSError, NSOperationQueue};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};

type Completion = Arc<CompletionCell>;

/// Caller-owned, main-thread-bound one-shot Core Motion backend.
///
/// Construct this around the app's single `CMMotionManager`; the backend retains that same
/// manager and does not create a global instance.
pub struct IosMotionBackend {
    manager: Retained<CMMotionManager>,
    queue: Retained<NSOperationQueue>,
    marker: MainThreadMarker,
}

impl IosMotionBackend {
    /// Creates a backend around the app-owned manager on its main-thread context.
    ///
    /// The app must coordinate all other users of this manager's accelerometer update channel.
    pub fn new(manager: Retained<CMMotionManager>, marker: MainThreadMarker) -> Self {
        let queue = NSOperationQueue::new();
        queue.setMaxConcurrentOperationCount(1);
        Self {
            manager,
            queue,
            marker,
        }
    }

    /// Borrows the retained app-owned Core Motion manager.
    ///
    /// Do not start another accelerometer update service while a backend request is active.
    pub fn native_manager(&self) -> &CMMotionManager {
        &self.manager
    }
}

impl MotionBackend for IosMotionBackend {
    fn availability(&self) -> Availability {
        // SAFETY: `marker` binds this backend to the main-thread context, and the retained
        // manager remains valid for the duration of the property read.
        if unsafe { self.manager.isAccelerometerAvailable() } {
            Availability::Available
        } else {
            Availability::TemporarilyUnavailable
        }
    }

    type CurrentAccelerationFuture<'a>
        = IosCurrentAccelerationFuture<'a>
    where
        Self: 'a;

    fn current_acceleration<'a>(&'a mut self) -> Self::CurrentAccelerationFuture<'a> {
        let marker = self.marker;
        IosCurrentAccelerationFuture {
            backend: self,
            completion: Arc::new(CompletionCell::new()),
            started: false,
            updates_started: false,
            finished: false,
            _marker: marker,
        }
    }
}

/// A lazy iOS request for one raw accelerometer sample.
///
/// Poll and drop this future on the main thread. Its callback runs on a dedicated serial operation
/// queue and accesses only synchronized callback state. It starts only on first poll and stops the
/// manager's accelerometer updates when the future next polls ready or when a pending future drops.
pub struct IosCurrentAccelerationFuture<'a> {
    backend: &'a mut IosMotionBackend,
    completion: Completion,
    started: bool,
    updates_started: bool,
    finished: bool,
    _marker: MainThreadMarker,
}

impl IosCurrentAccelerationFuture<'_> {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        // SAFETY: The future's marker keeps access on the main thread and the manager is retained.
        if !unsafe { self.backend.manager.isAccelerometerAvailable() } {
            self.completion
                .complete(Err(backend_error(ErrorKind::Unavailable, None)));
            return;
        }

        // SAFETY: The manager is retained and accessed on its main-thread context. A pre-existing
        // update service belongs to another app consumer and must not be replaced or stopped.
        if unsafe { self.backend.manager.isAccelerometerActive() } {
            self.completion
                .complete(Err(backend_error(ErrorKind::AlreadyExists, None)));
            return;
        }

        let completion = Arc::clone(&self.completion);
        let handler =
            block2::RcBlock::new(move |data: *mut CMAccelerometerData, error: *mut NSError| {
                if !completion.is_interested() {
                    return;
                }
                let result =
                    catch_unwind(AssertUnwindSafe(|| unsafe { callback_result(data, error) }))
                        .unwrap_or_else(|_| Err(backend_error(ErrorKind::Internal, None)));
                completion.complete(result);
            });
        self.updates_started = true;
        // SAFETY: `queue` is a dedicated serial NSOperationQueue. The callback captures only a
        // stable, synchronized Arc cell and cannot access the future or manager. Core Motion's
        // escaping-handler contract owns a copy for callback delivery; its exact release instant
        // after stop is unspecified.
        unsafe {
            self.backend
                .manager
                .startAccelerometerUpdatesToQueue_withHandler(
                    &self.backend.queue,
                    block2::RcBlock::as_ptr(&handler),
                );
        }
    }
}

impl Future for IosCurrentAccelerationFuture<'_> {
    type Output = Result<AccelerationSample, MotionError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        if !this.started {
            this.start();
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                if this.updates_started {
                    // SAFETY: Future polling is main-thread-bound by `_marker`; the backend owns
                    // the manager, and the callback never accesses it from the operation queue.
                    unsafe { this.backend.manager.stopAccelerometerUpdates() };
                }
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosCurrentAccelerationFuture<'_> {
    fn drop(&mut self) {
        self.completion.detach();
        if self.updates_started && !self.finished {
            // SAFETY: Drop is main-thread-bound by `marker`; this future started the service and
            // its manager remains retained by the backend.
            unsafe { self.backend.manager.stopAccelerometerUpdates() };
        }
    }
}

struct CompletionCell {
    state: Mutex<CompletionState>,
}

struct CompletionState {
    completed: bool,
    detached: bool,
    result: Option<Result<AccelerationSample, MotionError>>,
    waker: Option<Waker>,
}

impl CompletionCell {
    const fn new() -> Self {
        Self {
            state: Mutex::new(CompletionState {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    fn complete(&self, result: Result<AccelerationSample, MotionError>) -> bool {
        let waker = {
            let mut state = self.lock();
            if state.completed || state.detached {
                return false;
            }
            state.completed = true;
            state.result = Some(result);
            state.waker.take()
        };
        if let Some(waker) = waker {
            let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
        }
        true
    }

    fn is_interested(&self) -> bool {
        let state = self.lock();
        !state.completed && !state.detached
    }

    fn poll(&self, context: &mut Context<'_>) -> Poll<Result<AccelerationSample, MotionError>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
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
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, CompletionState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

unsafe fn callback_result(
    data: *mut CMAccelerometerData,
    error: *mut NSError,
) -> Result<AccelerationSample, MotionError> {
    if !error.is_null() {
        // SAFETY: Core Motion provides a valid NSError for the duration of this callback.
        return Err(native_error(unsafe { (&*error).code() }));
    }
    if data.is_null() {
        return Err(backend_error(ErrorKind::Unavailable, None));
    }
    // SAFETY: Core Motion provides a valid sample object for the duration of this callback.
    let data = unsafe { &*data };
    // SAFETY: `data` is the live callback object supplied by Core Motion.
    let native = unsafe { data.acceleration() };
    // SAFETY: `data` is the live callback object supplied by Core Motion.
    let timestamp_seconds = unsafe { data.timestamp() };
    let acceleration =
        Acceleration::new(native.x * 9.80665, native.y * 9.80665, native.z * 9.80665)?;
    let timestamp_nanos = timestamp_seconds * 1_000_000_000.0;
    if !timestamp_nanos.is_finite() || timestamp_nanos < 0.0 || timestamp_nanos >= u64::MAX as f64 {
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    let rounded_nanos = timestamp_nanos.round();
    if rounded_nanos >= u64::MAX as f64 {
        return Err(backend_error(ErrorKind::Unsupported, None));
    }
    Ok(AccelerationSample::new(
        acceleration,
        MotionTimestamp::from_monotonic_nanos(rounded_nanos as u64),
    ))
}

fn native_error(code: isize) -> MotionError {
    let native_code = i32::try_from(code).ok().and_then(PlatformErrorCode::new);
    backend_error(ErrorKind::Platform, native_code)
}

fn backend_error(kind: ErrorKind, code: Option<PlatformErrorCode>) -> MotionError {
    let mut error = Error::new(kind);
    if let Some(code) = code {
        error = error.with_platform_code(code);
    }
    MotionError::Backend(error)
}
