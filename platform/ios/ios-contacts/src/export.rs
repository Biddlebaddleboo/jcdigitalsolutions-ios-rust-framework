use crate::conversion::{
    authorization_from_native, export_error_from_native, fetch_authorization_error,
    fetch_error_from_native, validate_identifier,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use framework_contacts::{ContactsError, ContactsExportBackend};
use objc2::rc::autoreleasepool;
use objc2_contacts::{CNContactVCardSerialization, CNErrorDomain};
use objc2_foundation::{NSArray, NSString};

impl ContactsExportBackend for IosContactsBackend {
    fn export_contact_vcard(&self, identifier: &str) -> Result<Vec<u8>, ContactsError> {
        validate_identifier(identifier)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            let identifier = NSString::from_str(identifier);
            // SAFETY: This is the public descriptor Contacts requires for vCard serialization.
            let descriptor = unsafe { CNContactVCardSerialization::descriptorForRequiredKeys() };
            let keys = NSArray::from_slice(&[descriptor.as_ref()]);
            // SAFETY: The validated identifier and serializer-required descriptor are valid;
            // the caller-owned store remains retained for this one-record lookup.
            let contact = unsafe {
                self.store
                    .unifiedContactWithIdentifier_keysToFetch_error(&identifier, &keys)
            }
            .map_err(|error| {
                let current_status = authorization_from_native(native_authorization_status());
                let contacts_domain = unsafe { CNErrorDomain.as_ref() }
                    .is_some_and(|domain| error.domain().isEqualToString(domain));
                fetch_error_from_native(current_status, contacts_domain, error.code())
            })?;

            let contacts = NSArray::from_slice(&[contact.as_ref()]);
            // SAFETY: The sole contact was fetched with the descriptor required by this serializer.
            let data = unsafe { CNContactVCardSerialization::dataWithContacts_error(&contacts) }
                .map_err(|error| {
                    let current_status = authorization_from_native(native_authorization_status());
                    export_error_from_native(current_status, error.code())
                })?;

            Ok(data.to_vec())
        })
    }
}
