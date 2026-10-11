use crate::conversion::{
    authorization_from_native, delete_error_from_native, fetch_authorization_error,
    fetch_error_from_native, validate_identifier,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use crate::snapshot::contact_identifier_key_descriptors;
use framework_contacts::{ContactsDeleteBackend, ContactsError};
use objc2::rc::autoreleasepool;
use objc2_contacts::{CNErrorDomain, CNSaveRequest};
use objc2_foundation::{NSArray, NSMutableCopying, NSString};

impl ContactsDeleteBackend for IosContactsBackend {
    fn delete_contact(&self, identifier: &str) -> Result<(), ContactsError> {
        validate_identifier(identifier)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            let identifier = NSString::from_str(identifier);
            let descriptors = contact_identifier_key_descriptors();
            let keys = NSArray::from_slice(&descriptors);
            // SAFETY: The caller identifier and sole key descriptor are valid Contacts values;
            // the retained store is caller-owned and the fetched contact is used only in this pool.
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

            // SAFETY: CNContact publicly conforms to NSMutableCopying; the retained mutable copy
            // remains owned through the synchronous save request.
            let mutable_contact = contact.mutableCopy();
            // SAFETY: This fresh request contains exactly one public contact deletion; both values
            // remain retained and are not accessed while the synchronous save executes.
            let request = unsafe { CNSaveRequest::new() };
            unsafe { request.deleteContact(&mutable_contact) };
            unsafe { self.store.executeSaveRequest_error(&request) }.map_err(|error| {
                let current_status = authorization_from_native(native_authorization_status());
                let contacts_domain = unsafe { CNErrorDomain.as_ref() }
                    .is_some_and(|domain| error.domain().isEqualToString(domain));
                delete_error_from_native(current_status, contacts_domain, error.code())
            })
        })
    }
}
