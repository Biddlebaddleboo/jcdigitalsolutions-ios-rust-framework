use crate::queue::EventQueue;
use std::sync::{Mutex, PoisonError, Weak};

type CallbackBoundary = for<'a> fn(&'a mut (dyn FnMut() + 'a));

pub(crate) fn make_observer_callback(
    events: Weak<Mutex<EventQueue>>,
    update: impl Fn(&mut EventQueue) + 'static,
    boundary: CallbackBoundary,
) -> impl Fn() + 'static {
    move || {
        let mut callback = || {
            let Some(events) = events.upgrade() else {
                return;
            };
            let mut events = events.lock().unwrap_or_else(PoisonError::into_inner);
            update(&mut events);
        };
        boundary(&mut callback);
    }
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
    use crate::queue::LifecycleNotification;
    use framework_app::ApplicationState;
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
        notification: LifecycleNotification,
    ) -> impl Fn() + 'static {
        make_observer_callback(
            events,
            move |events| events.apply(notification),
            test_boundary,
        )
    }

    struct FakeCenter {
        removed: Rc<RefCell<Vec<u8>>>,
        callbacks: FakeCallbacks,
    }

    struct FakeBackend {
        _observers: ObserverOwner<FakeCenter, u8, 1>,
        _events: Arc<Mutex<EventQueue>>,
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
        let callback = state_callback(
            Arc::downgrade(&events),
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
        let events_weak = Arc::downgrade(&events);
        let removed = Rc::new(RefCell::new(Vec::new()));
        let callbacks = Rc::new(RefCell::new(Vec::new()));
        let center = FakeCenter {
            removed: Rc::clone(&removed),
            callbacks: Rc::clone(&callbacks),
        };
        let callback: FakeCallback = Rc::new(state_callback(
            events_weak.clone(),
            LifecycleNotification::DidEnterBackground,
        ));
        let token = center.register(Rc::clone(&callback));
        let backend = FakeBackend {
            _observers: ObserverOwner::new(center, [token]),
            _events: events,
        };
        let late_callback = Rc::clone(&callbacks.borrow()[0].1);

        drop(backend);
        assert_eq!(*removed.borrow(), [token]);
        assert!(events_weak.upgrade().is_none());
        late_callback();
        assert!(events_weak.upgrade().is_none());
    }

    #[test]
    fn callback_boundary_contains_panics_from_event_updates() {
        let events = Arc::new(Mutex::new(EventQueue::new(Some(
            ApplicationState::Foreground,
        ))));
        let callback = make_observer_callback(
            Arc::downgrade(&events),
            |_| panic!("observer callback regression fixture"),
            test_boundary,
        );

        callback();

        assert_eq!(
            events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop(),
            None
        );
    }
}
