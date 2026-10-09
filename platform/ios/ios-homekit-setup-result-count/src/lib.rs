#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A host-supplied HomeKit setup-result accessory identifier count for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessorySetupResult;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit setup-result accessory identifier count
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitSetupResultCountError {
    /// The iOS version does not provide `HMAccessorySetupResult.accessoryUniqueIdentifiers`
    NativeApiUnavailable,
}

/// Count accessory identifiers recorded in a host-supplied successful setup result
///
/// This reads only the length of `HMAccessorySetupResult.accessoryUniqueIdentifiers` on a live
/// result object already supplied by the host. It does not inspect or return home/accessory IDs,
/// enumerate HomeKit services or profiles, inspect the current home inventory, or establish
/// reachability, ongoing availability, or operational readiness. The crate does not create
/// `HMAccessorySetupManager`, run a setup flow, present UI, request permission, or query a manager
///
/// The API is available on iOS 15.4 and later. Older iOS versions return
/// [`HomeKitSetupResultCountError::NativeApiUnavailable`]. The host owns the setup flow and its
/// result object, HomeKit capability, `com.apple.developer.homekit` entitlement,
/// `NSHomeKitUsageDescription`, prior authorization, and the object's source and lifetime
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn setup_accessory_identifier_count(
    result: &HMAccessorySetupResult,
) -> Result<usize, HomeKitSetupResultCountError> {
    if !objc2::available!(ios = 15.4, ..) {
        return Err(HomeKitSetupResultCountError::NativeApiUnavailable);
    }

    // SAFETY: the host supplies a live HMAccessorySetupResult. The SDK declares
    // accessoryUniqueIdentifiers readonly/copy and omits nonatomic, so the Objective-C property
    // access is atomic. The result is NS_SWIFT_SENDABLE, and objc2-home-kit 0.3.2 gives this typed
    // getter no non-atomic/thread-safety warning. The retained array stays alive while its safe
    // length is read; no UUID element is accessed or copied.
    let identifiers = unsafe { result.accessoryUniqueIdentifiers() };
    Ok(identifiers.len())
}
