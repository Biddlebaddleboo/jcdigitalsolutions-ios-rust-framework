use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Configuration errors for a repeating dispatch timer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TimerError {
    /// The repeat interval must be greater than zero.
    ZeroInterval,
    /// The interval does not fit the signed libdispatch clock delta.
    IntervalTooLong,
    /// The leeway does not fit the unsigned libdispatch timer field.
    LeewayTooLong,
}

pub(super) fn interval_nanos(interval: Duration) -> Result<u64, TimerError> {
    let nanos = interval.as_nanos();
    if nanos == 0 {
        return Err(TimerError::ZeroInterval);
    }
    if nanos > i64::MAX as u128 {
        return Err(TimerError::IntervalTooLong);
    }
    Ok(nanos as u64)
}

pub(super) fn leeway_nanos(leeway: Duration) -> Result<u64, TimerError> {
    let nanos = leeway.as_nanos();
    if nanos > u64::MAX as u128 {
        return Err(TimerError::LeewayTooLong);
    }
    Ok(nanos as u64)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TickOutcome {
    Skipped,
    Invoked,
    Cancelled,
    Panicked,
}

pub(super) struct PanicContained<T>(Option<T>);

impl<T> PanicContained<T> {
    pub(super) fn new(value: T) -> Self {
        Self(Some(value))
    }

    pub(super) fn get(&self) -> &T {
        self.0
            .as_ref()
            .expect("callback remains owned until Block release")
    }
}

impl<T> Drop for PanicContained<T> {
    fn drop(&mut self) {
        if let Some(value) = self.0.take() {
            let _ = catch_unwind(AssertUnwindSafe(|| drop(value)));
        }
    }
}

pub(super) struct TimerLifecycle {
    active: AtomicBool,
}

impl TimerLifecycle {
    pub(super) const fn new() -> Self {
        Self {
            active: AtomicBool::new(true),
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub(super) fn cancel(&self) -> bool {
        self.active.swap(false, Ordering::AcqRel)
    }

    pub(super) fn invoke_tick(&self, callback: impl FnOnce()) -> TickOutcome {
        if !self.is_active() {
            return TickOutcome::Skipped;
        }
        match catch_unwind(AssertUnwindSafe(callback)) {
            Ok(()) if self.is_active() => TickOutcome::Invoked,
            Ok(()) => TickOutcome::Cancelled,
            Err(_) => {
                self.cancel();
                TickOutcome::Panicked
            }
        }
    }
}
