use framework_contacts::{
    Contacts, ContactsAuthorization, ContactsBackend, ContactsError, ContactsExportBackend,
};
use framework_core::Availability;

struct ByteBackend {
    bytes: Vec<u8>,
}

impl ContactsBackend for ByteBackend {
    type RequestAuthorizationFuture<'a> =
        core::future::Ready<Result<ContactsAuthorization, ContactsError>>;

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        ContactsAuthorization::Authorized
    }

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        core::future::ready(Ok(ContactsAuthorization::Authorized))
    }
}

impl ContactsExportBackend for ByteBackend {
    fn export_contact_vcard(&self, identifier: &str) -> Result<Vec<u8>, ContactsError> {
        assert_eq!(identifier, "selected-contact");
        Ok(self.bytes.clone())
    }
}

#[test]
fn facade_returns_exported_vcard_bytes_without_text_conversion() {
    let expected = b"BEGIN:VCARD\r\nX-RAW:\0\xff\r\nEND:VCARD\r\n".to_vec();
    let contacts = Contacts::new(ByteBackend {
        bytes: expected.clone(),
    });

    assert_eq!(
        contacts.export_contact_vcard("selected-contact").unwrap(),
        expected
    );
}
