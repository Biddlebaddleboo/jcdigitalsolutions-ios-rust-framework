#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A host-supplied HomeKit accessory profile-count snapshot for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessory;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit accessory profile count
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitAccessoryProfileCountError {
    /// The iOS version does not provide `HMAccessory.profiles`
    NativeApiUnavailable,
}

/// Read the number of profiles on a host-supplied HomeKit accessory
///
/// This reads only the length of `HMAccessory.profiles` from a live accessory that the host
/// already obtained through its HomeKit lifecycle. It does not inspect profile objects, services,
/// characteristics, or controls, and a count does not establish profile-feature availability,
/// reachability, operation success, or HomeKit readiness. This function never creates
/// `HMHomeManager`, requests permission, enumerates accessories, or invokes a HomeKit operation
///
/// The API is available on iOS 11.0 and later. Older iOS versions return
/// [`HomeKitAccessoryProfileCountError::NativeApiUnavailable`]. The host owns the HomeKit
/// capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior
/// HomeKit authorization, and the lifetime/source of the supplied accessory
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn profile_count(accessory: &HMAccessory) -> Result<usize, HomeKitAccessoryProfileCountError> {
    if !objc2::available!(ios = 11, ..) {
        return Err(HomeKitAccessoryProfileCountError::NativeApiUnavailable);
    }

    // SAFETY: the host supplies a live HMAccessory. The public `profiles` property is readonly,
    // copy, and atomic by default because the SDK declaration omits `nonatomic`. objc2-home-kit
    // 0.3.2 gives this getter no non-atomic/thread-safety warning. The retained NSArray keeps its
    // lifetime while only its length is read; no profile element is accessed.
    let profiles = unsafe { accessory.profiles() };
    Ok(profiles.len())
}
