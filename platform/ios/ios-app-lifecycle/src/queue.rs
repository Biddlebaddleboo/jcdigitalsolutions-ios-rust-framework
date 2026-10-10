use framework_app::{ApplicationState, LifecycleEvent};

pub(crate) const EVENT_QUEUE_CAPACITY: usize = 16;

#[derive(Clone, Copy)]
pub(crate) enum LifecycleNotification {
    DidBecomeActive,
    WillResignActive,
    WillEnterForeground,
    DidEnterBackground,
    WillTerminate,
}

pub(crate) struct EventQueue {
    events: [Option<LifecycleEvent>; EVENT_QUEUE_CAPACITY],
    head: usize,
    len: usize,
    last_state: Option<ApplicationState>,
    terminating: bool,
}

impl EventQueue {
    pub(crate) const fn new(initial_state: Option<ApplicationState>) -> Self {
        Self {
            events: [None; EVENT_QUEUE_CAPACITY],
            head: 0,
            len: 0,
            last_state: initial_state,
            terminating: false,
        }
    }

    pub(crate) fn apply(&mut self, notification: LifecycleNotification) {
        if self.terminating && !matches!(notification, LifecycleNotification::WillTerminate) {
            return;
        }
        match notification {
            LifecycleNotification::DidBecomeActive => {
                self.state_changed(ApplicationState::Foreground)
            }
            LifecycleNotification::WillResignActive
            | LifecycleNotification::WillEnterForeground => {
                self.state_changed(ApplicationState::Inactive)
            }
            LifecycleNotification::DidEnterBackground => {
                self.state_changed(ApplicationState::Background)
            }
            LifecycleNotification::WillTerminate => {
                if !self.terminating {
                    self.last_state = Some(ApplicationState::Terminating);
                    self.terminating = true;
                    self.push(LifecycleEvent::TerminationRequested);
                }
            }
        }
    }

    pub(crate) const fn is_terminating(&self) -> bool {
        self.terminating
    }

    pub(crate) fn pop(&mut self) -> Option<LifecycleEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % EVENT_QUEUE_CAPACITY;
        self.len -= 1;
        event
    }

    fn state_changed(&mut self, state: ApplicationState) {
        if self.last_state == Some(state) {
            return;
        }
        self.last_state = Some(state);
        self.push(LifecycleEvent::StateChanged(state));
    }

    fn push(&mut self, event: LifecycleEvent) {
        if self.len == EVENT_QUEUE_CAPACITY {
            self.events[self.head] = None;
            self.head = (self.head + 1) % EVENT_QUEUE_CAPACITY;
            self.len -= 1;
        }
        let tail = (self.head + self.len) % EVENT_QUEUE_CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_event(state: ApplicationState) -> LifecycleEvent {
        LifecycleEvent::StateChanged(state)
    }

    #[test]
    fn lifecycle_notifications_map_to_portable_states() {
        let mut events = EventQueue::new(Some(ApplicationState::Launching));
        events.apply(LifecycleNotification::DidBecomeActive);
        events.apply(LifecycleNotification::WillResignActive);
        events.apply(LifecycleNotification::WillEnterForeground);
        events.apply(LifecycleNotification::DidEnterBackground);
        assert_eq!(
            events.pop(),
            Some(state_event(ApplicationState::Foreground))
        );
        assert_eq!(events.pop(), Some(state_event(ApplicationState::Inactive)));
        assert_eq!(
            events.pop(),
            Some(state_event(ApplicationState::Background))
        );
        assert_eq!(events.pop(), None);
    }

    #[test]
    fn consecutive_duplicate_state_changes_are_coalesced() {
        let mut events = EventQueue::new(Some(ApplicationState::Foreground));
        events.apply(LifecycleNotification::DidBecomeActive);
        events.apply(LifecycleNotification::WillResignActive);
        events.apply(LifecycleNotification::WillEnterForeground);
        assert_eq!(events.pop(), Some(state_event(ApplicationState::Inactive)));
        assert_eq!(events.pop(), None);
    }

    #[test]
    fn termination_reports_request_without_claiming_completion() {
        let mut events = EventQueue::new(Some(ApplicationState::Background));
        events.apply(LifecycleNotification::WillTerminate);
        events.apply(LifecycleNotification::DidBecomeActive);
        events.apply(LifecycleNotification::WillTerminate);
        assert!(events.is_terminating());
        assert_eq!(events.pop(), Some(LifecycleEvent::TerminationRequested));
        assert_eq!(events.pop(), None);
    }

    #[test]
    fn overflow_drops_oldest_event_and_preserves_remaining_order() {
        let mut events = EventQueue::new(None);
        let states = [
            ApplicationState::Foreground,
            ApplicationState::Inactive,
            ApplicationState::Background,
        ];
        for index in 0..=EVENT_QUEUE_CAPACITY {
            events.state_changed(states[index % states.len()]);
        }
        assert_eq!(events.len, EVENT_QUEUE_CAPACITY);
        for index in 1..=6 {
            assert_eq!(
                events.pop(),
                Some(state_event(states[index % states.len()]))
            );
        }
        for index in EVENT_QUEUE_CAPACITY + 1..=EVENT_QUEUE_CAPACITY + 6 {
            events.state_changed(states[index % states.len()]);
        }
        assert_eq!(events.len, EVENT_QUEUE_CAPACITY);
        for index in 7..=EVENT_QUEUE_CAPACITY + 6 {
            assert_eq!(
                events.pop(),
                Some(state_event(states[index % states.len()]))
            );
        }
        assert_eq!(events.len, 0);
    }
}
