use crate::queue::EventQueue;
use framework_app::LifecycleEvent;
use std::sync::{Mutex, PoisonError, Weak};

type CallbackBoundary = for<'a> fn(&'a mut (dyn FnMut() + 'a));

pub(crate) fn make_observer_callback(
    events: Weak<Mutex<EventQueue>>,
    handler: Weak<LifecycleEventHandlerState>,
    notification: crate::queue::LifecycleNotification,
    boundary: CallbackBoundary,
) -> impl Fn() + 'static {
    move || {
        let mut callback = || {
            let Some(events) = events.upgrade() else {
                return;
            };
            let event = events
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .apply(notification);
            if let (Some(event), Some(handler)) = (event, handler.upgrade()) {
                handler.dispatch(event);
            }
        };
        boundary(&mut callback);
    }
}

pub(crate) const HANDLER_PENDING_CAPACITY: usize = 16;

type LifecycleEventHandler = Box<dyn FnMut(LifecycleEvent) + Send + 'static>;

struct HandlerDispatch {
    handler: Option<LifecycleEventHandler>,
    pending: [Option<LifecycleEvent>; HANDLER_PENDING_CAPACITY],
    head: usize,
    len: usize,
    dispatching: bool,
    cancel_active: bool,
    dropped: u64,
}

/// Shared callback state for the native observer blocks
pub(crate) struct LifecycleEventHandlerState {
    dispatch: Mutex<HandlerDispatch>,
}

impl LifecycleEventHandlerState {
    pub(crate) const fn new() -> Self {
        Self {
            dispatch: Mutex::new(HandlerDispatch {
                handler: None,
                pending: [None; HANDLER_PENDING_CAPACITY],
                head: 0,
                len: 0,
                dispatching: false,
                cancel_active: false,
                dropped: 0,
            }),
        }
    }

    pub(crate) fn set(&self, handler: impl FnMut(LifecycleEvent) + Send + 'static) {
        let old_handler = {
            let mut dispatch = self.dispatch.lock().unwrap_or_else(PoisonError::into_inner);
            dispatch.clear_pending();
            if dispatch.dispatching {
                dispatch.cancel_active = true;
            }
            dispatch.handler.replace(Box::new(handler))
        };
        drop(old_handler);
    }

    pub(crate) fn clear(&self) {
        let old_handler = {
            let mut dispatch = self.dispatch.lock().unwrap_or_else(PoisonError::into_inner);
            dispatch.clear_pending();
            if dispatch.dispatching {
                dispatch.cancel_active = true;
            }
            dispatch.handler.take()
        };
        drop(old_handler);
    }

    pub(crate) fn dropped(&self) -> u64 {
        self.dispatch
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .dropped
    }

    fn dispatch(&self, event: LifecycleEvent) {
        let mut handler = {
            let mut dispatch = self.dispatch.lock().unwrap_or_else(PoisonError::into_inner);
            if dispatch.handler.is_none() && (!dispatch.dispatching || dispatch.cancel_active) {
                return;
            }
            dispatch.push_pending(event);
            if dispatch.dispatching {
                return;
            }
            dispatch.dispatching = true;
            let Some(handler) = dispatch.handler.take() else {
                dispatch.dispatching = false;
                dispatch.clear_pending();
                return;
            };
            handler
        };

        loop {
            enum Step {
                Event(LifecycleEvent),
                Replace(LifecycleEventHandler),
                Stop,
            }

            let step = {
                let mut dispatch = self.dispatch.lock().unwrap_or_else(PoisonError::into_inner);
                if dispatch.cancel_active {
                    dispatch.cancel_active = false;
                    if let Some(replacement) = dispatch.handler.take() {
                        Step::Replace(replacement)
                    } else {
                        dispatch.clear_pending();
                        dispatch.dispatching = false;
                        Step::Stop
                    }
                } else if let Some(event) = dispatch.pop_pending() {
                    Step::Event(event)
                } else {
                    dispatch.handler = Some(handler);
                    dispatch.dispatching = false;
                    return;
                }
            };

            match step {
                Step::Event(event) => {
                    let mut callback = || handler(event);
                    contain_handler_panic(&mut callback);
                }
                Step::Replace(replacement) => {
                    drop(handler);
                    handler = replacement;
                }
                Step::Stop => return,
            }
        }
    }
}

