use framework_contacts::{Contact, ContactLabeledValue};
use objc2::runtime::ProtocolObject;
use objc2_contacts::{
    CNContact, CNContactEmailAddressesKey, CNContactFamilyNameKey, CNContactGivenNameKey,
    CNContactIdentifierKey, CNContactOrganizationNameKey, CNContactPhoneNumbersKey,
    CNKeyDescriptor,
};

/// Returns the exact stable public keys copied by the shared contact snapshot conversion.
pub(super) fn contact_key_descriptors() -> [&'static ProtocolObject<dyn CNKeyDescriptor>; 6] {
    // SAFETY: These immutable extern constants are public NSString key descriptors supplied by
    // Contacts and remain valid for process lifetime.
    unsafe {
        [
            ProtocolObject::from_ref(CNContactIdentifierKey),
            ProtocolObject::from_ref(CNContactGivenNameKey),
            ProtocolObject::from_ref(CNContactFamilyNameKey),
            ProtocolObject::from_ref(CNContactOrganizationNameKey),
            ProtocolObject::from_ref(CNContactPhoneNumbersKey),
            ProtocolObject::from_ref(CNContactEmailAddressesKey),
        ]
    }
}

/// Copies the common requested contact keys before their autorelease pool ends.
pub(super) fn contact_from_native(contact: &CNContact) -> Contact {
    // SAFETY: Callers request every property below in the fetch's key descriptors.
    let identifier = unsafe { contact.identifier() }.to_string();
    let given_name = unsafe { contact.givenName() }.to_string();
    let family_name = unsafe { contact.familyName() }.to_string();
    let organization_name = unsafe { contact.organizationName() }.to_string();
    let native_phones = unsafe { contact.phoneNumbers() };
    let phone_numbers = (0..native_phones.count())
        .map(|index| {
            let native_value = native_phones.objectAtIndex(index);
            let label = unsafe { native_value.label() }.map(|value| value.to_string());
            let phone = unsafe { native_value.value() };
            let value = unsafe { phone.stringValue() }.to_string();
            ContactLabeledValue { label, value }
        })
        .collect();
    let native_emails = unsafe { contact.emailAddresses() };
    let email_addresses = (0..native_emails.count())
        .map(|index| {
            let native_value = native_emails.objectAtIndex(index);
            let label = unsafe { native_value.label() }.map(|value| value.to_string());
            let value = unsafe { native_value.value() }.to_string();
            ContactLabeledValue { label, value }
        })
        .collect();

    Contact {
        identifier,
        given_name,
        family_name,
        organization_name,
        phone_numbers,
        email_addresses,
    }
}
