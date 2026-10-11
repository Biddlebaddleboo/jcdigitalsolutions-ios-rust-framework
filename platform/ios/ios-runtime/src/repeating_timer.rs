//! Optional per-handle repeating timers backed by public libdispatch timer sources.

use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use block2::RcBlock;
use dispatch2::{DispatchObject, DispatchQueue, DispatchRetained, DispatchSource, DispatchTime};

use crate::timer_policy::{
    PanicContained, TickOutcome, TimerLifecycle, interval_nanos, leeway_nanos,
};

pub use crate::timer_policy::TimerError;

struct TimerInner {
    lifecycle: TimerLifecycle,
    source_cancelled: AtomicBool,
    source: OnceLock<DispatchRetained<DispatchSource>>,
}

impl TimerInner {
    fn cancel(&self) -> bool {
        let was_active = self.lifecycle.cancel();
        if !self.source_cancelled.swap(true, Ordering::AcqRel) {
            if let Some(source) = self.source.get() {
                source.cancel();
            }
        }
        was_active
    }
}

/// A one-use cancellation view passed only for the duration of a timer callback.
///
/// The view does not retain the timer and cannot form a callback/source ownership cycle.
pub struct TimerContext<'a> {
    inner: &'a Weak<TimerInner>,
}

impl TimerContext<'_> {
    /// Cancel the timer without waiting for an in-flight callback to finish.
    ///
    /// Returns `true` only for the first active-to-cancelled transition.
    pub fn cancel(&self) -> bool {
        self.inner.upgrade().is_some_and(|inner| inner.cancel())
    }

    /// Return whether cancellation has not yet been requested.
    pub fn is_active(&self) -> bool {
        self.inner
            .upgrade()
            .is_some_and(|inner| inner.lifecycle.is_active())
    }
}

impl fmt::Debug for TimerContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimerContext")
            .field("is_active", &self.is_active())
            .finish()
    }
}

/// An owned handle to one repeating libdispatch timer source.
///
/// The first callback occurs after `interval`, then repeats on the supplied dispatch queue.
/// The system may delay or coalesce delivery, including within `leeway`; this is not a strict
/// cadence guarantee. Cancellation and drop are nonblocking. An event already dequeued may begin
/// or finish after cancellation returns. A callback panic is caught, the source is canceled, and
/// the panic hook still runs; with `panic=abort` the process aborts as usual.
#[must_use = "dropping the timer cancels it"]
pub struct RepeatingTimer {
    inner: Arc<TimerInner>,
}

impl RepeatingTimer {
    /// Create a repeating timer on `queue` with a monotonic first-fire delay of `interval`.
    ///
    /// Libdispatch may delay or coalesce delivery; `leeway` is not an exact scheduling promise.
    /// The timer retains its target queue through libdispatch. `callback` runs on that queue and
    /// must be `Send + 'static`; use `TimerContext::cancel` for cancellation from within a tick.
    /// The callback is serialized even if the target queue is concurrent. `interval` must be
    /// nonzero and fit `i64` nanoseconds; `leeway` must fit `u64` nanoseconds.
    pub fn schedule_on<F>(
        queue: &DispatchQueue,
        interval: Duration,
        leeway: Duration,
        callback: F,
    ) -> Result<Self, TimerError>
    where
        F: FnMut(&TimerContext<'_>) + Send + 'static,
    {
        let interval_nanos = interval_nanos(interval)?;
        let leeway_nanos = leeway_nanos(leeway)?;
        let inner = Arc::new(TimerInner {
            lifecycle: TimerLifecycle::new(),
            source_cancelled: AtomicBool::new(false),
            source: OnceLock::new(),
        });
        let weak_inner = Arc::downgrade(&inner);
        let callback = PanicContained::new(Mutex::new(callback));
        let handler = RcBlock::new(move || {
            let Some(inner) = weak_inner.upgrade() else {
                return;
            };
            let context = TimerContext { inner: &weak_inner };
            let outcome = inner.lifecycle.invoke_tick(|| {
                let mut callback = callback
                    .get()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if inner.lifecycle.is_active() {
                    callback(&context);
                }
            });
            if outcome == TickOutcome::Panicked {
                inner.cancel();
            }
        });

        // SAFETY: This is the public timer source type with its documented zero handle/mask and
        // a valid target queue. The source receives a copied Block whose captured state is
        // Send + Sync: its mutable callback lives behind a Mutex and source ownership is weak.
        let source = unsafe {
            DispatchSource::new(
                core::ptr::addr_of!(dispatch2::_dispatch_source_type_timer).cast_mut(),
                0,
                0,
                Some(queue),
            )
        };
        // SAFETY: `handler` has the no-argument void Block signature required by
        // dispatch_source_set_event_handler; its owned captures outlive this call.
        unsafe { source.set_event_handler_with_block(RcBlock::as_ptr(&handler)) };
        source.set_timer(
            DispatchTime::NOW.time(interval_nanos as i64),
            interval_nanos,
            leeway_nanos,
        );
        match inner.source.set(source) {
            Ok(()) => {}
            Err(_) => unreachable!("new timer source was already set"),
        }
        inner.source.get().expect("timer source was set").activate();
        Ok(Self { inner })
    }

    /// Cancel the source without waiting for an in-flight callback to finish.
    ///
    /// Returns `true` only for the first active-to-cancelled transition.
    pub fn cancel(&self) -> bool {
        self.inner.cancel()
    }

    /// Return whether cancellation has not yet been requested.
    pub fn is_active(&self) -> bool {
        self.inner.lifecycle.is_active()
    }
}

impl Drop for RepeatingTimer {
    fn drop(&mut self) {
        self.inner.cancel();
    }
}

impl fmt::Debug for RepeatingTimer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RepeatingTimer")
            .field("is_active", &self.is_active())
            .finish_non_exhaustive()
    }
}
