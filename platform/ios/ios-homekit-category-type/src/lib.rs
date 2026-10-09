#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A host-supplied HomeKit accessory category-type snapshot for iOS"]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use objc2_home_kit::HMAccessoryCategory;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
/// An error while reading a HomeKit accessory category type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HomeKitCategoryTypeError {
    /// The iOS version does not provide `HMAccessoryCategory.categoryType`
    NativeApiUnavailable,
}

/// Copy the category identifier from a host-supplied HomeKit category object
///
/// This reads only `HMAccessoryCategory.categoryType` from a live object already supplied by the
/// host. The returned Rust `String` preserves the native identifier without enum conversion or
/// normalization, including values this crate does not know. This does not establish HomeKit
/// support, authorization, reachability, profile availability, or accessory readiness. The crate
/// does not call `HMAccessory.category`, access profiles or services, create `HMHomeManager`,
/// request permission, enumerate accessories, or create a category object
///
/// The property is available on iOS 9.0 and later. Older iOS versions return
/// [`HomeKitCategoryTypeError::NativeApiUnavailable`]. The host must supply a valid live
/// `HMAccessoryCategory` object obtained through its own HomeKit lifecycle and owns the HomeKit
/// capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior
/// authorization, and the object's source and lifetime. This crate does not obtain the category
/// through the nonatomic `HMAccessory.category` property
#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub fn category_type(category: &HMAccessoryCategory) -> Result<String, HomeKitCategoryTypeError> {
    if !objc2::available!(ios = 9, ..) {
        return Err(HomeKitCategoryTypeError::NativeApiUnavailable);
    }

    // SAFETY: the host supplies a live HMAccessoryCategory. The SDK declares categoryType as
    // readonly/copy and omits nonatomic, so its Objective-C access is atomic. The class is
    // NS_SWIFT_SENDABLE and objc2-home-kit 0.3.2 gives this getter no non-atomic/thread warning.
    // The getter returns a retained NSString, which to_string copies into Rust-owned storage.
    Ok(unsafe { category.categoryType() }.to_string())
}
