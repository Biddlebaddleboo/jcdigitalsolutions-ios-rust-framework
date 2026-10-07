use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use core::task::{Context, Poll, Waker};
use framework_core::Cancellation;

use crate::waker_slot::WakerSlot;

const CREATED: u8 = 0;
const RUNNING: u8 = 1;
const CANCELLATION_REQUESTED: u8 = 2;
const COMPLETING: u8 = 3;
const COMPLETED: u8 = 4;
const CANCELLED: u8 = 5;

/// The observable lifecycle phase of a one-shot operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum OperationPhase {
    /// The operation has not started.
    Created,
    /// The backend may perform work or complete the operation.
    Running,
    /// Cancellation won the race and its reason is being published.
    CancellationRequested,
    /// Completion won the race and its result is being published.
    Completing,
    /// A result is ready for its single future consumer.
    Completed,
    /// Cancellation is terminal and its reason is ready.
    Cancelled,
}

/// The single terminal outcome of an operation.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Completion<T, E> {
    /// The operation completed successfully.
    Success(T),
    /// The backend completed with its error value.
    Failure(E),
    /// Cancellation won before backend completion.
    Cancelled(Cancellation),
}

/// A zero-sized indication that an operation has an active or already-used future consumer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FutureAlreadyTaken;

/// One-shot operation storage shared by a backend completion path and one Rust future.
///
/// The state itself performs no allocation and starts no task. The owner must keep it alive
/// until all backend callbacks stop; dropping a future only unregisters its waker and releases
/// the single-consumer claim.
pub struct OperationState<T, E> {
    phase: AtomicU8,
    cancellation: AtomicU8,
    future_taken: AtomicBool,
    terminal_taken: AtomicBool,
    result: UnsafeCell<Option<Completion<T, E>>>,
    waker: WakerSlot,
}

// `result` is written by the unique completion winner and read by the unique future consumer
// after an Acquire load of `phase`; sharing across threads therefore requires Send payloads.
unsafe impl<T: Send, E: Send> Sync for OperationState<T, E> {}

impl<T, E> OperationState<T, E> {
    /// Creates an idle operation cell without allocation or runtime initialization.
    pub const fn new() -> Self {
        Self {
            phase: AtomicU8::new(CREATED),
            cancellation: AtomicU8::new(0),
            future_taken: AtomicBool::new(false),
            terminal_taken: AtomicBool::new(false),
            result: UnsafeCell::new(None),
            waker: WakerSlot::new(),
        }
    }

    /// Returns the observable phase.
    pub fn phase(&self) -> OperationPhase {
        match self.phase.load(Ordering::Acquire) {
            CREATED => OperationPhase::Created,
            RUNNING => OperationPhase::Running,
            CANCELLATION_REQUESTED => OperationPhase::CancellationRequested,
            COMPLETING => OperationPhase::Completing,
            COMPLETED => OperationPhase::Completed,
            CANCELLED => OperationPhase::Cancelled,
            _ => unreachable!("operation phase is private and bounded"),
        }
    }

    /// Transitions a newly created operation to running; later calls return `false`.
    pub fn start(&self) -> bool {
        self.phase
            .compare_exchange(CREATED, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Publishes a successful or failed backend result exactly once.
    ///
    /// The operation must be running. If cancellation or another completion already won, this
    /// returns `false` and drops the supplied result.
    pub fn complete(&self, result: Result<T, E>) -> bool {
        if self
            .phase
            .compare_exchange(RUNNING, COMPLETING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        let completion = match result {
            Ok(value) => Completion::Success(value),
            Err(error) => Completion::Failure(error),
        };
        // SAFETY: only the thread that claimed RUNNING as COMPLETING writes this cell; the
        // release store below publishes the initialized value to the unique future consumer.
        unsafe {
            *self.result.get() = Some(completion);
        }
        self.phase.store(COMPLETED, Ordering::Release);
        self.waker.wake();
        true
    }

    /// Returns a borrowed cancellation source for this operation.
    pub fn cancellation_source(&self) -> CancellationSource<'_> {
        CancellationSource {
            phase: &self.phase,
            reason: &self.cancellation,
            waker: &self.waker,
        }
    }

    /// Creates the unique future consumer, or reports that a consumer is active or already finished.
    pub fn future(&self) -> Result<OperationFuture<'_, T, E>, FutureAlreadyTaken> {
        if self.terminal_taken.load(Ordering::Acquire) {
            return Err(FutureAlreadyTaken);
        }
        self.future_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| FutureAlreadyTaken)?;
        if self.terminal_taken.load(Ordering::Acquire) {
            self.future_taken.store(false, Ordering::Release);
            return Err(FutureAlreadyTaken);
        }
        Ok(OperationFuture {
            state: self,
            registration: None,
            finished: false,
        })
    }