impl HandlerDispatch {
    fn push_pending(&mut self, event: LifecycleEvent) {
        if self.len == HANDLER_PENDING_CAPACITY {
            self.pending[self.head] = None;
            self.head = (self.head + 1) % HANDLER_PENDING_CAPACITY;
            self.len -= 1;
            self.dropped = self.dropped.saturating_add(1);
        }
        let tail = (self.head + self.len) % HANDLER_PENDING_CAPACITY;
        self.pending[tail] = Some(event);
        self.len += 1;
    }

    fn pop_pending(&mut self) -> Option<LifecycleEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.pending[self.head].take();
        self.head = (self.head + 1) % HANDLER_PENDING_CAPACITY;
        self.len -= 1;
        event
    }

    fn clear_pending(&mut self) {
        self.pending.fill(None);
        self.head = 0;
        self.len = 0;
    }
}

#[cfg(target_os = "ios")]
fn contain_handler_panic(callback: &mut dyn FnMut()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(all(test, not(target_os = "ios")))]
fn contain_handler_panic(callback: &mut dyn FnMut()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

pub(crate) trait ObserverRemover<T> {
    fn remove(&self, observer: &T);
}

pub(crate) struct ObserverOwner<C, T, const N: usize>
where
    C: ObserverRemover<T>,
{
    remover: C,
    observers: [T; N],
}

impl<C, T, const N: usize> ObserverOwner<C, T, N>
where
    C: ObserverRemover<T>,
{
    pub(crate) const fn new(remover: C, observers: [T; N]) -> Self {
        Self { remover, observers }
    }
}

impl<C, T, const N: usize> Drop for ObserverOwner<C, T, N>
where
    C: ObserverRemover<T>,
{
    fn drop(&mut self) {
        for observer in &self.observers {
            self.remover.remove(observer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{EVENT_QUEUE_CAPACITY, LifecycleNotification};
    use framework_app::ApplicationState;
    use framework_app::LifecycleEvent;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    type FakeCallback = Rc<dyn Fn()>;
    type FakeCallbacks = Rc<RefCell<Vec<(u8, FakeCallback)>>>;

    fn test_boundary(callback: &mut (dyn FnMut() + '_)) {
        let _ = std::panic::catch_unwind(AssertUnwindSafe(callback));
    }

    fn state_callback(
        events: Weak<Mutex<EventQueue>>,
        handler: Weak<LifecycleEventHandlerState>,
        notification: LifecycleNotification,
    ) -> impl Fn() + 'static {
        make_observer_callback(events, handler, notification, test_boundary)
    }

    struct FakeCenter {
        removed: Rc<RefCell<Vec<u8>>>,
        callbacks: FakeCallbacks,
    }

    struct FakeBackend {
        _observers: ObserverOwner<FakeCenter, u8, 1>,
        _events: Arc<Mutex<EventQueue>>,
        _handler: Arc<LifecycleEventHandlerState>,
    }

    impl FakeCenter {
        fn register(&self, callback: Rc<dyn Fn()>) -> u8 {
            let token = 7;
            self.callbacks.borrow_mut().push((token, callback));
            token
        }
    }

    impl ObserverRemover<u8> for FakeCenter {
        fn remove(&self, observer: &u8) {
            self.removed.borrow_mut().push(*observer);
        }
    }

    #[test]
    fn owner_removes_each_observer_once_in_registration_order() {
        let removed = Rc::new(RefCell::new(Vec::new()));
        let owner = ObserverOwner::new(
            FakeCenter {
                removed: Rc::clone(&removed),
                callbacks: Rc::new(RefCell::new(Vec::new())),
            },
            [2, 4, 6, 8, 10],
        );
        assert!(removed.borrow().is_empty());
        drop(owner);
        assert_eq!(*removed.borrow(), [2, 4, 6, 8, 10]);
    }

    #[test]
    fn callback_updates_live_event_state() {
        let events = Arc::new(Mutex::new(EventQueue::new(Some(
            ApplicationState::Foreground,
        ))));
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback = state_callback(
            Arc::downgrade(&events),
            Arc::downgrade(&handler),
            LifecycleNotification::DidEnterBackground,
        );

        callback();

        assert_eq!(
            events.lock().unwrap().pop(),
            Some(framework_app::LifecycleEvent::StateChanged(
                ApplicationState::Background
            ))
        );
    }

    #[test]
    fn callback_does_not_retain_or_update_event_state_after_drop() {
        let events = Arc::new(Mutex::new(EventQueue::new(Some(
            ApplicationState::Foreground,
        ))));
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let handled = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_handled = Arc::clone(&handled);
        handler.set(move |_| {
            callback_handled.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let events_weak = Arc::downgrade(&events);
        let handler_weak = Arc::downgrade(&handler);
        let removed = Rc::new(RefCell::new(Vec::new()));
        let callbacks = Rc::new(RefCell::new(Vec::new()));
        let center = FakeCenter {
            removed: Rc::clone(&removed),
            callbacks: Rc::clone(&callbacks),
        };
        let callback: FakeCallback = Rc::new(state_callback(
            events_weak.clone(),
            handler_weak.clone(),
            LifecycleNotification::DidEnterBackground,
        ));
        let token = center.register(Rc::clone(&callback));
        let backend = FakeBackend {
            _observers: ObserverOwner::new(center, [token]),
            _events: events,
            _handler: handler,
        };
        let late_callback = Rc::clone(&callbacks.borrow()[0].1);

        drop(backend);
        assert_eq!(*removed.borrow(), [token]);
        assert!(events_weak.upgrade().is_none());
        assert!(handler_weak.upgrade().is_none());
        late_callback();
        assert!(events_weak.upgrade().is_none());
        assert!(handler_weak.upgrade().is_none());
        assert_eq!(handled.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn direct_handler_runs_after_queue_update_and_polling_stays_available() {
        let events = Arc::new(Mutex::new(EventQueue::new(Some(
            ApplicationState::Foreground,
        ))));
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback_events = Arc::clone(&events);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |event| {
            callback_received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
            assert_eq!(
                callback_events
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .pop(),
                Some(event)
            );
        });
        let callback = state_callback(
            Arc::downgrade(&events),
            Arc::downgrade(&handler),
            LifecycleNotification::DidEnterBackground,
        );

        callback();

        assert_eq!(
            *received.lock().unwrap_or_else(PoisonError::into_inner),
            [LifecycleEvent::StateChanged(ApplicationState::Background)]
        );
        assert_eq!(events.lock().unwrap().pop(), None);
    }

    #[test]
    fn direct_handler_receives_events_after_poll_queue_overflow() {
        let events = Arc::new(Mutex::new(EventQueue::new(None)));
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |event| {
            callback_received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
        });
        let states = [
            ApplicationState::Foreground,
            ApplicationState::Inactive,
            ApplicationState::Background,
        ];
        let notifications = [
            LifecycleNotification::DidBecomeActive,
            LifecycleNotification::WillResignActive,
            LifecycleNotification::DidEnterBackground,
        ];
        let count = EVENT_QUEUE_CAPACITY + 5;

        for index in 0..count {
            state_callback(
                Arc::downgrade(&events),
                Arc::downgrade(&handler),
                notifications[index % notifications.len()],
            )();
        }

        let received = received.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(received.len(), count);
        assert!(received.iter().enumerate().all(|(index, event)| {
            *event == LifecycleEvent::StateChanged(states[index % states.len()])
        }));
        let mut events = events.lock().unwrap_or_else(PoisonError::into_inner);
        for index in 5..count {
            assert_eq!(
                events.pop(),
                Some(LifecycleEvent::StateChanged(states[index % states.len()]))
            );
        }
        assert_eq!(events.pop(), None);
    }

    #[test]
    fn reentrant_handler_delivery_is_serialized_and_bounded() {
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback_handler = Arc::downgrade(&handler);
        let active = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_active = Arc::clone(&active);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |event| {
            assert!(!callback_active.swap(true, std::sync::atomic::Ordering::SeqCst));
            callback_received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
            if event == LifecycleEvent::StateChanged(ApplicationState::Foreground) {
                let handler = callback_handler.upgrade().unwrap();
                for index in 1..=HANDLER_PENDING_CAPACITY {
                    handler.dispatch(if index % 2 == 0 {
                        LifecycleEvent::StateChanged(ApplicationState::Background)
                    } else {
                        LifecycleEvent::StateChanged(ApplicationState::Inactive)
                    });
                }
                assert_eq!(
                    callback_received
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .len(),
                    1
                );
            }
            callback_active.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Foreground));

        let received = received.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(received.len(), HANDLER_PENDING_CAPACITY + 1);
        assert_eq!(
            received[0],
            LifecycleEvent::StateChanged(ApplicationState::Foreground)
        );
        assert_eq!(handler.dropped(), 0);

        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback_handler = Arc::downgrade(&handler);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |event| {
            callback_received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
            if event == LifecycleEvent::StateChanged(ApplicationState::Foreground) {
                let handler = callback_handler.upgrade().unwrap();
                for index in 0..=(HANDLER_PENDING_CAPACITY) {
                    handler.dispatch(if index % 2 == 0 {
                        LifecycleEvent::StateChanged(ApplicationState::Inactive)
                    } else {
                        LifecycleEvent::StateChanged(ApplicationState::Background)
                    });
                }
            }
        });
        handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Foreground));
        let received = received.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(received.len(), HANDLER_PENDING_CAPACITY + 1);
        assert_eq!(handler.dropped(), 1);
        handler.clear();
        handler.set(|_| {});
        assert_eq!(handler.dropped(), 1);
    }

    #[test]
    fn clear_rejects_nested_and_later_stale_events() {
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback_handler = Arc::downgrade(&handler);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |event| {
            callback_received
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
            let handler = callback_handler.upgrade().unwrap();
            handler.clear();
            handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Background));
        });

        handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Foreground));
        handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Inactive));

        assert_eq!(
            *received.lock().unwrap_or_else(PoisonError::into_inner),
            [LifecycleEvent::StateChanged(ApplicationState::Foreground)]
        );
    }

    #[test]
    fn replacement_does_not_restore_the_current_handler_or_deliver_stale_backlog() {
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let callback_handler = Arc::downgrade(&handler);
        let old_calls = Arc::new(Mutex::new(Vec::new()));
        let callback_old_calls = Arc::clone(&old_calls);
        let new_calls = Arc::new(Mutex::new(Vec::new()));
        let callback_new_calls = Arc::clone(&new_calls);
        handler.set(move |event| {
            callback_old_calls
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(event);
            let handler = callback_handler.upgrade().unwrap();
            handler.set({
                let callback_new_calls = Arc::clone(&callback_new_calls);
                move |event| {
                    callback_new_calls
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(event);
                }
            });
            handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Background));
        });

        handler.dispatch(LifecycleEvent::StateChanged(ApplicationState::Foreground));

        assert_eq!(
            *old_calls.lock().unwrap_or_else(PoisonError::into_inner),
            [LifecycleEvent::StateChanged(ApplicationState::Foreground)]
        );
        assert_eq!(
            *new_calls.lock().unwrap_or_else(PoisonError::into_inner),
            [LifecycleEvent::StateChanged(ApplicationState::Background)]
        );
    }

    #[test]
    fn callback_panic_is_contained_and_handler_remains_installed() {
        let events = Arc::new(Mutex::new(EventQueue::new(Some(
            ApplicationState::Foreground,
        ))));
        let handler = Arc::new(LifecycleEventHandlerState::new());
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_calls = Arc::clone(&calls);
        handler.set(move |_| {
            if callback_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                panic!("lifecycle handler panic regression");
            }
        });

        state_callback(
            Arc::downgrade(&events),
            Arc::downgrade(&handler),
            LifecycleNotification::DidEnterBackground,
        )();
        state_callback(
            Arc::downgrade(&events),
            Arc::downgrade(&handler),
            LifecycleNotification::WillResignActive,
        )();

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(
            events.lock().unwrap().pop(),
            Some(LifecycleEvent::StateChanged(ApplicationState::Background))
        );
        assert_eq!(
            events.lock().unwrap().pop(),
            Some(LifecycleEvent::StateChanged(ApplicationState::Inactive))
        );
    }
}
