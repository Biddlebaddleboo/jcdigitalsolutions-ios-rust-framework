use core::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::ContactPickerEvent;

pub(super) type Callback = Box<dyn FnOnce(ContactPickerEvent)>;

pub(super) struct PickerCompletion {
    callback: RefCell<Option<Callback>>,
}

impl PickerCompletion {
    pub(super) fn new(callback: impl FnOnce(ContactPickerEvent) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(callback))),
        }
    }

    pub(super) fn is_pending(&self) -> bool {
        self.callback.borrow().is_some()
    }

    fn take(&self) -> Option<Callback> {
        self.callback.borrow_mut().take()
    }

    pub(super) fn detach(&self) -> bool {
        self.take().is_some()
    }

    pub(super) fn settle(&self, event: ContactPickerEvent) -> bool {
        let Some(callback) = self.take() else {
            return false;
        };
        Self::invoke(callback, event);
        true
    }

    pub(super) fn invoke(callback: Callback, event: ContactPickerEvent) {
        let _ = catch_unwind(AssertUnwindSafe(|| callback(event)));
    }
}

#[cfg(test)]
mod tests {
    use framework_contacts::{Contact, ContactLabeledValue};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::string::String;
    use std::vec;
    use std::vec::Vec;

    use super::PickerCompletion;
    use crate::ContactPickerEvent;

    fn contact() -> Contact {
        Contact {
            identifier: String::from("selected-id"),
            given_name: String::from("Ada"),
            family_name: String::from("Lovelace"),
            organization_name: String::from("Analytical Engines"),
            phone_numbers: vec![ContactLabeledValue {
                label: Some(String::from("mobile")),
                value: String::from("+1 555 0100"),
            }],
            email_addresses: Vec::new(),
        }
    }

    #[test]
    fn selected_event_settles_once_with_owned_contact_snapshot() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let callback_events = events.clone();
        let completion =
            PickerCompletion::new(move |event| callback_events.borrow_mut().push(event));

        assert!(completion.settle(ContactPickerEvent::Selected(contact())));
        assert!(!completion.settle(ContactPickerEvent::Cancelled));

        let events = events.borrow();
        assert_eq!(events.len(), 1);
        let ContactPickerEvent::Selected(selected) = &events[0] else {
            panic!("expected selected contact event");
        };
        assert_eq!(selected.identifier, "selected-id");
        assert_eq!(selected.given_name, "Ada");
        assert_eq!(selected.family_name, "Lovelace");
        assert_eq!(selected.organization_name, "Analytical Engines");
        assert_eq!(selected.phone_numbers[0].value, "+1 555 0100");
    }

    #[test]
    fn user_cancel_settles_once_and_ignores_later_selection() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let callback_events = events.clone();
        let completion =
            PickerCompletion::new(move |event| callback_events.borrow_mut().push(event));

        assert!(completion.settle(ContactPickerEvent::Cancelled));
        assert!(!completion.settle(ContactPickerEvent::Selected(contact())));

        let events = events.borrow();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ContactPickerEvent::Cancelled));
    }

    #[test]
    fn explicit_cancel_settles_with_cancelled_event() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let callback_events = events.clone();
        let completion =
            PickerCompletion::new(move |event| callback_events.borrow_mut().push(event));

        assert!(completion.settle(ContactPickerEvent::Cancelled));
        assert!(!completion.settle(ContactPickerEvent::Cancelled));

        assert_eq!(events.borrow().len(), 1);
        assert!(matches!(events.borrow()[0], ContactPickerEvent::Cancelled));
    }

    #[test]
    fn dropped_session_detaches_without_delivering_an_event() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let callback_events = events.clone();
        let completion =
            PickerCompletion::new(move |event| callback_events.borrow_mut().push(event));

        assert!(completion.detach());
        assert!(!completion.is_pending());
        assert!(!completion.settle(ContactPickerEvent::Cancelled));
        drop(completion);

        assert!(events.borrow().is_empty());
    }

    #[test]
    fn callback_panic_is_contained_after_terminal_settlement() {
        let completion = PickerCompletion::new(|_| panic!("callback failure"));

        assert!(completion.settle(ContactPickerEvent::Cancelled));
        assert!(!completion.is_pending());
        assert!(!completion.settle(ContactPickerEvent::Cancelled));
    }
}
