//! One-shot handoff to UIKit's main thread.

use crate::ffi::catch_unwind;
use crate::main_thread::MainThread;
use dispatch2::DispatchQueue;

/// Whether a main-thread callback ran inline or was submitted to the main queue.
#[must_use = "distinguish inline execution from a queued handoff"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MainThreadHandoff {
    /// The caller was already on the main thread and the callback ran before return.
    RanInline,
    /// The callback was transferred to the main queue and may run after return.
    Enqueued,
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
