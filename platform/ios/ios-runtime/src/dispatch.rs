//! One-shot handoff to UIKit's main thread.

use crate::dispatch_gate::DispatchTaskGate;
use crate::ffi::catch_unwind;
use crate::main_thread::MainThread;
use dispatch2::DispatchQueue;
use std::sync::Arc;

/// Whether a main-thread callback ran inline or was submitted to the main queue.
#[must_use = "distinguish inline execution from a queued handoff"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainThreadHandoff {
    /// The caller was already on the main thread and the callback ran before return.
    RanInline,
    /// The callback was transferred to the main queue and may run after return.
    Enqueued,
}

/// The result of a main-thread handoff that supports explicit cancellation while queued.
#[must_use = "retain the queued task handle when cancellation may be needed"]
#[derive(Debug)]
pub enum CancellableMainThreadHandoff {
    /// The caller was already on main, so the callback completed inline and cannot be cancelled.
    RanInline,
    /// The callback was queued; cancellation succeeds only before the callback starts.
    Enqueued(MainThreadTaskHandle),
}

/// A best-effort cancellation handle for a callback queued on the main dispatch queue.
#[derive(Debug)]
pub struct MainThreadTaskHandle {
    gate: Arc<DispatchTaskGate>,
}

impl MainThreadTaskHandle {
    /// Cancel the callback if it has not started.
    ///
    /// Returns `true` only when this call prevents the callback from starting. Returns `false`
    /// after another cancellation won or the callback claimed execution. Cancellation does not
    /// remove the block from libdispatch and cannot stop a callback already running. Dropping
    /// this handle does not cancel its task.
    pub fn cancel(&self) -> bool {
        self.gate.cancel()
    }
}

/// Run a one-shot callback inline on the main thread or enqueue it on the main dispatch queue.
///
/// The callback must be `Send + 'static` because an off-main caller transfers ownership to
/// libdispatch. This API cannot transfer closures that capture non-`Send` values. The callback is
/// consumed once; enqueued work has no cancellation handle. A panic is caught and not reported to
/// the caller on the queued path, and the panic hook still runs. Inline execution uses ordinary
/// Rust panic propagation. If this function is called from a C or Objective-C callback, keep the
/// inline invocation inside the caller's existing `ios_runtime::ffi::catch_unwind` boundary so a
/// panic cannot unwind through that foreign callback. With `panic=abort`, the process aborts as
/// usual.
pub fn run_on_main(callback: impl FnOnce(MainThread) + Send + 'static) -> MainThreadHandoff {
    if let Some(main_thread) = MainThread::current() {
        callback(main_thread);
        MainThreadHandoff::RanInline
    } else {
        DispatchQueue::main().exec_async(move || {
            // SAFETY: libdispatch executes blocks submitted to the main queue on the process
            // main thread, which satisfies MainThread's proof invariant.
            let main_thread = unsafe { MainThread::assume_current() };
            let _ = catch_unwind(|| callback(main_thread));
        });
        MainThreadHandoff::Enqueued
    }
}

/// Run a one-shot callback inline on main or enqueue it with an explicit cancellation handle.
///
/// The callback has the same `Send + 'static`, main-thread, and panic behavior as
/// [`run_on_main`]. On the queued path, `MainThreadTaskHandle::cancel` races atomically with
/// callback start; it reports whether cancellation prevented the callback from starting. The
/// callback still occupies its libdispatch block until dequeued, even when cancelled.
pub fn run_on_main_cancellable(
    callback: impl FnOnce(MainThread) + Send + 'static,
) -> CancellableMainThreadHandoff {
    if let Some(main_thread) = MainThread::current() {
        callback(main_thread);
        CancellableMainThreadHandoff::RanInline
    } else {
        let gate = Arc::new(DispatchTaskGate::new());
        let queued_gate = Arc::clone(&gate);
        DispatchQueue::main().exec_async(move || {
            if !queued_gate.try_start() {
                return;
            }
            // SAFETY: libdispatch executes blocks submitted to the main queue on the process
            // main thread, which satisfies MainThread's proof invariant.
            let main_thread = unsafe { MainThread::assume_current() };
            let _ = catch_unwind(|| callback(main_thread));
            queued_gate.finish();
        });
        CancellableMainThreadHandoff::Enqueued(MainThreadTaskHandle { gate })
    }
}
