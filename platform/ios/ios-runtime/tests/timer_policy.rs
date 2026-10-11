#[path = "../src/timer_policy.rs"]
mod timer_policy;

use core::time::Duration;
use timer_policy::{
    PanicContained, TickOutcome, TimerError, TimerLifecycle, interval_nanos, leeway_nanos,
};

#[test]
fn timer_durations_reject_zero_and_native_overflow() {
    assert_eq!(
        interval_nanos(Duration::ZERO),
        Err(TimerError::ZeroInterval)
    );
    assert_eq!(
        interval_nanos(Duration::MAX),
        Err(TimerError::IntervalTooLong)
    );
    assert_eq!(leeway_nanos(Duration::MAX), Err(TimerError::LeewayTooLong));
    assert_eq!(interval_nanos(Duration::from_nanos(1)), Ok(1));
    assert_eq!(
        interval_nanos(Duration::from_nanos(i64::MAX as u64)),
        Ok(i64::MAX as u64)
    );
    assert_eq!(
        interval_nanos(Duration::from_nanos(i64::MAX as u64 + 1)),
        Err(TimerError::IntervalTooLong)
    );
    assert_eq!(leeway_nanos(Duration::ZERO), Ok(0));
    assert_eq!(leeway_nanos(Duration::from_nanos(u64::MAX)), Ok(u64::MAX));
    assert_eq!(
        leeway_nanos(Duration::new(u64::MAX / 1_000_000_000 + 1, 0)),
        Err(TimerError::LeewayTooLong)
    );
}

#[test]
fn timer_lifecycle_runs_repeated_ticks_until_cancel() {
    let lifecycle = TimerLifecycle::new();
    let mut ticks = 0;
    assert_eq!(lifecycle.invoke_tick(|| ticks += 1), TickOutcome::Invoked);
    assert_eq!(
        lifecycle.invoke_tick(|| {
            ticks += 1;
            lifecycle.cancel();
        }),
        TickOutcome::Cancelled
    );
    assert_eq!(lifecycle.invoke_tick(|| ticks += 1), TickOutcome::Skipped);
    assert_eq!(ticks, 2);
}

#[test]
fn timer_lifecycle_stops_after_callback_panic() {
    let lifecycle = TimerLifecycle::new();
    assert_eq!(
        lifecycle.invoke_tick(|| panic!("timer callback regression")),
        TickOutcome::Panicked
    );
    assert_eq!(lifecycle.invoke_tick(|| {}), TickOutcome::Skipped);
}

#[test]
fn timer_lifecycle_cancel_is_idempotent() {
    let lifecycle = TimerLifecycle::new();
    assert!(lifecycle.cancel());
    assert!(!lifecycle.cancel());
    assert!(!lifecycle.is_active());
}

#[test]
fn callback_destructor_panic_is_contained() {
    struct PanicOnDrop;

    impl Drop for PanicOnDrop {
        fn drop(&mut self) {
            panic!("timer callback destructor regression");
        }
    }

    let contained = PanicContained::new(PanicOnDrop);
    let _ = contained.get();
    drop(contained);
}
