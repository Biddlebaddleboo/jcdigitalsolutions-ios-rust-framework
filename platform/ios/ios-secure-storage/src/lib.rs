#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![doc = "Caller-owned iOS Keychain backend for the portable opaque-byte secure-storage contract."]

extern crate alloc;

#[cfg(any(target_os = "ios", test))]
mod policy;
#[cfg(any(target_os = "ios", test))]
mod status;

#[cfg(target_os = "ios")]
mod keychain;

#[cfg(target_os = "ios")]
pub use keychain::IosSecureStorage;

#[cfg(test)]
mod tests {
    use crate::policy::{KeychainClass, map_policy};
    use crate::status::{NativeStatus, native_error};
    use framework_core::{ErrorKind, PlatformErrorCode};
    use framework_secure_storage::{AccessPolicy, SecureStorageError};

    #[test]
    fn maps_every_portable_policy_to_a_public_accessibility_class() {
        let cases = [
            (
                AccessPolicy::new(false, false),
                KeychainClass::AfterFirstUnlock,
            ),
            (AccessPolicy::new(true, false), KeychainClass::WhenUnlocked),
            (
                AccessPolicy::new(false, true),
                KeychainClass::AfterFirstUnlockThisDeviceOnly,
            ),
            (
                AccessPolicy::new(true, true),
                KeychainClass::WhenUnlockedThisDeviceOnly,
            ),
        ];

        for (required, class) in cases {
            let mapped = map_policy(required).expect("all boolean policies have a public class");
            assert_eq!(mapped.class, class);
            assert_eq!(mapped.effective, required);
            assert!(mapped.effective.satisfies(required));
        }
    }

    #[test]
    fn only_item_not_found_is_a_miss_and_other_statuses_remain_native_errors() {
        assert_eq!(
            NativeStatus::from_os_status(0, -25300),
            NativeStatus::Success
        );
        assert_eq!(
            NativeStatus::from_os_status(-25300, -25300),
            NativeStatus::ItemNotFound
        );
        assert_eq!(
            NativeStatus::from_os_status(-50, -25300),
            NativeStatus::Failure(-50)
        );
        let error = native_error(-50);
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(
            error.platform_code(),
            Some(PlatformErrorCode::new(-50).unwrap())
        );
        assert_eq!(
            SecureStorageError::Backend(error).platform_code(),
            Some(PlatformErrorCode::new(-50).unwrap())
        );
    }
}
