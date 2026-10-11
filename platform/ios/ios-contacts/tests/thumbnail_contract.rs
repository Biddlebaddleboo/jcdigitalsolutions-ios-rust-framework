use core::cell::{Cell, RefCell};
use core::future::Ready;
use framework_contacts::{
    Contacts, ContactsAuthorization, ContactsBackend, ContactsError, ContactsImageBackend,
};
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use std::rc::Rc;

struct ThumbnailBackend {
    status: ContactsAuthorization,
    bytes: Rc<RefCell<Option<Vec<u8>>>>,
    requested: Rc<RefCell<Vec<String>>>,
    reads: Rc<Cell<u32>>,
    native_error: Option<ContactsError>,
}

struct ThumbnailFixture {
    backend: ThumbnailBackend,
    stored_bytes: Rc<RefCell<Option<Vec<u8>>>>,
    requested: Rc<RefCell<Vec<String>>>,
    reads: Rc<Cell<u32>>,
}

impl ContactsBackend for ThumbnailBackend {
    type RequestAuthorizationFuture<'a> = Ready<Result<ContactsAuthorization, ContactsError>>;

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        self.status
    }

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        core::future::ready(Ok(self.status))
    }
}

impl ContactsImageBackend for ThumbnailBackend {
    fn fetch_contact_thumbnail(&self, identifier: &str) -> Result<Option<Vec<u8>>, ContactsError> {
        self.requested.borrow_mut().push(identifier.into());
        if !self.status.allows_contact_access() {
            return Err(ContactsError::AuthorizationRequired(self.status));
        }
        self.reads.set(self.reads.get() + 1);
        if let Some(error) = self.native_error {
            return Err(error);
        }
        Ok(self.bytes.borrow().clone())
    }
}

fn backend(
    status: ContactsAuthorization,
    bytes: Option<Vec<u8>>,
    native_error: Option<ContactsError>,
) -> ThumbnailFixture {
    let stored_bytes = Rc::new(RefCell::new(bytes));
    let requested = Rc::new(RefCell::new(Vec::new()));
    let reads = Rc::new(Cell::new(0));
    ThumbnailFixture {
        backend: ThumbnailBackend {
            status,
            bytes: stored_bytes.clone(),
            requested: requested.clone(),
            reads: reads.clone(),
            native_error,
        },
        stored_bytes,
        requested,
        reads,
    }
}

#[test]
fn facade_fetches_only_the_selected_id_and_returns_owned_raw_bytes() {
    let expected = vec![0x00, 0xff, 0x89, 0x50, 0x4e, 0x47, 0x00];
    let fixture = backend(ContactsAuthorization::Limited, Some(expected.clone()), None);
    let contacts = Contacts::new(fixture.backend);

    let actual = contacts
        .fetch_contact_thumbnail("caller-selected-local-id")
        .unwrap()
        .unwrap();

    assert_eq!(
        &*fixture.requested.borrow(),
        &[String::from("caller-selected-local-id")]
    );
    fixture.stored_bytes.borrow_mut().as_mut().unwrap()[0] = 0x7f;
    assert_eq!(actual, expected);
}

#[test]
fn absent_thumbnail_is_none() {
    let fixture = backend(ContactsAuthorization::Authorized, None, None);
    let contacts = Contacts::new(fixture.backend);

    assert_eq!(contacts.fetch_contact_thumbnail("selected-id"), Ok(None));
    assert_eq!(fixture.reads.get(), 1);
}

#[test]
fn denied_status_returns_before_thumbnail_read() {
    let fixture = backend(ContactsAuthorization::Denied, None, None);
    let contacts = Contacts::new(fixture.backend);

    assert_eq!(
        contacts.fetch_contact_thumbnail("selected-id"),
        Err(ContactsError::AuthorizationRequired(
            ContactsAuthorization::Denied
        ))
    );
    assert_eq!(fixture.reads.get(), 0);
}

#[test]
fn native_backend_error_is_preserved() {
    let error =
        Error::new(ErrorKind::Platform).with_platform_code(PlatformErrorCode::new(-47).unwrap());
    let expected = ContactsError::Backend(error);
    let fixture = backend(ContactsAuthorization::Authorized, None, Some(expected));
    let contacts = Contacts::new(fixture.backend);

    assert_eq!(
        contacts.fetch_contact_thumbnail("selected-id"),
        Err(expected)
    );
}
