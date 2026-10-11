use crate::conversion::{
    authorization_from_native, fetch_authorization_error, fetch_error_from_native,
    update_error_from_native, validate_contact_update, validate_identifier,
};
use crate::create::{native_email_values, native_phone_values};
use crate::platform::{IosContactsBackend, native_authorization_status};
use crate::snapshot::contact_key_descriptors;
use framework_contacts::{ContactUpdate, ContactsError, ContactsUpdateBackend};
use objc2::rc::autoreleasepool;
use objc2_contacts::{CNErrorDomain, CNSaveRequest};
use objc2_foundation::{NSArray, NSMutableCopying, NSString};

impl ContactsUpdateBackend for IosContactsBackend {
    fn update_contact(
        &self,
        identifier: &str,
        update: &ContactUpdate,
    ) -> Result<(), ContactsError> {
        validate_identifier(identifier)?;
        validate_contact_update(update)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            let identifier = NSString::from_str(identifier);
            let descriptors = contact_key_descriptors();
            let keys = NSArray::from_slice(&descriptors);
            // SAFETY: The identifier and all mutable-copy properties are requested; the retained
            // store is caller-owned, and the fetched object is used only inside this pool.
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

            // CNContact publicly conforms to NSMutableCopying and objc2-contacts maps the copy
            // result to CNMutableContact. Retained owns the copy through the save operation.
            let mutable_contact = contact.mutableCopy();
            if let Some(given_name) = &update.given_name {
                let value = NSString::from_str(given_name);
                // SAFETY: This property was requested above and the mutable copy is owned here.
                unsafe { mutable_contact.setGivenName(&value) };
            }
            if let Some(family_name) = &update.family_name {
                let value = NSString::from_str(family_name);
                // SAFETY: This property was requested above and the mutable copy is owned here.
                unsafe { mutable_contact.setFamilyName(&value) };
            }
            if let Some(organization_name) = &update.organization_name {
                let value = NSString::from_str(organization_name);
                // SAFETY: This property was requested above and the mutable copy is owned here.
                unsafe { mutable_contact.setOrganizationName(&value) };
            }
            if let Some(phone_numbers) = &update.phone_numbers {
                let values = native_phone_values(phone_numbers)?;
                let values = NSArray::from_retained_slice(&values);
                // SAFETY: This property was requested above; an empty array deliberately clears it.
                unsafe { mutable_contact.setPhoneNumbers(&values) };
            }
            if let Some(email_addresses) = &update.email_addresses {
                let values = native_email_values(email_addresses);
                let values = NSArray::from_retained_slice(&values);
                // SAFETY: This property was requested above; an empty array deliberately clears it.
                unsafe { mutable_contact.setEmailAddresses(&values) };
            }

            // SAFETY: This request contains exactly one public update. It and the mutable copy
            // remain retained for the synchronous call; neither is accessed while it executes.
            let request = unsafe { CNSaveRequest::new() };
            unsafe { request.updateContact(&mutable_contact) };
            unsafe { self.store.executeSaveRequest_error(&request) }.map_err(|error| {
                let current_status = authorization_from_native(native_authorization_status());
                let contacts_domain = unsafe { CNErrorDomain.as_ref() }
                    .is_some_and(|domain| error.domain().isEqualToString(domain));
                update_error_from_native(current_status, contacts_domain, error.code())
            })
        })
    }
}
