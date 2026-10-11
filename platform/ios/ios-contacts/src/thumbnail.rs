use crate::conversion::{
    authorization_from_native, fetch_authorization_error, fetch_error_from_native,
    validate_identifier,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use framework_contacts::{ContactsError, ContactsImageBackend};
use objc2::rc::autoreleasepool;
use objc2::runtime::ProtocolObject;
use objc2_contacts::{CNContactThumbnailImageDataKey, CNErrorDomain, CNKeyDescriptor};
use objc2_foundation::{NSArray, NSString};

fn thumbnail_key_descriptors() -> [&'static ProtocolObject<dyn CNKeyDescriptor>; 1] {
    // SAFETY: This public Contacts key descriptor is immutable and valid for process lifetime.
    unsafe { [ProtocolObject::from_ref(CNContactThumbnailImageDataKey)] }
}

impl ContactsImageBackend for IosContactsBackend {
    fn fetch_contact_thumbnail(&self, identifier: &str) -> Result<Option<Vec<u8>>, ContactsError> {
        validate_identifier(identifier)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }

        autoreleasepool(|_| {
            let identifier = NSString::from_str(identifier);
            let descriptors = thumbnail_key_descriptors();
            let keys = NSArray::from_slice(&descriptors);
            // SAFETY: The identifier and sole thumbnail key are valid Contacts values, the store
            // is caller-owned, and NSData is copied before this autorelease pool ends.
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

            // SAFETY: CNContactThumbnailImageDataKey was the sole requested descriptor.
            let data = unsafe { contact.thumbnailImageData() };
            Ok(data.map(|data| data.to_vec()))
        })
    }
}
