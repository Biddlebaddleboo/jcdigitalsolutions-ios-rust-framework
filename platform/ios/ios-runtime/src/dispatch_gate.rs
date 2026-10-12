use core::sync::atomic::{AtomicU8, Ordering};

const PENDING: u8 = 0;
const CANCELLED: u8 = 1;
const RUNNING: u8 = 2;
const COMPLETE: u8 = 3;

#[derive(Debug)]
pub(crate) struct DispatchTaskGate {
    state: AtomicU8,
}

impl DispatchTaskGate {
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU8::new(PENDING),
        }
    }

    pub(crate) fn cancel(&self) -> bool {
        self.state
            .compare_exchange(PENDING, CANCELLED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn try_start(&self) -> bool {
        self.state
            .compare_exchange(PENDING, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn finish(&self) {
        self.state.store(COMPLETE, Ordering::Release);
    }
}
