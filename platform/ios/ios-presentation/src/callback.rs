use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) struct OneShot<F> {
    callback: RefCell<Option<F>>,
}

impl<F> OneShot<F> {
    pub(crate) fn new(callback: F) -> Self {
        Self {
            callback: RefCell::new(Some(callback)),
        }
    }
}

impl<F: FnOnce()> OneShot<F> {
    pub(crate) fn invoke(&self) -> bool {
        let callback = self.callback.borrow_mut().take();
        if let Some(callback) = callback {
            callback();
            true
        } else {
            false
        }
    }
}

pub(crate) fn catch_foreign_panic<F: FnOnce()>(callback: F) -> bool {
    catch_unwind(AssertUnwindSafe(callback)).is_ok()
}

pub(crate) fn return_callback_on_error<E, F>(
    result: Result<(), E>,
    callback: Option<F>,
) -> Result<(), (E, F)> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err((
            error,
            callback.expect("a preflight error must retain the uncalled callback"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{OneShot, catch_foreign_panic, return_callback_on_error};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn one_shot_invokes_callback_at_most_once() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let callback = OneShot::new(move || callback_calls.set(callback_calls.get() + 1));

        assert!(callback.invoke());
        assert!(!callback.invoke());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn dropping_unselected_callback_drops_its_state() {
        struct DropFlag(Rc<Cell<bool>>);

        impl Drop for DropFlag {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let dropped = Rc::new(Cell::new(false));
        let flag = DropFlag(Rc::clone(&dropped));
        let callback = OneShot::new(move || drop(flag));

        drop(callback);
        assert!(dropped.get());
    }

    #[test]
    fn callback_panic_is_contained_and_consumed_once() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let callback = OneShot::new(move || {
            callback_calls.set(callback_calls.get() + 1);
            panic!("callback panic regression");
        });

        assert!(!catch_foreign_panic(|| {
            callback.invoke();
        }));
        assert!(catch_foreign_panic(|| {
            callback.invoke();
        }));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn invalid_preflight_returns_callback_uninvoked() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let callback = move || callback_calls.set(callback_calls.get() + 1);
        let (error, callback) =
            return_callback_on_error(Err("invalid anchor"), Some(callback)).unwrap_err();

        assert_eq!(error, "invalid anchor");
        assert_eq!(calls.get(), 0);
        callback();
        assert_eq!(calls.get(), 1);
    }
}
