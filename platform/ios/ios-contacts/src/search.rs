use crate::conversion::{
    append_search_result, authorization_from_native, fetch_authorization_error,
    search_error_from_native, validate_search_query,
};
use crate::platform::{IosContactsBackend, native_authorization_status};
use crate::snapshot::{contact_from_native, contact_key_descriptors};
use block2::StackBlock;
use core::ptr::NonNull;
use framework_contacts::{Contact, ContactsError, ContactsSearchBackend};
use framework_core::{Error, ErrorKind};
use objc2::AnyThread;
use objc2::rc::autoreleasepool;
use objc2::runtime::Bool;
use objc2_contacts::{CNContact, CNContactFetchRequest};
use objc2_foundation::{NSArray, NSError, NSString};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

impl ContactsSearchBackend for IosContactsBackend {
    fn search_contacts_by_name(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<Contact>, ContactsError> {
        validate_search_query(query)?;
        let status = authorization_from_native(native_authorization_status());
        if let Some(error) = fetch_authorization_error(status) {
            return Err(error);
        }
        if max_results == 0 {
            return Ok(Vec::new());
        }

        autoreleasepool(|_| {
            let query = NSString::from_str(query);
            // SAFETY: The public name predicate is valid for a CNContactFetchRequest.
            let predicate = unsafe { CNContact::predicateForContactsMatchingName(&query) };
            let descriptors = contact_key_descriptors();
            let keys = NSArray::from_slice(&descriptors);
            // SAFETY: The key descriptors are public Contacts keys and request only properties
            // copied by `contact_from_native` before this autorelease pool ends.
            let request = unsafe {
                CNContactFetchRequest::initWithKeysToFetch(CNContactFetchRequest::alloc(), &keys)
            };
            // SAFETY: `predicate` is the public Contacts name predicate supported by this request.
            unsafe { request.setPredicate(Some(&predicate)) };

            let results = Mutex::new(Vec::new());
            let callback_panicked = AtomicBool::new(false);
            let block = StackBlock::new(|contact: NonNull<CNContact>, stop: NonNull<Bool>| {
                if callback_panicked.load(Ordering::Acquire) {
                    // SAFETY: Contacts provides a valid mutable stop pointer for this callback.
                    unsafe { stop.as_ptr().write(Bool::YES) };
                    return;
                }
                let should_stop = catch_unwind(AssertUnwindSafe(|| {
                    let mut results = results
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    append_search_result(&mut results, max_results, || {
                        // SAFETY: Contacts lends this contact only for the callback's duration.
                        contact_from_native(unsafe { contact.as_ref() })
                    })
                }));
                match should_stop {
                    Ok(true) => {
                        // SAFETY: Contacts provides a valid mutable stop pointer for this callback.
                        unsafe { stop.as_ptr().write(Bool::YES) };
                    }
                    Ok(false) => {}
                    Err(_) => {
                        callback_panicked.store(true, Ordering::Release);
                        // SAFETY: Contacts provides a valid mutable stop pointer for this callback.
                        unsafe { stop.as_ptr().write(Bool::YES) };
                    }
                }
            });
            let mut error: Option<objc2::rc::Retained<NSError>> = None;
            // SAFETY: The request and its predicate/keys remain alive for this synchronous call.
            // The block borrows only stack state and the API marks it NS_NOESCAPE; every native
            // contact is copied before return and the callback contains Rust panics.
            let succeeded = unsafe {
                self.store
                    .enumerateContactsWithFetchRequest_error_usingBlock(
                        &request,
                        Some(&mut error),
                        &block,
                    )
            };
            if callback_panicked.load(Ordering::Acquire) {
                return Err(ContactsError::Backend(Error::new(ErrorKind::Internal)));
            }
            if !succeeded {
                let current_status = authorization_from_native(native_authorization_status());
                let native_error_code = error.as_ref().map(|error| error.code());
                return Err(search_error_from_native(current_status, native_error_code));
            }
            Ok(results
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner))
        })
    }
}
