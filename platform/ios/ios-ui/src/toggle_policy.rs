use core::cell::RefCell;

type ToggleCallback = Box<dyn FnMut(bool)>;

#[cfg(target_os = "ios")]
fn catch_toggle_panic(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_toggle_panic(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

pub(crate) fn invoke_toggle_callback(action: &RefCell<ToggleCallback>, on: bool) {
    catch_toggle_panic(|| {
        if let Ok(mut callback) = action.try_borrow_mut() {
            callback(on);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{ToggleCallback, invoke_toggle_callback};
    use core::cell::{Cell, RefCell};
    use std::boxed::Box;
    use std::rc::{Rc, Weak};

    type CallbackCell = RefCell<ToggleCallback>;
    type Callback = Rc<CallbackCell>;
    type WeakCallback = Weak<CallbackCell>;
    type CallbackSlot = Rc<RefCell<Option<WeakCallback>>>;

    #[test]
    fn callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("toggle callback panic") }) as Box<dyn FnMut(bool)>
        );
        invoke_toggle_callback(&callback, true);
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_callback_is_ignored_while_mutably_borrowed() {
        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(move |on| {
            nested_calls.set(nested_calls.get() + 1);
            if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                invoke_toggle_callback(&callback, !on);
            }
        })));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_toggle_callback(&callback, true);
        assert_eq!(calls.get(), 1);
    }
}
