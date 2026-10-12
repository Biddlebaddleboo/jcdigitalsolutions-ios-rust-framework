use core::task::{Context, Poll, Waker};
use framework_nfc::NdefWriteError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Idle,
    AwaitingTag,
    Connecting,
    QueryingStatus,
    Writing,
    Settled,
}

pub(super) struct NdefWriteState {
    started: bool,
    phase: Phase,
    result: Option<Result<(), NdefWriteError>>,
    waker: Option<Waker>,
}

impl NdefWriteState {
    pub(super) const fn new() -> Self {
        Self {
            started: false,
            phase: Phase::Idle,
            result: None,
            waker: None,
        }
    }

    pub(super) fn start(&mut self) -> Result<(), NdefWriteError> {
        if self.started {
            return Err(NdefWriteError::AlreadyStarted);
        }
        self.started = true;
        self.phase = Phase::AwaitingTag;
        Ok(())
    }

    pub(super) fn begin_connect(&mut self) -> bool {
        if self.phase != Phase::AwaitingTag {
            return false;
        }
        self.phase = Phase::Connecting;
        true
    }

    pub(super) fn begin_status_query(&mut self) -> bool {
        if self.phase != Phase::Connecting {
            return false;
        }
        self.phase = Phase::QueryingStatus;
        true
    }

    pub(super) fn begin_write(&mut self) -> bool {
        if self.phase != Phase::QueryingStatus {
            return false;
        }
        self.phase = Phase::Writing;
        true
    }

    pub(super) fn settle(&mut self, result: Result<(), NdefWriteError>) -> (bool, Option<Waker>) {
        if !self.started || self.phase == Phase::Settled {
            return (false, None);
        }
        self.phase = Phase::Settled;
        self.result = Some(result);
        (true, self.waker.take())
    }

    pub(super) fn cancel(&mut self) -> (bool, Option<Waker>) {
        self.settle(Err(NdefWriteError::Cancelled))
    }

