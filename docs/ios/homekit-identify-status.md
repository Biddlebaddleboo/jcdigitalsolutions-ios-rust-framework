# iOS HomeKit accessory identify support and operation

`ios-homekit-identify-status` exposes `supports_identify(&HMAccessory) -> Result<bool, HomeKitIdentifyStatusError>` and `identify(&HMAccessory) -> HomeKitIdentifyFuture` on iOS

The caller supplies a live `HMAccessory` already obtained through its HomeKit host. `supports_identify` reads only `HMAccessory.supportsIdentify`, available from iOS 11.3; false means Apple documents that an identify request returns an error, while true does not guarantee later success

`identify` returns a lazy future. The request starts on first poll, yields `Ok(())` only when HomeKit calls back with no `NSError`, and copies a failure's original domain and code into `HomeKitIdentifyError::NativeFailure`. Dropping the future detaches its result and waker but does not cancel the native request; the completion retains the accessory and its callback state until HomeKit releases the block. A successful callback means only that HomeKit reports the request processed; it does not prove reachability or a visible accessory response

The public typed `HMAccessory.identifyWithCompletionHandler:` binding is available through the `objc2-home-kit` 0.3.2 `HMAccessory` and `block2` features. The method inherits `HMAccessory`'s iOS 8.0 floor. The registered device and Simulator profiles retain their existing iOS 11.3 and iOS 14.0 floors because this package also exposes the later `supportsIdentify` getter

The crate does not create `HMHomeManager`, request permission, enumerate accessories, configure HomeKit, or manage the host lifecycle. The host owns the HomeKit capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior authorization, and the supplied accessory's source/lifetime. This API does not establish HomeKit readiness, authorization, reachability, or visible identification

See [B339 and B488](../../PLAN_CAPABILITIES_HOMEKIT.md) and Apple's [`HMAccessory.supportsIdentify`](https://developer.apple.com/documentation/homekit/hmaccessory/supportsidentify?changes=_5&language=objc), [`HMAccessory.identifyWithCompletionHandler:`](https://developer.apple.com/documentation/homekit/hmaccessory/identify%28completionhandler%3A%29?language=objc), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory?language=objc), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
