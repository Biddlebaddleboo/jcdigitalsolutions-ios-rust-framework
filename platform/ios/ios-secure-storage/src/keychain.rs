use crate::policy::{KeychainClass, map_policy};
use crate::status::{NativeStatus, storage_error};
use core::ffi::c_void;
use core::ptr::{self, NonNull};
use framework_core::{Availability, Error, ErrorKind};
use framework_secure_storage::{
    AccessPolicy, ItemId, SecureStorageBackend, SecureStorageError, ServiceId, StoreOutcome,
};
use objc2_core_foundation::{
    CFData, CFDictionary, CFIndex, CFRetained, CFString, CFType, kCFBooleanTrue,
    kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
};
use objc2_security::{
    SecItemAdd, SecItemCopyMatching, SecItemDelete, SecItemUpdate, errSecDuplicateItem,
    errSecItemNotFound, kSecAttrAccessible, kSecAttrAccessibleAfterFirstUnlock,
    kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly, kSecAttrAccessibleWhenUnlocked,
    kSecAttrAccessibleWhenUnlockedThisDeviceOnly, kSecAttrAccount, kSecAttrService, kSecClass,
    kSecClassGenericPassword, kSecReturnData, kSecValueData,
};

/// Stateless caller-owned access to generic-password items in the app's default Keychain group.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosSecureStorage;

impl IosSecureStorage {
    /// Creates a backend value without initializing or registering a process-global service.
    pub const fn new() -> Self {
        Self
    }
}

