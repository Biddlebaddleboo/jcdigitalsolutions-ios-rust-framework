# iOS HomeKit accessory category-description snapshot

`ios-homekit-category-description` exposes `ios_homekit_category_description::localized_description(&HMAccessoryCategory) -> Result<String, HomeKitCategoryDescriptionError>` on iOS

The caller supplies a live `HMAccessoryCategory` object already obtained through its HomeKit host. The function reads only `localizedDescription` and copies the retained `NSString` into a Rust-owned `String`. The result is localized presentation text; it is not a stable identifier, authorization or support value, profile-feature result, or readiness claim

`localizedDescription` is available from iOS 9.0; older iOS versions return `NativeApiUnavailable`. The package uses `objc2-home-kit` 0.3.2 with only the `HMAccessoryCategory` feature. The iOS 26.5 SDK declares the property readonly/copy without `nonatomic`, so it uses Objective-C's atomic default; the generated getter has no non-atomic/thread-safety warning and returns a retained `NSString`

The crate accepts the category object directly. It does not call the nonatomic `HMAccessory.category` getter, create a category object, access profile or service values, create `HMHomeManager`, request permission, or enumerate accessories. The host owns HomeKit lifecycle, the HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the supplied object's source and lifetime

See [B449](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessoryCategory.localizedDescription`](https://developer.apple.com/documentation/homekit/hmaccessorycategory/localizeddescription), [`HMAccessoryCategory`](https://developer.apple.com/documentation/homekit/hmaccessorycategory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
