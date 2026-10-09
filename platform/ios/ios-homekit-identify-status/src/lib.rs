#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A host-supplied HomeKit accessory identify-support snapshot for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessory;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit accessory support value
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitIdentifyStatusError {
    /// The iOS version does not provide `HMAccessory.supportsIdentify`
    NativeApiUnavailable,
}

/// Read whether a host-supplied accessory supports the HomeKit identify action
///
/// This reads only `HMAccessory.supportsIdentify` from a live accessory that the host already
/// obtained through its HomeKit lifecycle. `Ok(false)` means Apple documents that calls to
/// `identifyWithCompletionHandler:` return an error; `Ok(true)` does not guarantee that a later
/// identify call succeeds. This function never invokes identify, creates `HMHomeManager`, requests
/// permission, enumerates accessories, or reports HomeKit readiness, authorization, or reachability
///
/// The API is available on iOS 11.3 and later. Older iOS versions return
/// [`HomeKitIdentifyStatusError::NativeApiUnavailable`]. The host owns the HomeKit capability,
/// `NSHomeKitUsageDescription`, prior HomeKit authorization, and the lifetime/source of the supplied
/// accessory
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn supports_identify(accessory: &HMAccessory) -> Result<bool, HomeKitIdentifyStatusError> {
    if !objc2::available!(ios = 11.3, ..) {
        return Err(HomeKitIdentifyStatusError::NativeApiUnavailable);
    }

    // SAFETY: the input is a borrowed live HMAccessory; Apple marks the class NS_SWIFT_SENDABLE,
    // and this scalar readonly Objective-C property omits nonatomic, so its access is atomic
    Ok(unsafe { accessory.supportsIdentify() })
}
