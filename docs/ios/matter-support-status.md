# iOS MatterSupport API-support snapshot

`ios-matter-support-status` returns the current `MatterAddDeviceRequest.isSupported` value through
a compiler-derived C `swiftcall` thunk. The operation is available on iOS 17.0 and later

`Ok(true)` or `Ok(false)` preserves Apple's result for use of this request API only. The Boolean
does not show whether an app has an ecosystem topology, host extension, entitlement, home,
accessory, permission, Thread network, or a working commissioning path. The query does not create a
request, scan for devices, begin setup, or show system UI. Older iOS and non-iOS targets return
`NativeApiUnavailable`

The package contains no Swift source, uses no Objective-C selector, and adds no external Rust
dependency. Xcode 26.6 / iOS 26.5 supplied the local compiler and SDK evidence; this is below the
repository's Xcode 27.x baseline

See the [B233 MatterSupport plan record](../../PLAN_CAPABILITIES_MATTERSUPPORT.md) and the
[package README](../../platform/ios/ios-matter-support-status/README.md)
