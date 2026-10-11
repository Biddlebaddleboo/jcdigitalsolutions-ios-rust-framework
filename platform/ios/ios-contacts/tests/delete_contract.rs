use core::future::{Ready, ready};
use framework_contacts::{
    Contacts, ContactsAuthorization, ContactsBackend, ContactsDeleteBackend, ContactsError,
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

struct DeleteBackend {
    calls: Rc<RefCell<Vec<String>>>,
}

impl ContactsBackend for DeleteBackend {
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

impl ContactsDeleteBackend for DeleteBackend {
    fn delete_contact(&self, identifier: &str) -> Result<(), ContactsError> {
        self.calls.borrow_mut().push(String::from(identifier));
        Ok(())
    }
}

#[test]
fn authorization_only_backend_does_not_need_delete_extension() {
    let contacts = Contacts::new(AuthorizationOnlyBackend);
    assert_eq!(
        contacts.authorization_status(),
        ContactsAuthorization::Authorized
    );
}

#[test]
fn delete_extension_forwards_the_exact_single_identifier() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let contacts = Contacts::new(DeleteBackend {
        calls: calls.clone(),
    });

    assert_eq!(contacts.delete_contact("selected-local-id"), Ok(()));
    assert_eq!(*calls.borrow(), vec![String::from("selected-local-id")]);
}
