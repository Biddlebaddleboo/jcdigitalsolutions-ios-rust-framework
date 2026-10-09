# iOS HomeKit setup-result accessory identifier count

`ios-homekit-setup-result-count` exposes `ios_homekit_setup_result_count::setup_accessory_identifier_count(&HMAccessorySetupResult) -> Result<usize, HomeKitSetupResultCountError>` on iOS

The caller supplies a live `HMAccessorySetupResult` produced by its own successful HomeKit accessory-setup flow. The function reads only the length of `accessoryUniqueIdentifiers`; it does not inspect, copy, or return home or accessory identifiers. Apple documents that the result array contains identifiers for accessories set up and may contain multiple identifiers when setup adds a bridge

The result means only the number of identifier entries recorded in this setup result. It is not a count of the current home inventory and does not establish reachability, ongoing availability, future operation success, or general HomeKit readiness. The API is available from iOS 15.4; older iOS versions return `NativeApiUnavailable`

The iOS 26.5 SDK declares `accessoryUniqueIdentifiers` readonly/copy without `nonatomic`, so the property uses Objective-C's atomic default. The `objc2-home-kit` 0.3.2 typed getter has no non-atomic/thread-safety warning and returns a retained `NSArray<NSUUID>`; safe `NSArray::len()` reads its count while the array remains retained. The crate never accesses an array element

The host owns the setup flow and result object, HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the result object's source/lifetime. This package does not create `HMAccessorySetupManager`, present setup UI, prompt, query a manager, or access services or profiles

See [B452](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessorySetupResult`](https://developer.apple.com/documentation/homekit/hmaccessorysetupresult), [`accessoryUniqueIdentifiers`](https://developer.apple.com/documentation/homekit/hmaccessorysetupresult/accessoryuniqueidentifiers?changes=_5&language=objc), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
