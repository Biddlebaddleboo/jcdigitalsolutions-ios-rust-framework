use core::task::{Context, Poll, Waker};
use framework_nfc::{NdefMessage, NdefReadError};

pub(super) struct NdefReadState {
    started: bool,
    settled: bool,
    result: Option<Result<NdefMessage, NdefReadError>>,
    waker: Option<Waker>,
}

impl NdefReadState {
    pub(super) const fn new() -> Self {
        Self {
            started: false,
            settled: false,
            result: None,
            waker: None,
        }
    }

    pub(super) fn start(&mut self) -> Result<(), NdefReadError> {
        if self.started {
            return Err(NdefReadError::AlreadyStarted);
        }
        self.started = true;
        Ok(())
    }

    pub(super) fn settle(
        &mut self,
        result: Result<NdefMessage, NdefReadError>,
    ) -> (bool, Option<Waker>) {
        if !self.started || self.settled {
            return (false, None);
        }
        self.result = Some(result);
        self.settled = true;
        (true, self.waker.take())
    }

    pub(super) fn cancel(&mut self) -> (bool, Option<Waker>) {
        self.settle(Err(NdefReadError::Cancelled))
    }

    pub(super) fn poll_result(
        &mut self,
        context: &Context<'_>,
    ) -> Poll<Result<NdefMessage, NdefReadError>> {
        if self.settled {
            return Poll::Ready(
                self.result
                    .take()
                    .unwrap_or(Err(NdefReadError::NativeFailure)),
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
        self.started && !self.settled
    }
}

pub(super) const fn map_core_nfc_error(domain_is_core_nfc: bool, code: i64) -> NdefReadError {
    if !domain_is_core_nfc {
        return NdefReadError::NativeFailure;
    }
    match code {
        1 => NdefReadError::UnsupportedOrNotProvisioned,
        2 => NdefReadError::SecurityRequirementsUnsatisfied,
        6 => NdefReadError::RadioDisabled,
        7 => NdefReadError::Ineligible,
        8 => NdefReadError::AccessDenied,
        200 => NdefReadError::Cancelled,
        201 => NdefReadError::TimedOut,
        202 => NdefReadError::Interrupted,
        203 => NdefReadError::Busy,
        _ => NdefReadError::NativeFailure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_start_and_first_terminal_result_win() {
        let mut state = NdefReadState::new();
        let message = NdefMessage::default();

        assert_eq!(state.start(), Ok(()));
        assert_eq!(state.start(), Err(NdefReadError::AlreadyStarted));
        assert!(state.settle(Ok(message.clone())).0);
        assert!(!state.settle(Err(NdefReadError::Cancelled)).0);
        assert_eq!(state.result.take(), Some(Ok(message)));
        assert!(!state.is_active());
    }

    #[test]
    fn cancel_settles_once_and_ignores_late_native_callbacks() {
        let mut state = NdefReadState::new();
        assert!(!state.cancel().0);
        assert_eq!(state.start(), Ok(()));
        assert!(state.cancel().0);
        assert!(!state.cancel().0);
        assert!(!state.settle(Ok(NdefMessage::default())).0);
        assert_eq!(state.result.take(), Some(Err(NdefReadError::Cancelled)));
    }

    #[test]
    fn native_completion_before_start_is_discarded() {
        let mut state = NdefReadState::new();
        assert!(!state.settle(Err(NdefReadError::Unsupported)).0);
        assert_eq!(state.result, None);
    }

    #[test]
    fn core_nfc_access_and_lifecycle_errors_remain_distinct() {
        assert_eq!(
            map_core_nfc_error(true, 1),
            NdefReadError::UnsupportedOrNotProvisioned
        );
        assert_eq!(
            map_core_nfc_error(true, 2),
            NdefReadError::SecurityRequirementsUnsatisfied
        );
        assert_eq!(map_core_nfc_error(true, 8), NdefReadError::AccessDenied);
        assert_eq!(map_core_nfc_error(true, 200), NdefReadError::Cancelled);
        assert_eq!(map_core_nfc_error(true, 201), NdefReadError::TimedOut);
        assert_eq!(map_core_nfc_error(true, 203), NdefReadError::Busy);
        assert_eq!(map_core_nfc_error(false, 8), NdefReadError::NativeFailure);
        assert_eq!(map_core_nfc_error(true, 999), NdefReadError::NativeFailure);
    }

    struct WakeCapture(core::sync::atomic::AtomicBool);

    impl std::task::Wake for WakeCapture {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.store(true, core::sync::atomic::Ordering::SeqCst);
        }

        fn wake_by_ref(self: &std::sync::Arc<Self>) {
            self.0.store(true, core::sync::atomic::Ordering::SeqCst);
        }
    }

    #[test]
    fn result_settlement_wakes_the_registered_task() {
        let wake_capture =
            std::sync::Arc::new(WakeCapture(core::sync::atomic::AtomicBool::new(false)));
        let waker = std::task::Waker::from(std::sync::Arc::clone(&wake_capture));
        let context = Context::from_waker(&waker);
        let mut state = NdefReadState::new();
        assert_eq!(state.start(), Ok(()));
        assert_eq!(state.poll_result(&context), Poll::Pending);

        let (settled, wake) = state.settle(Ok(NdefMessage::default()));
        assert!(settled);
        assert!(!wake_capture.0.load(core::sync::atomic::Ordering::SeqCst));
        wake.expect("pending task waker").wake();
        assert!(wake_capture.0.load(core::sync::atomic::Ordering::SeqCst));
        assert_eq!(
            state.poll_result(&context),
            Poll::Ready(Ok(NdefMessage::default()))
        );
    }
}
