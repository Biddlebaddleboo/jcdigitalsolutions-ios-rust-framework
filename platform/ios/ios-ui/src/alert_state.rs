use core::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

/// A terminal result from one native alert presentation.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IosAlertResult {
    /// The user selected the action with this insertion-order ID.
    ActionSelected {
        /// The stable ID returned by IosAlertSession::add_action.
        action_id: u32,
        /// One exact Rust UTF-8 value per configured field; None preserves native nil.
        text_fields: Vec<Option<String>>,
    },
    /// The alert was cancelled, dismissed interactively, or dismissed by the caller.
    Cancelled,
    /// UIKit returned text that cannot be represented as valid Rust UTF-8.
    Failed(IosAlertError),
}

/// An error from configuring or presenting a native UIKit alert.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosAlertError {
    /// An action title must be non-empty and within the per-string limit.
    InvalidInput,
    /// The alert action limit was reached.
    ActionLimitReached,
    /// The alert text-field limit was reached.
    TextFieldLimitReached,
    /// Text fields can only be added to alert-style controllers.
    TextFieldsRequireAlertStyle,
    /// At least one action is required before presentation.
    NoActions,
    /// UIAlertController supports one cancellation action.
    CancelActionAlreadyAdded,
    /// The text-field UTF-16 bound must be between 1 and 4096 units.
    InvalidTextFieldLimit,
    /// A popover source rectangle has invalid coordinates or dimensions.
    InvalidPopoverAnchor,
    /// The presenter is detached, already presenting, or otherwise unavailable.
    PresenterUnavailable,
    /// The caller attempted to mutate or present a session after its first presentation.
    AlreadyPresented,
    /// UIKit did not supply the requested popover presentation controller.
    PopoverUnavailable,
    /// The current call did not have a UIKit main-thread proof.
    MainThreadUnavailable,
    /// UIKit did not return the text field just added to the alert.
    TextFieldUnavailable,
    /// A fallible allocation failed.
    ResourceExhausted,
    /// Native text contained invalid UTF-16.
    InvalidNativeText,
}

/// A failed alert start with the caller's callback returned intact.
pub struct IosAlertStartError<F> {
    /// The synchronous reason the alert did not start.
    pub error: IosAlertError,
    pub(crate) callback: F,
}

impl<F> IosAlertStartError<F> {
    /// Separates the error from the callback that was not consumed.
    pub fn into_parts(self) -> (IosAlertError, F) {
        (self.error, self.callback)
    }
}

type AlertCallback = Box<dyn FnOnce(IosAlertResult)>;

/// Main-thread-only shared callback state retained by UIKit action Blocks.
#[derive(Clone, Default)]
pub(crate) struct AlertCompletion {
    callback: Rc<RefCell<Option<AlertCallback>>>,
}

impl AlertCompletion {
    pub(crate) fn install<F>(&self, callback: F) -> Result<(), F>
    where
        F: FnOnce(IosAlertResult) + 'static,
    {
        let Ok(mut slot) = self.callback.try_borrow_mut() else {
            return Err(callback);
        };
        if slot.is_some() {
            return Err(callback);
        }
        *slot = Some(Box::new(callback));
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn is_active(&self) -> bool {
        self.callback.try_borrow().is_ok_and(|slot| slot.is_some())
    }

    pub(crate) fn complete(&self, result: IosAlertResult) -> bool {
        let Ok(mut slot) = self.callback.try_borrow_mut() else {
            return false;
        };
        let Some(callback) = slot.take() else {
            return false;
        };
        drop(slot);
        let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
        true
    }

    pub(crate) fn detach(&self) -> bool {
        self.callback
            .try_borrow_mut()
            .ok()
            .and_then(|mut slot| slot.take())
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::{AlertCompletion, IosAlertError, IosAlertResult, IosAlertStartError};

    #[test]
    fn selection_completes_once_with_the_selected_action_and_fields() {
        let completion = AlertCompletion::default();
        let selected = Rc::new(RefCell::new(None));
        let selected_in_callback = Rc::clone(&selected);
        assert!(
            completion
                .install(move |result| *selected_in_callback.borrow_mut() = Some(result))
                .is_ok()
        );

        let action = IosAlertResult::ActionSelected {
            action_id: 2,
            text_fields: vec![Some("é\0🚗".to_owned()), None],
        };
        assert!(completion.complete(action.clone()));
        assert!(!completion.complete(IosAlertResult::Cancelled));
        assert_eq!(*selected.borrow(), Some(action));
        assert!(!completion.is_active());
    }

    #[test]
    fn cancellation_and_detach_release_the_callback_exactly_once() {
        let completion = AlertCompletion::default();
        let dropped = Rc::new(Cell::new(false));
        struct DropFlag(Rc<Cell<bool>>);
        impl Drop for DropFlag {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }
        let flag = DropFlag(Rc::clone(&dropped));
        assert!(completion.install(move |_| drop(flag)).is_ok());
        assert!(completion.complete(IosAlertResult::Cancelled));
        assert!(dropped.get());
        assert!(!completion.detach());

        let completion = AlertCompletion::default();
        let dropped = Rc::new(Cell::new(false));
        let flag = DropFlag(Rc::clone(&dropped));
        assert!(completion.install(move |_| drop(flag)).is_ok());
        assert!(completion.detach());
        assert!(dropped.get());
        assert!(!completion.is_active());
    }

    #[test]
    fn callback_panic_is_contained_and_state_is_already_detached() {
        let completion = AlertCompletion::default();
        assert!(
            completion
                .install(|_| panic!("alert callback regression"))
                .is_ok()
        );
        assert!(completion.complete(IosAlertResult::Cancelled));
        assert!(!completion.is_active());
    }

    #[test]
    fn failed_start_returns_callback_without_consuming_it() {
        let called = Rc::new(Cell::new(false));
        let called_in_callback = Rc::clone(&called);
        let error = IosAlertStartError {
            error: IosAlertError::NoActions,
            callback: move |_| called_in_callback.set(true),
        };
        let (reason, callback) = error.into_parts();
        assert_eq!(reason, IosAlertError::NoActions);
        callback(IosAlertResult::Cancelled);
        assert!(called.get());
    }

    #[test]
    fn independent_sessions_do_not_share_completion_state() {
        let first = AlertCompletion::default();
        let second = AlertCompletion::default();
        let first_result = Rc::new(Cell::new(false));
        let second_result = Rc::new(Cell::new(false));
        let first_result_in_callback = Rc::clone(&first_result);
        let second_result_in_callback = Rc::clone(&second_result);
        assert!(
            first
                .install(move |_| first_result_in_callback.set(true))
                .is_ok()
        );
        assert!(
            second
                .install(move |_| second_result_in_callback.set(true))
                .is_ok()
        );
        assert!(first.complete(IosAlertResult::Cancelled));
        assert!(first_result.get());
        assert!(!second_result.get());
        assert!(second.is_active());
    }
}