impl SecureStorageBackend for IosSecureStorage {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn read(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<Option<alloc::vec::Vec<u8>>, SecureStorageError> {
        let service = CFString::from_str(service.as_str());
        let item = CFString::from_str(item.as_str());
        // SAFETY: The generated Security statics are non-null, immutable, process-lifetime CFStrings.
        let (class_key, generic_password, service_key, account_key, return_data_key) = unsafe {
            (
                kSecClass,
                kSecClassGenericPassword,
                kSecAttrService,
                kSecAttrAccount,
                kSecReturnData,
            )
        };
        // SAFETY: CoreFoundation defines this exported Boolean as a process-lifetime singleton.
        let return_data = unsafe { kCFBooleanTrue.expect("CoreFoundation boolean constant") };
        let query = dictionary([
            (class_key, generic_password),
            (service_key, &service),
            (account_key, &item),
            (return_data_key, return_data),
        ])?;
        let mut result: *const CFType = ptr::null();
        let status = unsafe { SecItemCopyMatching(&query, &mut result) };
        match NativeStatus::from_os_status(status, errSecItemNotFound) {
            NativeStatus::Success => {
                let result = NonNull::new(result as *mut CFType).ok_or_else(internal_error)?;
                let result = unsafe { CFRetained::from_raw(result) };
                let data = result.downcast_ref::<CFData>().ok_or_else(internal_error)?;
                Ok(Some(data.to_vec()))
            }
            NativeStatus::ItemNotFound => Ok(None),
            NativeStatus::Failure(code) => Err(storage_error(code)),
        }
    }

    fn store(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
        secret: &[u8],
        required_policy: AccessPolicy,
    ) -> Result<StoreOutcome, SecureStorageError> {
        let mapped = map_policy(required_policy).ok_or(SecureStorageError::UnsupportedPolicy)?;
        if !mapped.effective.satisfies(required_policy) {
            return Err(SecureStorageError::UnsupportedPolicy);
        }
        let service = CFString::from_str(service.as_str());
        let item = CFString::from_str(item.as_str());
        let secret = CFData::from_bytes(secret);
        let accessibility = accessibility_value(mapped.class);
        let query = identity_query(&service, &item)?;
        // SAFETY: These generated Security statics are non-null, immutable, process-lifetime CFStrings.
        let (value_data_key, accessible_key) = unsafe { (kSecValueData, kSecAttrAccessible) };
        let attributes = dictionary([(value_data_key, &secret), (accessible_key, accessibility)])?;
        let status = unsafe { SecItemUpdate(&query, &attributes) };
        match NativeStatus::from_os_status(status, errSecItemNotFound) {
            NativeStatus::Success => Ok(StoreOutcome::new(mapped.effective)),
            NativeStatus::ItemNotFound => {
                // SAFETY: These generated Security statics are non-null, immutable, process-lifetime CFStrings.
                let (
                    class_key,
                    generic_password,
                    service_key,
                    account_key,
                    accessible_key,
                    value_data_key,
                ) = unsafe {
                    (
                        kSecClass,
                        kSecClassGenericPassword,
                        kSecAttrService,
                        kSecAttrAccount,
                        kSecAttrAccessible,
                        kSecValueData,
                    )
                };
                let attributes = dictionary([
                    (class_key, generic_password),
                    (service_key, &service),
                    (account_key, &item),
                    (accessible_key, accessibility),
                    (value_data_key, &secret),
                ])?;
                let status = unsafe { SecItemAdd(&attributes, ptr::null_mut()) };
                if status == 0 {
                    Ok(StoreOutcome::new(mapped.effective))
                } else if status == errSecDuplicateItem {
                    let retry_attributes = attributes_for_update(&secret, accessibility)?;
                    let status = unsafe { SecItemUpdate(&query, &retry_attributes) };
                    if status == 0 {
                        Ok(StoreOutcome::new(mapped.effective))
                    } else {
                        Err(storage_error(status))
                    }
                } else {
                    Err(storage_error(status))
                }
            }
            NativeStatus::Failure(code) => Err(storage_error(code)),
        }
    }

    fn remove(
        &mut self,
        service: ServiceId<'_>,
        item: ItemId<'_>,
    ) -> Result<bool, SecureStorageError> {
        let service = CFString::from_str(service.as_str());
        let item = CFString::from_str(item.as_str());
        let query = identity_query(&service, &item)?;
        let status = unsafe { SecItemDelete(&query) };
        match NativeStatus::from_os_status(status, errSecItemNotFound) {
            NativeStatus::Success => Ok(true),
            NativeStatus::ItemNotFound => Ok(false),
            NativeStatus::Failure(code) => Err(storage_error(code)),
        }
    }
}

fn identity_query(
    service: &CFString,
    item: &CFString,
) -> Result<CFRetained<CFDictionary>, SecureStorageError> {
    // SAFETY: The generated Security statics are non-null, immutable, process-lifetime CFStrings.
    let (class_key, generic_password, service_key, account_key) = unsafe {
        (
            kSecClass,
            kSecClassGenericPassword,
            kSecAttrService,
            kSecAttrAccount,
        )
    };
    dictionary([
        (class_key, generic_password),
        (service_key, service),
        (account_key, item),
    ])
}

fn attributes_for_update(
    secret: &CFData,
    accessibility: &'static CFString,
) -> Result<CFRetained<CFDictionary>, SecureStorageError> {
    // SAFETY: The generated Security statics are non-null, immutable, process-lifetime CFStrings.
    let (value_data_key, accessible_key) = unsafe { (kSecValueData, kSecAttrAccessible) };
    dictionary([(value_data_key, secret), (accessible_key, accessibility)])
}

fn dictionary<const N: usize>(
    pairs: [(&CFType, &CFType); N],
) -> Result<CFRetained<CFDictionary>, SecureStorageError> {
    let count = CFIndex::try_from(N)
        .map_err(|_| SecureStorageError::Backend(Error::new(ErrorKind::ResourceExhausted)))?;
    let mut keys = pairs.map(|(key, _)| key as *const CFType as *const c_void);
    let mut values = pairs.map(|(_, value)| value as *const CFType as *const c_void);
    // SAFETY: Arrays point to live CF objects, their lengths match `count`, and the type callbacks
    // retain each entry before these local arrays and values are dropped.
    let dictionary = unsafe {
        CFDictionary::new(
            None,
            keys.as_mut_ptr(),
            values.as_mut_ptr(),
            count,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        )
    };
    dictionary.ok_or_else(|| SecureStorageError::Backend(Error::new(ErrorKind::ResourceExhausted)))
}

fn accessibility_value(class: KeychainClass) -> &'static CFString {
    // SAFETY: Each selected generated Security static is a non-null, immutable process-lifetime CFString.
    unsafe {
        match class {
            KeychainClass::AfterFirstUnlock => kSecAttrAccessibleAfterFirstUnlock,
            KeychainClass::WhenUnlocked => kSecAttrAccessibleWhenUnlocked,
            KeychainClass::AfterFirstUnlockThisDeviceOnly => {
                kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            }
            KeychainClass::WhenUnlockedThisDeviceOnly => {
                kSecAttrAccessibleWhenUnlockedThisDeviceOnly
            }
        }
    }
}

fn internal_error() -> SecureStorageError {
    SecureStorageError::Backend(Error::new(ErrorKind::Internal))
}
