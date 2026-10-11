use core::cell::RefCell;
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;

use framework_core::{Error, ErrorKind, Result};

/// One owned action in a native UIKit context menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosContextMenuAction {
    /// Fixed-width identity passed to the selection callback.
    pub id: u64,
    /// Display title for this action.
    pub title: String,
}

impl IosContextMenuAction {
    /// Creates an owned context-menu action with a fixed-width identity and title.
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

pub(crate) fn validate_actions(actions: &[IosContextMenuAction]) -> Result<()> {
    if actions.is_empty() || actions.iter().any(|action| action.title.is_empty()) {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    let mut ids = Vec::new();
    ids.try_reserve_exact(actions.len())
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    ids.extend(actions.iter().map(|action| action.id));
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::new(ErrorKind::InvalidInput));
    }
    Ok(())
}

pub(crate) type ContextMenuCallback = Box<dyn FnMut(u64)>;

pub(crate) fn invoke_action_callback(callback: &RefCell<ContextMenuCallback>, id: u64) {
    catch_callback(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(id);
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
    use super::{IosContextMenuAction, invoke_action_callback, validate_actions};
    use core::cell::{Cell, RefCell};
    use framework_core::ErrorKind;
    use std::boxed::Box;
    use std::rc::{Rc, Weak};
    use std::string::String;

    #[test]
    fn accepts_distinct_fixed_width_action_ids() {
        assert!(
            validate_actions(&[
                IosContextMenuAction::new(0, "Zero"),
                IosContextMenuAction::new(u64::MAX, "Maximum"),
            ])
            .is_ok()
        );
    }

    #[test]
    fn rejects_duplicate_action_ids() {
        let error = validate_actions(&[
            IosContextMenuAction::new(17, "First"),
            IosContextMenuAction::new(17, "Second"),
        ])
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn rejects_empty_menu_or_empty_action_title_before_attachment() {
        assert_eq!(
            validate_actions(&[]).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            validate_actions(&[IosContextMenuAction::new(1, "")])
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn selection_callback_receives_the_action_id() {
        let selected = Rc::new(Cell::new(None));
        let callback_selected = selected.clone();
        let callback = RefCell::new(
            Box::new(move |id: u64| callback_selected.set(Some(id))) as Box<dyn FnMut(u64)>
        );
        invoke_action_callback(&callback, 0x1020_3040_5060_7080);
        assert_eq!(selected.get(), Some(0x1020_3040_5060_7080));
    }

    #[test]
    fn callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("context-menu callback panic") }) as Box<dyn FnMut(u64)>,
        );
        invoke_action_callback(&callback, 1);
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_callback_is_ignored_while_mutably_borrowed() {
        type Callback = Rc<RefCell<Box<dyn FnMut(u64)>>>;
        type CallbackSlot = Rc<RefCell<Option<Weak<RefCell<Box<dyn FnMut(u64)>>>>>>;

        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(move |id| {
            nested_calls.set(nested_calls.get() + 1);
            if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                invoke_action_callback(&callback, id);
            }
        })));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_action_callback(&callback, 9);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn action_title_is_owned() {
        let title = String::from("Owned");
        let action = IosContextMenuAction::new(2, &title);
        drop(title);
        assert_eq!(action.title, "Owned");
    }
}
