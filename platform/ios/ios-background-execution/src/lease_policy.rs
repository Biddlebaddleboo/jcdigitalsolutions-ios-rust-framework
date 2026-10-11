use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

const ID_READY: u8 = 1;
const EXPIRED: u8 = 2;
const ENDED: u8 = 4;

/// One main-thread callback slot shared by the lease and UIKit's retained block.
pub(crate) struct OneShot<F>(Rc<RefCell<Option<F>>>);

impl<F> OneShot<F> {
    pub(crate) fn new(callback: F) -> Self {
        Self(Rc::new(RefCell::new(Some(callback))))
    }

    fn take(&self) -> Option<F> {
        self.0.borrow_mut().take()
    }

    /// Drops a pending callback without invocation and contains panics from captured destructors.
    pub(crate) fn cancel(&self) -> bool {
        let Some(callback) = self.take() else {
            return false;
        };
        let _ = catch_unwind(AssertUnwindSafe(|| drop(callback)));
        true
    }
}

impl<F> Clone for OneShot<F> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<F: FnOnce()> OneShot<F> {
    /// Takes the callback before invocation so reentrant calls observe an empty slot.
    pub(crate) fn call_once(&self) -> bool {
        let Some(callback) = self.take() else {
            return false;
        };
        let _ = catch_unwind(AssertUnwindSafe(callback));
        true
    }
}

/// Host-testable lifecycle state for one UIKit background-task identifier.
#[derive(Default)]
pub(crate) struct LeaseState {
    flags: AtomicU8,
    identifier: AtomicUsize,
}

impl LeaseState {
    pub(crate) fn is_expired(&self) -> bool {
        self.flags.load(Ordering::Acquire) & EXPIRED != 0
    }

    /// Records expiry, invokes the one-shot callback, then claims native end if its ID is ready.
    pub(crate) fn handle_expiration<F, C>(
        &self,
        callback: Option<&OneShot<F>>,
        main_thread_context: impl FnOnce() -> Option<C>,
        end: impl FnOnce(C, usize),
    ) where
        F: FnOnce(),
    {
        self.flags.fetch_or(EXPIRED, Ordering::AcqRel);
        if let Some(callback) = callback {
            callback.call_once();
        }
        if let Some(context) = main_thread_context() {
            if let Some(identifier) = self.end_once() {
                end(context, identifier);
            }
        }
    }

    /// Publishes the native ID and returns it only when an earlier expiry needs end.
    pub(crate) fn install_identifier(&self, identifier: usize) -> Option<usize> {
        self.identifier.store(identifier, Ordering::Release);
        let previous = self.flags.fetch_or(ID_READY, Ordering::AcqRel);
        if previous & EXPIRED != 0 {
            self.end_once()
        } else {
            None
        }
    }

    /// Claims the native ID for end once, whether from expiry, explicit end, or drop.
    pub(crate) fn end_once(&self) -> Option<usize> {
        loop {
            let current = self.flags.load(Ordering::Acquire);
            if current & ID_READY == 0 || current & ENDED != 0 {
                return None;
            }
            if self
                .flags
                .compare_exchange(
                    current,
                    current | ENDED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return Some(self.identifier.load(Ordering::Acquire));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LeaseState, OneShot};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn callback_runs_once_and_releases_captured_state() {
        struct DropProbe(Arc<AtomicUsize>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let calls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let call_count = Arc::clone(&calls);
        let probe = DropProbe(Arc::clone(&drops));
        let callback = OneShot::new(move || {
            let _ = &probe;
            call_count.fetch_add(1, Ordering::SeqCst);
        });

        assert!(callback.call_once());
        assert!(!callback.call_once());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn callback_can_reenter_after_its_slot_is_taken() {
        type Callback = Box<dyn FnOnce()>;
        type Slot = OneShot<Callback>;

        let slot_holder: Rc<RefCell<Option<Slot>>> = Rc::new(RefCell::new(None));
        let reentrant_holder = Rc::clone(&slot_holder);
        let callback: Callback = Box::new(move || {
            assert!(
                !reentrant_holder
                    .borrow()
                    .as_ref()
                    .expect("slot installed before invocation")
                    .call_once()
            );
        });
        let slot = Slot::new(callback);
        *slot_holder.borrow_mut() = Some(slot.clone());

        assert!(slot.call_once());
        assert!(!slot.call_once());
    }

    #[test]
    fn explicit_cancellation_drops_without_invocation() {
        struct DropProbe(Arc<AtomicUsize>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let calls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let call_count = Arc::clone(&calls);
        let probe = DropProbe(Arc::clone(&drops));
        let callback = OneShot::new(move || {
            let _ = &probe;
            call_count.fetch_add(1, Ordering::SeqCst);
        });

        assert!(callback.cancel());
        assert!(!callback.call_once());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn callback_panic_is_contained_and_still_consumes_slot() {
        let callback = OneShot::new(|| panic!("expiry callback failure"));

        assert!(callback.call_once());
        assert!(!callback.call_once());
    }

    #[test]
    fn expiry_before_identifier_install_ends_once_after_callback() {
        let state = Arc::new(LeaseState::default());
        let order = Arc::new(Mutex::new(Vec::new()));
        let callback_state = Arc::clone(&state);
        let callback_order = Arc::clone(&order);
        let callback = OneShot::new(move || {
            assert!(callback_state.is_expired());
            callback_order.lock().unwrap().push("callback");
        });

        state.handle_expiration(
            Some(&callback),
            || Some(()),
            |(), identifier| {
                assert_eq!(identifier, 17);
                order.lock().unwrap().push("native end");
            },
        );
        assert_eq!(*order.lock().unwrap(), ["callback"]);

        if let Some(identifier) = state.install_identifier(17) {
            assert_eq!(identifier, 17);
            order.lock().unwrap().push("native end");
        } else {
            panic!("expired lease must end when its identifier becomes ready");
        }
        assert_eq!(*order.lock().unwrap(), ["callback", "native end"]);
        assert_eq!(state.end_once(), None);
    }

    #[test]
    fn callback_panic_does_not_skip_native_end() {
        let state = LeaseState::default();
        assert_eq!(state.install_identifier(23), None);
        let callback = OneShot::new(|| panic!("expiry callback failure"));
        let end_count = AtomicUsize::new(0);

        state.handle_expiration(
            Some(&callback),
            || Some(()),
            |(), identifier| {
                assert_eq!(identifier, 23);
                end_count.fetch_add(1, Ordering::SeqCst);
            },
        );

        assert_eq!(end_count.load(Ordering::SeqCst), 1);
        assert_eq!(state.end_once(), None);
    }

    #[test]
    fn reentrant_end_during_expiry_claims_identifier_once() {
        let state = Arc::new(LeaseState::default());
        assert_eq!(state.install_identifier(31), None);
        let reentrant_state = Arc::clone(&state);
        let end_count = Arc::new(AtomicUsize::new(0));
        let callback_end_count = Arc::clone(&end_count);
        let callback = OneShot::new(move || {
            assert_eq!(reentrant_state.end_once(), Some(31));
            callback_end_count.fetch_add(1, Ordering::SeqCst);
        });

        state.handle_expiration(
            Some(&callback),
            || Some(()),
            |(), identifier| {
                assert_eq!(identifier, 31);
                end_count.fetch_add(1, Ordering::SeqCst);
            },
        );

        assert_eq!(end_count.load(Ordering::SeqCst), 1);
        assert_eq!(state.end_once(), None);
    }
}