    fn take_completion(&self) -> Option<Completion<T, E>> {
        let phase = self.phase.load(Ordering::Acquire);
        if phase != COMPLETED && phase != CANCELLED {
            return None;
        }
        self.terminal_taken
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        match phase {
            COMPLETED => {
                // SAFETY: `future_taken` grants one consumer, and COMPLETED is published only
                // after the unique completion winner initializes the cell.
                unsafe { (*self.result.get()).take() }
            }
            CANCELLED => {
                let reason = cancellation_from_code(self.cancellation.load(Ordering::Acquire));
                reason.map(Completion::Cancelled)
            }
            _ => None,
        }
    }
}

impl<T, E> Default for OperationState<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// A read-only, borrow-scoped view of an operation's cancellation state.
#[derive(Clone, Copy)]
pub struct CancellationToken<'a> {
    phase: &'a AtomicU8,
    reason: &'a AtomicU8,
}

impl CancellationToken<'_> {
    /// Returns whether cancellation reached its terminal state.
    pub fn is_cancelled(self) -> bool {
        self.phase.load(Ordering::Acquire) == CANCELLED
    }

    /// Returns the reason after cancellation reaches its terminal state.
    pub fn reason(self) -> Option<Cancellation> {
        if !self.is_cancelled() {
            return None;
        }
        cancellation_from_code(self.reason.load(Ordering::Acquire))
    }
}

/// The borrow-scoped owner-side capability that may request operation cancellation.
#[derive(Clone, Copy)]
pub struct CancellationSource<'a> {
    phase: &'a AtomicU8,
    reason: &'a AtomicU8,
    waker: &'a WakerSlot,
}

impl<'a> CancellationSource<'a> {
    /// Returns a read-only token for polling cancellation state.
    pub const fn token(self) -> CancellationToken<'a> {
        CancellationToken {
            phase: self.phase,
            reason: self.reason,
        }
    }

    /// Requests cancellation; returns `true` only when this request wins the terminal race.
    pub fn cancel(self, reason: Cancellation) -> bool {
        loop {
            let current = self.phase.load(Ordering::Acquire);
            if current != CREATED && current != RUNNING {
                return false;
            }
            if self
                .phase
                .compare_exchange(
                    current,
                    CANCELLATION_REQUESTED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                self.reason.store(reason as u8, Ordering::Relaxed);
                self.phase.store(CANCELLED, Ordering::Release);
                self.waker.wake();
                return true;
            }
        }
    }
}

/// A guard that unregisters the future's task waker when dropped.
pub struct CancellationRegistration<'a> {
    pub(crate) waker: &'a WakerSlot,
    pub(crate) id: u32,
}

impl Drop for CancellationRegistration<'_> {
    fn drop(&mut self) {
        self.waker.unregister(self.id);
    }
}

/// A future that borrows an operation cell and consumes its terminal outcome once.
pub struct OperationFuture<'a, T, E> {
    state: &'a OperationState<T, E>,
    registration: Option<CancellationRegistration<'a>>,
    finished: bool,
}

impl<T, E> Future for OperationFuture<'_, T, E> {
    type Output = Completion<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This type has no self-references and may be safely projected without pin machinery.
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        if let Some(completion) = this.state.take_completion() {
            this.finished = true;
            return Poll::Ready(completion);
        }

        this.registration.take();
        let registration = this.state.waker.register(cx.waker());
        if let Some(completion) = this.state.take_completion() {
            drop(registration);
            this.finished = true;
            return Poll::Ready(completion);
        }
        this.registration = Some(registration);
        Poll::Pending
    }
}

impl<T, E> Drop for OperationFuture<'_, T, E> {
    fn drop(&mut self) {
        self.registration.take();
        if !self.finished {
            self.state.future_taken.store(false, Ordering::Release);
        }
    }
}

fn cancellation_from_code(code: u8) -> Option<Cancellation> {
    match code {
        1 => Some(Cancellation::Caller),
        2 => Some(Cancellation::Parent),
        3 => Some(Cancellation::Timeout),
        4 => Some(Cancellation::BackendTeardown),
        _ => None,
    }
}

#[allow(dead_code)]
fn _waker_typecheck(_: &Waker) {}
