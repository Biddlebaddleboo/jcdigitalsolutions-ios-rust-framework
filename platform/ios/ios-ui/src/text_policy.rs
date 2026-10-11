use core::cell::RefCell;

use framework_core::Result;

pub(crate) type TextFieldCallback = Box<dyn FnMut(Result<Option<String>>)>;

#[cfg(target_os = "ios")]
fn catch_text_panic(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_text_panic(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

pub(crate) fn invoke_text_callback(
    callback: &RefCell<TextFieldCallback>,
    text: Result<Option<String>>,
) {
    catch_text_panic(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(text);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{TextFieldCallback, invoke_text_callback};
    use core::cell::{Cell, RefCell};
    use framework_core::{Error, ErrorKind, Result};
    use std::boxed::Box;
    use std::rc::{Rc, Weak};

    type CallbackCell = RefCell<TextFieldCallback>;
    type Callback = Rc<CallbackCell>;
    type WeakCallback = Weak<CallbackCell>;
    type CallbackSlot = Rc<RefCell<Option<WeakCallback>>>;

    #[test]
    fn callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("text callback panic") }) as TextFieldCallback
        );
        invoke_text_callback(&callback, Ok(Some(String::from("edit"))));
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_callback_is_ignored_while_mutably_borrowed() {
        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(
            move |text: Result<Option<String>>| {
                nested_calls.set(nested_calls.get() + 1);
                if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                    invoke_text_callback(&callback, text.clone());
                }
            },
        )));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_text_callback(&callback, Ok(Some(String::from("edit"))));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn invalid_utf16_error_reaches_callback_unchanged() {
        let received = Rc::new(Cell::new(None::<Error>));
        let action_received = received.clone();
        let callback = RefCell::new(Box::new(move |text: Result<Option<String>>| {
            action_received.set(Some(text.unwrap_err()));
        }) as TextFieldCallback);
        let invalid = Error::new(ErrorKind::InvalidInput);

        invoke_text_callback(&callback, Err(invalid));
        assert_eq!(received.get(), Some(invalid));
    }
}
