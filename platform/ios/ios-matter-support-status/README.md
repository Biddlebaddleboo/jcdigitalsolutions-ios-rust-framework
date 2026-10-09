# iOS MatterSupport API-support snapshot

This crate exposes only `MatterAddDeviceRequest.isSupported` through a compiler-derived C
`swiftcall` thunk

The result means only that MatterSupport reports use of `MatterAddDeviceRequest` as supported on
the current device. It does not establish a configured ecosystem, entitlement, home, accessory,
Thread network, permission, commissioning, or future setup success. The query does not create a
request, scan, start setup, or show system UI

The API floor is iOS 17.0. Older iOS and non-iOS targets return
`MatterSupportError::NativeApiUnavailable`. The crate adds no Swift source and uses no Objective-C
selector or external Rust dependency

See [the focused guide](../../../docs/ios/matter-support-status.md) and
[B233 plan record](../../../PLAN_CAPABILITIES_MATTERSUPPORT.md)
