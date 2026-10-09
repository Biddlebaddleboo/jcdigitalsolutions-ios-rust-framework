# iOS HomeKit accessory profile-count snapshot

`ios-homekit-accessory-profile-count` exposes `ios_homekit_accessory_profile_count::profile_count(&HMAccessory) -> Result<usize, HomeKitAccessoryProfileCountError>` on iOS

The caller supplies a live `HMAccessory` already obtained by its HomeKit host. The function reads only the retained `HMAccessory.profiles` array length. It does not inspect profile objects, services, characteristics, controls, or identifiers. A count does not establish profile-feature availability, reachability, operation success, authorization, or HomeKit readiness

The getter is available from iOS 11.0. Older iOS versions return `NativeApiUnavailable`. The package links `HomeKit.framework` through `objc2-home-kit` 0.3.2 with only the `HMAccessory` and `HMAccessoryProfile` features. The host owns the HomeKit capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior HomeKit authorization, and the supplied accessory's lifetime/source

The crate does not create `HMHomeManager`, request permission, enumerate accessories, or invoke a HomeKit operation. This is a per-accessory profile count only, not general HomeKit support or proof that any profile feature works

See [B394](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessory.profiles`](https://developer.apple.com/documentation/homekit/hmaccessory/profiles?changes=_7_1&language=objc), [`HMAccessoryProfile`](https://developer.apple.com/documentation/homekit/hmaccessoryprofile?language=objc), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
