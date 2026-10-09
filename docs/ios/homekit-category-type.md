# iOS HomeKit accessory category-type snapshot

`ios-homekit-category-type` exposes `ios_homekit_category_type::category_type(&HMAccessoryCategory) -> Result<String, HomeKitCategoryTypeError>` on iOS

The caller supplies a live `HMAccessoryCategory` object already obtained through its HomeKit host. The function reads only `categoryType` and copies the returned `NSString` into a Rust-owned `String`. It preserves the raw category identifier, including unknown future values, without enum conversion or normalization. The result is an accessory classification string, not HomeKit support, authorization, reachability, profile availability, or readiness

`categoryType` is available from iOS 9.0; older iOS versions return `NativeApiUnavailable`. The getter is typed through `objc2-home-kit` 0.3.2's `HMAccessoryCategory` feature. The iOS 26.5 SDK declares the property readonly/copy without `nonatomic`, so it uses Objective-C's atomic default; the generated getter has no non-atomic/thread-safety warning and returns a retained `NSString`

The crate accepts the category object directly. It does not call the nonatomic `HMAccessory.category` getter, create `HMAccessoryCategory`, access profile or service values, create `HMHomeManager`, request permission, or enumerate accessories. The host owns HomeKit lifecycle, the HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the supplied object's source and lifetime

See [B445](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessoryCategory.categoryType`](https://developer.apple.com/documentation/homekit/hmaccessorycategory/categorytype), [Accessory Category Types](https://developer.apple.com/documentation/homekit/accessory-category-types), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
