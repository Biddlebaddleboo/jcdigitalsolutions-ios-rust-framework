use core::future::{Ready, ready};
use framework_contacts::{
    ContactUpdate, Contacts, ContactsAuthorization, ContactsBackend, ContactsError,
    ContactsUpdateBackend,
};
use framework_core::Availability;
use std::cell::RefCell;
use std::rc::Rc;

struct AuthorizationOnlyBackend;

impl ContactsBackend for AuthorizationOnlyBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        ContactsAuthorization::Authorized
    }

    type RequestAuthorizationFuture<'a> = Ready<Result<ContactsAuthorization, ContactsError>>;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        ready(Ok(self.authorization_status()))
    }
}

struct UpdateBackend {
    call: Rc<RefCell<Option<(String, ContactUpdate)>>>,
}

impl ContactsBackend for UpdateBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        ContactsAuthorization::Limited
    }

    type RequestAuthorizationFuture<'a> = Ready<Result<ContactsAuthorization, ContactsError>>;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        ready(Ok(self.authorization_status()))
    }
}

impl ContactsUpdateBackend for UpdateBackend {
    fn update_contact(
        &self,
        identifier: &str,
        update: &ContactUpdate,
    ) -> Result<(), ContactsError> {
        *self.call.borrow_mut() = Some((String::from(identifier), update.clone()));
        Ok(())
    }
}

#[test]
fn authorization_only_backend_does_not_need_update_extension() {
    let contacts = Contacts::new(AuthorizationOnlyBackend);
    assert_eq!(
        contacts.authorization_status(),
        ContactsAuthorization::Authorized
    );
}

#[test]
fn update_extension_forwards_identifier_and_explicit_clear_patch() {
    let call = Rc::new(RefCell::new(None));
    let contacts = Contacts::new(UpdateBackend { call: call.clone() });
    let update = ContactUpdate {
        given_name: Some(String::from("Ada")),
        phone_numbers: Some(Vec::new()),
        ..ContactUpdate::default()
    };

    assert_eq!(
        contacts.update_contact("selected-local-id", &update),
        Ok(())
    );
    assert_eq!(
        *call.borrow(),
        Some((String::from("selected-local-id"), update))
    );
}
