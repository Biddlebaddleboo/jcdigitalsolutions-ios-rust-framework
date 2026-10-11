use crate::conversion::{
    authorization_from_native, fetch_authorization_error, fetch_error_from_native,
    validate_identifier,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use crate::snapshot::{contact_from_native, contact_key_descriptors};
use framework_contacts::{Contact, ContactsError, ContactsFetchBackend};
use objc2::rc::autoreleasepool;
use objc2_contacts::CNErrorDomain;
use objc2_foundation::{NSArray, NSString};

impl ContactsFetchBackend for IosContactsBackend {
    fn fetch_contact(&self, identifier: &str) -> Result<Contact, ContactsError> {
        validate_identifier(identifier)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            let identifier = NSString::from_str(identifier);
            let descriptors = contact_key_descriptors();
            let keys = NSArray::from_slice(&descriptors);
            // SAFETY: The identifier and key descriptors are valid Contacts values, the retained
            // store is caller-owned, and every property read below is requested. Native values
            // are copied before this autorelease pool ends.
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

            Ok(contact_from_native(&contact))
        })
    }
}
