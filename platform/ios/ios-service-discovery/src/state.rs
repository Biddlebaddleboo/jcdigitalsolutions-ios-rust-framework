use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

#[cfg(test)]
use framework_service_discovery::NativeBonjourError;
use framework_service_discovery::{
    BonjourBrowseError, BonjourBrowserEvent, BonjourBrowserState, BonjourService,
    MAX_QUEUED_BONJOUR_EVENTS,
};

pub(crate) struct Shared {
    state: Mutex<BrowserState>,
}

struct BrowserState {
    started: bool,
    state: BonjourBrowserState,
    state_pending: bool,
    events: VecDeque<BonjourBrowserEvent>,
    next_waker: Option<Waker>,
    close_waker: Option<Waker>,
    cancel_requested: bool,
    close_complete: bool,
    terminal_error: Option<BonjourBrowseError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueueServiceError {
    Full,
    Allocation,
}

impl Shared {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(BrowserState {
                started: false,
                state: BonjourBrowserState::Starting,
                state_pending: false,
                events: VecDeque::new(),
                next_waker: None,
                close_waker: None,
                cancel_requested: false,
                close_complete: false,
                terminal_error: None,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, BrowserState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    pub(crate) fn begin_start(&self) -> Result<(), BonjourBrowseError> {
        let mut state = self.lock();
        if state.started || state.close_complete {
            return Err(BonjourBrowseError::InvalidState);
        }
        state
            .events
            .try_reserve_exact(MAX_QUEUED_BONJOUR_EVENTS)
            .map_err(|_| BonjourBrowseError::ResourceExhausted)?;
        state.started = true;
        Ok(())
    }

    pub(crate) fn fail_start(&self, error: BonjourBrowseError) {
        let close_waker = {
            let mut state = self.lock();
            state.state = BonjourBrowserState::Failed(None);
            state.state_pending = true;
            state.terminal_error = Some(error);
            state.close_complete = true;
            state.close_waker.take()
        };
        wake(close_waker);
    }

    pub(crate) fn push_service(
        &self,
        added: bool,
        service: BonjourService,
    ) -> Result<(), QueueServiceError> {
        let next_waker = {
            let mut state = self.lock();
            if !state.started || state.close_complete || state.cancel_requested {
                return Ok(());
            }
            if state.events.len() >= MAX_QUEUED_BONJOUR_EVENTS {
                state.terminal_error = Some(BonjourBrowseError::QueueOverflow);
                state.cancel_requested = true;
                return Err(QueueServiceError::Full);
            }
            if state.events.try_reserve(1).is_err() {
                state.terminal_error = Some(BonjourBrowseError::ResourceExhausted);
                state.cancel_requested = true;
                return Err(QueueServiceError::Allocation);
            }
            state.events.push_back(if added {
                BonjourBrowserEvent::ServiceAdded(service)
            } else {
                BonjourBrowserEvent::ServiceRemoved(service)
            });
            state.next_waker.take()
        };
        wake(next_waker);
        Ok(())
    }

    pub(crate) fn set_native_state(
        &self,
        new_state: BonjourBrowserState,
        terminal_error: Option<BonjourBrowseError>,
    ) {
        let (next_waker, close_waker) = {
            let mut state = self.lock();
            if state.close_complete {
                return;
            }
            state.state = new_state;
            state.state_pending = true;
            if let Some(error) = terminal_error {
                state.terminal_error = Some(error);
                state.cancel_requested = true;
            }
            if new_state == BonjourBrowserState::Closed {
                state.close_complete = true;
                state.cancel_requested = true;
                if state.terminal_error.is_none() {
                    state.terminal_error = Some(BonjourBrowseError::Cancelled);
                }
                (state.next_waker.take(), state.close_waker.take())
            } else {
                (state.next_waker.take(), None)
            }
        };
        wake(next_waker);
        wake(close_waker);
    }

    pub(crate) fn callback_panic(&self) {
        self.set_native_state(
            BonjourBrowserState::Failed(None),
            Some(BonjourBrowseError::BackendFailure),
        );
    }

    pub(crate) fn request_cancel(&self) -> bool {
        let (should_cancel, next_waker, close_waker) = {
            let mut state = self.lock();
            if state.close_complete || state.cancel_requested {
                return false;
            }
            state.cancel_requested = true;
            if !state.started {
                state.state = BonjourBrowserState::Closed;
                state.state_pending = true;
                state.close_complete = true;
                if state.terminal_error.is_none() {
                    state.terminal_error = Some(BonjourBrowseError::Cancelled);
                }
                (false, state.next_waker.take(), state.close_waker.take())
            } else {
                (true, state.next_waker.take(), state.close_waker.take())
            }
        };
        wake(next_waker);
        wake(close_waker);
        should_cancel
    }

    pub(crate) fn poll_next_event(
        &self,
        context: &Context<'_>,
    ) -> Poll<Result<BonjourBrowserEvent, BonjourBrowseError>> {
        let mut state = self.lock();
        if !state.started && !state.close_complete {
            return Poll::Ready(Err(BonjourBrowseError::InvalidState));
        }
        if let Some(event) = state.events.pop_front() {
            return Poll::Ready(Ok(event));
        }
        if state.state_pending {
            state.state_pending = false;
            return Poll::Ready(Ok(BonjourBrowserEvent::State(state.state)));
        }
        if let Some(error) = state.terminal_error.take() {
            return Poll::Ready(Err(error));
        }
        if state.close_complete {
            return Poll::Ready(Err(BonjourBrowseError::Cancelled));
        }
        store_waker(&mut state.next_waker, context.waker());
        Poll::Pending
    }

    pub(crate) fn poll_close(&self, context: &Context<'_>) -> Poll<Result<(), BonjourBrowseError>> {
        let mut state = self.lock();
        if state.close_complete {
            return Poll::Ready(Ok(()));
        }
        store_waker(&mut state.close_waker, context.waker());
        Poll::Pending
    }

    pub(crate) fn detach_next(&self) {
        self.lock().next_waker = None;
    }

    pub(crate) fn detach_close(&self) {
        self.lock().close_waker = None;
    }
}

fn store_waker(slot: &mut Option<Waker>, waker: &Waker) {
    if slot.as_ref().is_none_or(|stored| !stored.will_wake(waker)) {
        *slot = Some(waker.clone());
    }
}

fn wake(waker: Option<Waker>) {
    if let Some(waker) = waker {
        waker.wake();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::task::Waker;

    fn service(name: &str) -> BonjourService {
        BonjourService::new(name, "_demo._tcp", "local.").unwrap()
    }

    #[test]
    fn membership_delivery_is_fifo_and_state_is_coalesced() {
        let shared = Shared::new();
        shared.begin_start().unwrap();
        shared.set_native_state(BonjourBrowserState::Ready, None);
        shared.push_service(true, service("camera")).unwrap();
        shared.push_service(false, service("speaker")).unwrap();
        shared.set_native_state(
            BonjourBrowserState::Waiting(Some(NativeBonjourError::new(2, -65_570))),
            None,
        );
        let context = Context::from_waker(Waker::noop());
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Ok(BonjourBrowserEvent::ServiceAdded(service("camera"))))
        );
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Ok(BonjourBrowserEvent::ServiceRemoved(service("speaker"))))
        );
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Ok(BonjourBrowserEvent::State(
                BonjourBrowserState::Waiting(Some(NativeBonjourError::new(2, -65_570)))
            )))
        );
    }

    #[test]
    fn overflow_is_terminal_instead_of_silent_membership_loss() {
        let shared = Shared::new();
        shared.begin_start().unwrap();
        for index in 0..MAX_QUEUED_BONJOUR_EVENTS {
            shared
                .push_service(true, service(&format!("service-{index}")))
                .unwrap();
        }
        assert_eq!(
            shared.push_service(true, service("overflow")),
            Err(QueueServiceError::Full)
        );
        let context = Context::from_waker(Waker::noop());
        for index in 0..MAX_QUEUED_BONJOUR_EVENTS {
            assert_eq!(
                shared.poll_next_event(&context),
                Poll::Ready(Ok(BonjourBrowserEvent::ServiceAdded(service(&format!(
                    "service-{index}"
                )))))
            );
        }
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Err(BonjourBrowseError::QueueOverflow))
        );
    }

    #[test]
    fn close_completes_only_after_cancelled_native_state() {
        let shared = Shared::new();
        shared.begin_start().unwrap();
        assert!(shared.request_cancel());
        let context = Context::from_waker(Waker::noop());
        assert_eq!(shared.poll_close(&context), Poll::Pending);
        shared.set_native_state(BonjourBrowserState::Closed, None);
        assert_eq!(shared.poll_close(&context), Poll::Ready(Ok(())));
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Ok(BonjourBrowserEvent::State(BonjourBrowserState::Closed)))
        );
        assert_eq!(
            shared.poll_next_event(&context),
            Poll::Ready(Err(BonjourBrowseError::Cancelled))
        );
    }
}
