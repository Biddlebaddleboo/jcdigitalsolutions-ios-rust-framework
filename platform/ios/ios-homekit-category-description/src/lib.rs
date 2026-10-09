#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A host-supplied HomeKit accessory category-description snapshot for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessoryCategory;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit accessory category description
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitCategoryDescriptionError {
    /// The iOS version does not provide `HMAccessoryCategory.localizedDescription`
    NativeApiUnavailable,
}

/// Copy the localized display description from a host-supplied HomeKit category object
///
/// This reads only `HMAccessoryCategory.localizedDescription` from a live category object already
/// supplied by the host and copies the retained native `NSString` into Rust-owned storage. The
/// result is localized presentation text, not a stable identifier, support value, authorization
/// status, reachability, profile availability, or accessory readiness. This function does not
/// create a category object, call `HMAccessory.category`, access profile or service values, create
/// `HMHomeManager`, request permission, or enumerate accessories
///
/// The property is available on iOS 9.0 and later. Older iOS versions return
/// [`HomeKitCategoryDescriptionError::NativeApiUnavailable`]. The host must supply a valid live
/// `HMAccessoryCategory` object obtained through its own HomeKit lifecycle and owns the HomeKit
/// capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior
/// authorization, and the object's source and lifetime. This crate does not obtain the category
/// through the nonatomic `HMAccessory.category` property
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn localized_description(
    category: &HMAccessoryCategory,
) -> Result<String, HomeKitCategoryDescriptionError> {
    if !objc2::available!(ios = 9, ..) {
        return Err(HomeKitCategoryDescriptionError::NativeApiUnavailable);
    }

    // SAFETY: the host supplies a live HMAccessoryCategory. The SDK declares localizedDescription
    // readonly/copy and omits nonatomic, so its Objective-C access is atomic. The class is
    // NS_SWIFT_SENDABLE and objc2-home-kit 0.3.2 gives this getter no non-atomic/thread warning.
    // The getter returns a retained NSString, which to_string copies into Rust-owned storage.
    Ok(unsafe { category.localizedDescription() }.to_string())
}
