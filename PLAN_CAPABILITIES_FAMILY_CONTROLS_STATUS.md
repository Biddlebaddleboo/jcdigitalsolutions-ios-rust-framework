# Workstream D67/B80: Family Controls status snapshot

## Status

The bounded iOS status adapter and local ABI gates pass. They use a compiler-matched C `swiftcall` bridge with no Swift source. Host, device, and Simulator check, strict Clippy, rustdoc, compiler-oracle, and link/import gates pass; no test or linked probe ran

Row 074 is partial `B` for the raw status snapshot only. The row stays platform-exclusive. No portable contract or row 075 DeviceActivity/ManagedSettings support is in scope

## Objective

Read one signed `AuthorizationStatus.RawValue` snapshot from `AuthorizationCenter.shared.authorizationStatus` on the main dispatch queue, without authorization request, UI, activity data, or control operations

## API and bounds

- Package: `platform/ios/ios-family-controls-status`
- Guide: `docs/ios/family-controls-status.md`
- API: `unsafe authorization_status_raw_value_on_main_queue() -> Result<AuthorizationStatusRawValue, AuthorizationStatusError>`
- Floor: iOS 15.0
- Value: exact signed Swift `Int` as `i64`; do not guess enum storage or hard-code raw values
- `approvedWithDataAccess` is a distinct Swift case from iOS 26.4. Its raw value passes through without a Rust case map; unknown future raw values also pass through
- Apple requires this getter on the main dispatch queue. The API is `unsafe` so its safety contract requires that queue; it does not infer queue identity from the main thread or hop queues
- No portable facade, authorization request or revoke, prompt, picker, activity data, DeviceActivity, ManagedSettings, or C ABI is part of this slice
- The iOS 15 cases are `.notDetermined`, `.denied`, and `.approved`; the iOS 26.4+ case is `.approvedWithDataAccess`. This API returns raw values only and does not map or collapse any case
- Apple docs say the initial value is `.notDetermined`; a successful `requestAuthorization(for:)` updates it, then it may change after revocation or app exit. The property read is a snapshot; change observation uses the separate `@Published` publisher
- The exact Family Controls entitlement key is `com.apple.developer.family-controls`. Apple requires it before `requestAuthorization` or `revokeAuthorization`; distribution also requires Apple's entitlement approval. The host owns this setup for its broader feature
- Apple does not state whether this read alone needs that entitlement or define its value when absent. This snapshot neither inspects nor proves entitlement presence, distribution approval, activity-data access, or control use; no absent-entitlement behavior is claimed
- `.approved` means a person, parent, or guardian approved the request for parental controls. It does not prove distribution approval, data-access entitlement, token selection, DeviceActivity monitoring, or ManagedSettings enforcement
- The separate `com.apple.developer.family-controls.app-and-website-usage` entitlement is for `FamilyActivityData`; it is not a status-query requirement or evidence that this crate reads activity data. `.approvedWithDataAccess` stays a distinct raw value; this crate does not access that data
- `requestAuthorization(for:)` may show a system alert and, for an individual, Face ID or Touch ID. D67 does not call it, call revoke, show a picker, read activity data, schedule DeviceActivity, or apply ManagedSettings

## SDK and ABI evidence

- Inspected Xcode 26.6 build 17F113 and the iPhoneOS 26.5 SDK. `FamilyControls.framework` has a public Swift module interface but no public Objective-C declaration for these APIs
- The SDK marks `AuthorizationStatus` and `AuthorizationCenter` as iOS 15.0 APIs; the interface marks `approvedWithDataAccess` as iOS 26.4+
- Device and arm64 Simulator Swift LLVM IR at min iOS 15.0 derives the same public calls: `_$s14FamilyControls19AuthorizationStatusOMa`, `_$s14FamilyControls19AuthorizationCenterCMa`, `_$s14FamilyControls19AuthorizationCenterC6sharedACvgZ`, `_$s14FamilyControls19AuthorizationCenterC19authorizationStatusAA0cF0OvgTj`, and `_$s14FamilyControls19AuthorizationStatusO8rawValueSivg`
- Swift IR returns the singleton as owned; the compiler calls `_swift_release` after the property getter. The property result uses indirect result storage; the raw getter returns signed i64; the compiler calls the VWT destroy witness after the raw read
- Compiler IR gives the VWT as eight pointer witnesses then `size`, `stride`, `flags`, and extra-inhabitant count. Metadata holds the VWT pointer one word before the metadata pointer
- The C bridge reads size and the low eight alignment-mask bits from the runtime VWT, allocates by that layout, calls the destroy witness, then frees the value. It uses `malloc` for alignment no greater than `_Alignof(max_align_t)` and `posix_memalign` for larger valid alignment; it does not use a guessed fixed buffer
- Swift ABI source defines the VWT alignment mask as `0x000000FF`; the gate checks the compiler VWT shape and bridge mask
- The Swift/Clang ABI gate uses min iOS 15.0 / SDK 26.5 on arm64 device and Simulator. The final Rust link probes use rustc target defaults: device minos 10.0 and Simulator minos 14.0. `FamilyControls.framework` is `LC_LOAD_WEAK_DYLIB`, and each FamilyControls symbol is a weak external; the bridge returns `NativeApiUnavailable` if a weak symbol is absent. Imports are only `FamilyControls.framework`, `/usr/lib/swift/libswiftCore.dylib`, and `libSystem.B.dylib`
- These link facts support a weak-link path below the API floor; no runtime behavior on any iOS release was tested, and no linked artifact ran

## Validation

`platform/ios/ios-family-controls-status/scripts/check-swift-abi.sh` emits Swift oracle and C bridge LLVM IR for device and Simulator, then checks the VWT shape, call signatures, C-convention destroy and release calls, dynamic allocation fields, and output path. It adds no Swift source to the repository

`platform/ios/ios-family-controls-status/scripts/check-link-imports.sh` builds a small probe for device and Simulator and checks `otool -L` against the exact allowlist plus `nm -u` for each required symbol. It does not execute the probes

The full package gate is `sh platform/ios/ios-family-controls-status/check.sh`. It passed. No tests, permission prompts, runtime query, or consumer execution took place

## Apple and ABI references

- [AuthorizationCenter](https://developer.apple.com/documentation/familycontrols/authorizationcenter)
- [authorizationStatus](https://developer.apple.com/documentation/familycontrols/authorizationcenter/authorizationstatus)
- [AuthorizationStatus](https://developer.apple.com/documentation/familycontrols/authorizationstatus)
- [AuthorizationStatus.approvedWithDataAccess](https://developer.apple.com/documentation/familycontrols/authorizationstatus/approvedwithdataaccess)
- [Family Controls entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.family-controls)
- [Configuring Family Controls](https://developer.apple.com/documentation/xcode/configuring-family-controls)
- [Swift VWT flags](https://github.com/swiftlang/swift/blob/main/include/swift/ABI/MetadataValues.h)
- [Swift type metadata](https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst)
