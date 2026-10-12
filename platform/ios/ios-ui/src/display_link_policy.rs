use core::cell::Cell;

pub(crate) struct CallbackGate {
    active: Cell<bool>,
    paused: Cell<bool>,
    in_callback: Cell<bool>,
}

impl CallbackGate {
    pub(crate) const fn new() -> Self {
        Self {
            active: Cell::new(true),
            paused: Cell::new(false),
            in_callback: Cell::new(false),
        }
    }

    pub(crate) fn pause(&self) {
        if self.active.get() {
            self.paused.set(true);
        }
    }

    pub(crate) fn resume(&self) {
        if self.active.get() {
            self.paused.set(false);
        }
    }

    pub(crate) fn invalidate(&self) {
        self.active.set(false);
        self.paused.set(true);
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.get()
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused.get()
    }

    pub(crate) fn begin_callback(&self) -> Option<CallbackPermit<'_>> {
        if !self.active.get() || self.paused.get() || self.in_callback.get() {
            return None;
        }
        self.in_callback.set(true);
        Some(CallbackPermit(self))
    }
}

pub(crate) struct CallbackPermit<'a>(&'a CallbackGate);

impl Drop for CallbackPermit<'_> {
    fn drop(&mut self) {
        self.0.in_callback.set(false);
    }
}

pub(crate) const fn valid_frame_timing(
    timestamp_seconds: f64,
    target_timestamp_seconds: f64,
    duration_seconds: f64,
) -> bool {
    timestamp_seconds.is_finite()
        && timestamp_seconds >= 0.0
        && target_timestamp_seconds.is_finite()
        && target_timestamp_seconds >= 0.0
        && duration_seconds.is_finite()
        && duration_seconds > 0.0
}

#[cfg(test)]
mod tests {
    use super::{CallbackGate, valid_frame_timing};

    #[test]
    fn frame_timing_requires_finite_nonnegative_times_and_positive_duration() {
        assert!(valid_frame_timing(1.0, 1.016, 0.016));
        assert!(valid_frame_timing(0.0, 0.016, f64::MIN_POSITIVE));
        assert!(!valid_frame_timing(-1.0, 0.016, 0.016));
        assert!(!valid_frame_timing(1.0, f64::NAN, 0.016));
        assert!(!valid_frame_timing(1.0, f64::INFINITY, 0.016));
        assert!(!valid_frame_timing(1.0, 1.016, 0.0));
        assert!(!valid_frame_timing(1.0, 1.016, -0.016));
        assert!(!valid_frame_timing(1.0, 1.016, f64::INFINITY));
    }

    #[test]
    fn pause_suppresses_callbacks_until_resume() {
        let gate = CallbackGate::new();
        assert!(gate.begin_callback().is_some());
        gate.pause();
        assert!(gate.is_paused());
        assert!(gate.begin_callback().is_none());
        gate.resume();
        assert!(!gate.is_paused());
        assert!(gate.begin_callback().is_some());
    }

    #[test]
    fn reentrant_callback_is_rejected_without_releasing_outer_permit() {
        let gate = CallbackGate::new();
        let permit = gate.begin_callback().unwrap();
        assert!(gate.begin_callback().is_none());
        drop(permit);
        assert!(gate.begin_callback().is_some());
    }

    #[test]
    fn invalidation_rejects_stale_and_future_callbacks() {
        let gate = CallbackGate::new();
        let in_flight = gate.begin_callback().unwrap();
        gate.invalidate();
        assert!(!gate.is_active());
        assert!(gate.begin_callback().is_none());
        drop(in_flight);
        assert!(gate.begin_callback().is_none());
        gate.resume();
        assert!(gate.begin_callback().is_none());
    }
}
