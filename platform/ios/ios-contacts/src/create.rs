use crate::conversion::{
    authorization_from_native, contact_value_parts, create_authorization_error,
    create_error_from_native, validate_contact_draft,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use framework_contacts::{ContactDraft, ContactLabeledValue, ContactsCreateBackend, ContactsError};
use framework_core::{Error, ErrorKind};
use objc2::rc::{Retained, autoreleasepool};
use objc2_contacts::{CNLabeledValue, CNMutableContact, CNPhoneNumber, CNSaveRequest};
use objc2_foundation::{NSArray, NSString};

impl ContactsCreateBackend for IosContactsBackend {
    fn create_contact(&self, draft: &ContactDraft) -> Result<(), ContactsError> {
        validate_contact_draft(draft)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = create_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            // SAFETY: CNMutableContact is a public, caller-owned value object. It contains only
            // Rust strings and labeled values copied below, and remains local to this save.
            let contact = unsafe { CNMutableContact::new() };
            let given_name = NSString::from_str(&draft.given_name);
            let family_name = NSString::from_str(&draft.family_name);
            let organization_name = NSString::from_str(&draft.organization_name);
            // SAFETY: These setters accept non-null NSString values and contact is mutable.
            unsafe {
                contact.setGivenName(&given_name);
                contact.setFamilyName(&family_name);
                contact.setOrganizationName(&organization_name);
            }

            let phone_numbers = native_phone_values(&draft.phone_numbers)?;
            if !phone_numbers.is_empty() {
                let values = NSArray::from_retained_slice(&phone_numbers);
                // SAFETY: Every array element is the documented labeled CNPhoneNumber value.
                unsafe { contact.setPhoneNumbers(&values) };
            }
            let email_addresses = native_email_values(&draft.email_addresses);
            if !email_addresses.is_empty() {
                let values = NSArray::from_retained_slice(&email_addresses);
                // SAFETY: Every array element is a labeled NSString email value.
                unsafe { contact.setEmailAddresses(&values) };
            }

            // SAFETY: Each operation is one public add-only save request. A nil container ID is
            // documented to select the default container; no existing record is fetched or edited.
            let request = unsafe { CNSaveRequest::new() };
            unsafe { request.addContact_toContainerWithIdentifier(&contact, None) };
            // SAFETY: The backend owns the store and this scope owns the request; both remain
            // alive for the synchronous save call, and the request contains valid Contacts values.
            unsafe { self.store.executeSaveRequest_error(&request) }.map_err(|error| {
                let current_status = authorization_from_native(native_authorization_status());
                create_error_from_native(current_status, error.code())
            })
        })
    }
}

pub(super) fn native_phone_values(
    values: &[ContactLabeledValue],
) -> Result<Vec<Retained<CNLabeledValue<CNPhoneNumber>>>, ContactsError> {
    values
        .iter()
        .map(|value| {
            let (label, number) = contact_value_parts(value);
            let label = label.map(NSString::from_str);
            let number = NSString::from_str(number);
            // SAFETY: The phone string is non-empty after draft validation.
            let phone = unsafe { CNPhoneNumber::phoneNumberWithStringValue(&number) }
                .ok_or_else(internal_error)?;
            // SAFETY: The optional label and non-null CNPhoneNumber match the public initializer.
            Ok(unsafe {
                CNLabeledValue::<CNPhoneNumber>::labeledValueWithLabel_value(
                    label.as_deref(),
                    &phone,
                )
            })
        })
        .collect()
}

pub(super) fn native_email_values(
    values: &[ContactLabeledValue],
) -> Vec<Retained<CNLabeledValue<NSString>>> {
    values
        .iter()
        .map(|value| {
            let (label, email) = contact_value_parts(value);
            let label = label.map(NSString::from_str);
            let email = NSString::from_str(email);
            // SAFETY: The optional label and non-null NSString match the public initializer.
            unsafe {
                CNLabeledValue::<NSString>::labeledValueWithLabel_value(label.as_deref(), &email)
            }
        })
        .collect()
}

fn internal_error() -> ContactsError {
    ContactsError::Backend(Error::new(ErrorKind::Internal))
}
