# D67/B80 Family Controls authorization status

`ios-family-controls-status` reads the signed `AuthorizationStatus.RawValue` from
`AuthorizationCenter.shared.authorizationStatus` on iOS 15.0 and later

```rust,ignore
// SAFETY: this call runs on the main dispatch queue
let status = unsafe {
    ios_family_controls_status::authorization_status_raw_value_on_main_queue()?
};
let raw_value: i64 = status.raw_value();
```

Apple requires this property read on the main dispatch queue. The unsafe call makes the queue
contract visible; the API does not hop queues

The initial Apple value is `.notDetermined`. A successful `requestAuthorization(for:)` may change
the snapshot; a read does not request or observe that change. The iOS 15 cases are `.notDetermined`,
`.denied`, and `.approved`. iOS 26.4 adds `.approvedWithDataAccess`

The result keeps the exact signed Swift `Int`, so the iOS 26.4+ `approvedWithDataAccess` case stays
distinct and an unknown future raw value stays intact. The crate does not guess enum storage or map
unknown values

This read does not request or revoke authorization, show UI, read activity data, inspect
DeviceActivity or ManagedSettings, or prove the Family Controls entitlement, distribution approval,
activity-data access, or control use. `com.apple.developer.family-controls` is a host setup need
for authorization requests and the broader Family Controls feature. Apple does not state that this
status read alone requires the entitlement

`.approved` does not prove Apple's distribution approval, data-access entitlement, token selection,
DeviceActivity monitoring, or ManagedSettings enforcement. The separate
`com.apple.developer.family-controls.app-and-website-usage` entitlement is for `FamilyActivityData`;
this crate does not read those data

The C `swiftcall` bridge uses compiler-matched public symbols, runtime VWT size and alignment,
the VWT destroy witness, and `_swift_release` for the owned singleton. It returns a bridge result
code separate from the signed raw value. No Swift source is part of the package

The local ABI script emits Swift and C LLVM IR for arm64 device and Simulator at iOS 15.0. The link
script builds probes, then checks `otool -L`, weak-load commands, and symbols; it does not run them.
The Rust link defaults are device minos 10.0 and Simulator minos 14.0, while the FamilyControls
framework and symbols are weak imports. This records a back-deploy link path only; it does not prove
runtime behavior below iOS 15.0, entitlement state, live status, or Apple parity
