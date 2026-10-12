use core::cell::RefCell;
use std::boxed::Box;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};

pub(crate) type ActionCallback = Box<dyn FnMut()>;
pub(crate) type SharedActionCallback = Rc<RefCell<ActionCallback>>;

pub(crate) fn validate_action_input(title: &str, symbol: Option<&str>) -> Result<()> {
    if title.trim().is_empty()
        || symbol.is_some_and(|name| name.trim().is_empty() || name.contains('\0'))
    {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(())
}

pub(crate) fn callback_cell<F>(callback: F) -> SharedActionCallback
where
    F: FnMut() + 'static,
{
    Rc::new(RefCell::new(Box::new(callback)))
}

pub(crate) fn invoke_action_callback(callback: &RefCell<ActionCallback>) {
    catch_callback(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback();
        }
    });
}

#[cfg(target_os = "ios")]
fn catch_callback(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_callback(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

#[cfg(test)]
mod tests {
    use super::{callback_cell, invoke_action_callback, validate_action_input};
    use core::cell::Cell;
    use framework_core::ErrorKind;
    use std::boxed::Box;
    use std::rc::Rc;

    #[test]
    fn accepts_accessible_title_with_or_without_symbol() {
        assert!(validate_action_input("Settings", None).is_ok());
        assert!(validate_action_input(" Settings ", Some("gearshape")).is_ok());
    }

    #[test]
    fn rejects_empty_or_whitespace_action_titles() {
        for title in ["", " \t\n"] {
            assert_eq!(
                validate_action_input(title, None).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn rejects_empty_whitespace_or_nul_system_symbol_names() {
        for symbol in ["", " \t\n", "gear\0shape"] {
            assert_eq!(
                validate_action_input("Settings", Some(symbol))
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn callback_panic_is_contained_and_releases_its_borrow() {
        let callback = callback_cell(|| panic!("navigation action callback panic"));
        invoke_action_callback(&callback);
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_callback_is_ignored_while_mutably_borrowed() {
        let calls = Rc::new(Cell::new(0));
        let nested_calls = calls.clone();
        let callback = callback_cell(move || {
            nested_calls.set(nested_calls.get() + 1);
        });
        let nested_callback = Rc::clone(&callback);
        let callback_calls = calls.clone();
        *callback.borrow_mut() = Box::new(move || {
            callback_calls.set(callback_calls.get() + 1);
            invoke_action_callback(&nested_callback);
        });

        invoke_action_callback(&callback);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn copied_handler_capture_keeps_callback_alive_after_action_value_drop() {
        let calls = Rc::new(Cell::new(0));
        let callback_calls = calls.clone();
        let action_value = callback_cell(move || callback_calls.set(callback_calls.get() + 1));
        let weak_callback = Rc::downgrade(&action_value);
        let copied_handler_capture = Rc::clone(&action_value);

        drop(action_value);
        assert!(weak_callback.upgrade().is_some());
        invoke_action_callback(&copied_handler_capture);
        assert_eq!(calls.get(), 1);
        drop(copied_handler_capture);
        assert!(weak_callback.upgrade().is_none());
    }
}