    pub(super) fn poll_result(
        &mut self,
        context: &Context<'_>,
    ) -> Poll<Result<(), NdefWriteError>> {
        if self.phase == Phase::Settled {
            return Poll::Ready(
                self.result
                    .take()
                    .unwrap_or(Err(NdefWriteError::NativeFailure)),
            );
        }
        if !self
            .waker
            .as_ref()
            .is_some_and(|waker| waker.will_wake(context.waker()))
        {
            self.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    pub(super) const fn is_active(&self) -> bool {
        self.started && !matches!(self.phase, Phase::Idle | Phase::Settled)
    }
}

pub(super) const fn tag_status_error(status: u8) -> Option<NdefWriteError> {
    match status {
        1 => Some(NdefWriteError::TagNotSupported),
        2 => None,
        3 => Some(NdefWriteError::ReadOnly),
        _ => Some(NdefWriteError::NativeFailure),
    }
}

pub(super) const fn message_exceeds_capacity(message_length: u32, capacity: usize) -> bool {
    message_length as usize > capacity
}

pub(super) const fn map_core_nfc_write_error(
    domain_is_core_nfc: bool,
    code: i64,
) -> NdefWriteError {
    if !domain_is_core_nfc {
        return NdefWriteError::NativeFailure;
    }
    match code {
        1 => NdefWriteError::UnsupportedOrNotProvisioned,
        2 => NdefWriteError::SecurityRequirementsUnsatisfied,
        6 => NdefWriteError::RadioDisabled,
        7 => NdefWriteError::Ineligible,
        8 => NdefWriteError::AccessDenied,
        200 => NdefWriteError::Cancelled,
        201 => NdefWriteError::TimedOut,
        202 => NdefWriteError::Interrupted,
        203 => NdefWriteError::Busy,
        _ => NdefWriteError::NativeFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::task::Context;

    #[test]
    fn write_stages_accept_only_the_one_shot_sequence() {
        let mut state = NdefWriteState::new();
        assert!(!state.begin_connect());
        assert_eq!(state.start(), Ok(()));
        assert_eq!(state.start(), Err(NdefWriteError::AlreadyStarted));
        assert!(state.begin_connect());
        assert!(!state.begin_connect());
        assert!(state.begin_status_query());
        assert!(!state.begin_status_query());
        assert!(state.begin_write());
        assert!(!state.begin_write());
        assert!(state.settle(Ok(())).0);
        assert!(!state.settle(Err(NdefWriteError::Cancelled)).0);
        assert_eq!(state.result, Some(Ok(())));
        assert!(!state.is_active());
    }

    #[test]
    fn cancellation_settles_once_and_ignores_late_native_results() {
        let mut state = NdefWriteState::new();
        assert!(!state.cancel().0);
        assert_eq!(state.start(), Ok(()));
        assert!(state.begin_connect());
        assert!(state.cancel().0);
        assert!(!state.cancel().0);
        assert!(!state.begin_status_query());
        assert!(!state.settle(Ok(())).0);
        assert_eq!(state.result, Some(Err(NdefWriteError::Cancelled)));
    }

    #[test]
    fn terminal_result_wakes_the_registered_task_once() {
        struct WakeCapture(std::sync::atomic::AtomicBool);

        impl std::task::Wake for WakeCapture {
            fn wake(self: std::sync::Arc<Self>) {
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            }

            fn wake_by_ref(self: &std::sync::Arc<Self>) {
                self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }

        let wake_capture =
            std::sync::Arc::new(WakeCapture(std::sync::atomic::AtomicBool::new(false)));
        let waker = std::task::Waker::from(std::sync::Arc::clone(&wake_capture));
        let context = Context::from_waker(&waker);
        let mut state = NdefWriteState::new();
        assert_eq!(state.start(), Ok(()));
        assert_eq!(state.poll_result(&context), Poll::Pending);

        let (settled, wake) = state.settle(Err(NdefWriteError::ReadOnly));
        assert!(settled);
        assert!(!wake_capture.0.load(std::sync::atomic::Ordering::SeqCst));
        wake.expect("pending task waker").wake();
        assert!(wake_capture.0.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            state.poll_result(&context),
            Poll::Ready(Err(NdefWriteError::ReadOnly))
        );
    }

    #[test]
    fn tag_status_and_capacity_policy_rejects_non_writable_targets() {
        assert_eq!(tag_status_error(1), Some(NdefWriteError::TagNotSupported));
        assert_eq!(tag_status_error(2), None);
        assert_eq!(tag_status_error(3), Some(NdefWriteError::ReadOnly));
        assert_eq!(tag_status_error(4), Some(NdefWriteError::NativeFailure));
        assert!(!message_exceeds_capacity(128, 128));
        assert!(message_exceeds_capacity(129, 128));
    }

    #[test]
    fn core_nfc_write_failures_keep_access_and_lifecycle_reasons() {
        assert_eq!(
            map_core_nfc_write_error(true, 1),
            NdefWriteError::UnsupportedOrNotProvisioned
        );
        assert_eq!(
            map_core_nfc_write_error(true, 2),
            NdefWriteError::SecurityRequirementsUnsatisfied
        );
        assert_eq!(
            map_core_nfc_write_error(true, 6),
            NdefWriteError::RadioDisabled
        );
        assert_eq!(
            map_core_nfc_write_error(true, 7),
            NdefWriteError::Ineligible
        );
        assert_eq!(
            map_core_nfc_write_error(true, 8),
            NdefWriteError::AccessDenied
        );
        assert_eq!(
            map_core_nfc_write_error(true, 200),
            NdefWriteError::Cancelled
        );
        assert_eq!(
            map_core_nfc_write_error(true, 201),
            NdefWriteError::TimedOut
        );
        assert_eq!(
            map_core_nfc_write_error(true, 202),
            NdefWriteError::Interrupted
        );
        assert_eq!(map_core_nfc_write_error(true, 203), NdefWriteError::Busy);
        assert_eq!(
            map_core_nfc_write_error(false, 8),
            NdefWriteError::NativeFailure
        );
        assert_eq!(
            map_core_nfc_write_error(true, 999),
            NdefWriteError::NativeFailure
        );
    }
}
