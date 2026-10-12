use std::sync::atomic::{AtomicU8, Ordering};

const PENDING: u8 = 0;
const EXPIRED: u8 = 1;
const FINISHED: u8 = 2;

pub(crate) struct CompletionState(AtomicU8);

impl CompletionState {
    pub(crate) const fn new() -> Self {
        Self(AtomicU8::new(PENDING))
    }

    pub(crate) fn is_expired(&self) -> bool {
        self.0.load(Ordering::Acquire) == EXPIRED
    }

    pub(crate) fn expire(&self) -> bool {
        self.0
            .compare_exchange(PENDING, EXPIRED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn claim_completion(&self) -> bool {
        self.0
            .compare_exchange(PENDING, FINISHED, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

pub(crate) fn completion_succeeded(
    state: &CompletionState,
    callback_result: std::thread::Result<bool>,
) -> bool {
    state.claim_completion() && callback_result.unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_claim_is_one_shot_and_prevents_late_expiry() {
        let state = CompletionState::new();
        assert!(!state.is_expired());
        assert!(state.claim_completion());
        assert!(!state.claim_completion());
        assert!(!state.expire());
        assert!(!state.is_expired());
    }

    #[test]
    fn successful_callback_claims_completion_once() {
        let state = CompletionState::new();
        assert!(completion_succeeded(&state, Ok(true)));
        assert!(!completion_succeeded(&state, Ok(true)));
        assert!(!state.expire());
    }

    #[test]
    fn expiry_wins_over_successful_callback_return() {
        let state = CompletionState::new();
        assert!(state.expire());
        assert!(!state.expire());
        assert!(state.is_expired());
        assert!(!completion_succeeded(&state, Ok(true)));
    }

    #[test]
    fn callback_failure_and_panic_map_to_native_failure() {
        let failed = CompletionState::new();
        assert!(!completion_succeeded(&failed, Ok(false)));

        let panicked = CompletionState::new();
        let result = std::panic::catch_unwind(|| panic!("processing callback panic"));
        assert!(!completion_succeeded(&panicked, result));
    }
}
