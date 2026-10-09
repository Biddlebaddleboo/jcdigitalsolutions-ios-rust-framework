# iOS HomeKit accessory identify-support snapshot

`ios-homekit-identify-status` exposes `ios_homekit_identify_status::supports_identify(&HMAccessory) -> Result<bool, HomeKitIdentifyStatusError>` on iOS

The caller supplies a live `HMAccessory` already obtained by its HomeKit host. The function reads only `HMAccessory.supportsIdentify`. `Ok(false)` means Apple documents that `identifyWithCompletionHandler:` returns an error; `Ok(true)` does not guarantee that a later identify call succeeds. The function does not invoke identify, create `HMHomeManager`, request permission, enumerate accessories, or report HomeKit readiness, authorization, or reachability

The getter is available from iOS 11.3. Older iOS versions return `NativeApiUnavailable`. The package links `HomeKit.framework` through `objc2-home-kit` 0.3.2 with only the `HMAccessory` feature. The host owns the HomeKit capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior HomeKit authorization, and the supplied accessory's lifetime/source

This is a per-accessory action-support snapshot only. It does not establish that the accessory is reachable, that identify will succeed, or that a host can use any other HomeKit API

See [B339](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessory.supportsIdentify`](https://developer.apple.com/documentation/homekit/hmaccessory/supportsidentify?changes=_5&language=objc), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory?language=objc), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
