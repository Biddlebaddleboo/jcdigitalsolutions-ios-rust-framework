# PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md — Workstream D67: Row 074 Feasibility Gate

## Status

D67 found a truthful read-only status property, but no supported Rust call path or clear entitlement contract for a standalone row-074 slice. Keep row 074 at `X`. Do not treat this plan as implementation evidence or edit the canonical capability matrix. No B or G implementation gate starts until the evidence below closes.

## Objective

Assess whether Family Controls exposes a caller-triggered, non-prompting snapshot of the app's Family Controls authorization state that does not imply entitlement approval, access to Screen Time data, or capability to enforce controls.

## SDK and API evidence

- Inspected Xcode 26.6 build 17F113 and the iPhoneOS 26.5 SDK. `FamilyControls.framework` has a public Swift module interface at `System/Library/Frameworks/FamilyControls.framework/Modules/FamilyControls.swiftmodule/arm64e-apple-ios.swiftinterface`; the framework has no public Objective-C headers in this SDK.
- The Swift interface marks `FamilyControls.AuthorizationStatus` and `AuthorizationCenter` available from iOS 15.0. It exposes `AuthorizationCenter.shared` and the read-only `authorizationStatus` property.
- Apple's `authorizationStatus` docs say the initial value is always `.notDetermined`; the system sets it only after `requestAuthorization(for:)` succeeds, then updates it until successful revocation or app exit. Apple requires access to this property on the main queue. The read itself is a snapshot; observing changes uses the separate `@Published` publisher.
- The iOS 15 enum cases are `.notDetermined`, `.denied`, and `.approved`. The current iOS 26.5 SDK adds `.approvedWithDataAccess` at iOS 26.4. Apple defines that case as approval with access to non-tokenized family activity data. Preserve it separately in any future wrapper; do not treat it as equivalent to `.approved`.
- `AuthorizationCenter` and `AuthorizationStatus` are Swift APIs, not Objective-C APIs. No `objc2-family-controls` dependency or generated Rust binding is present in the current workspace, lockfile, or local Cargo source cache. This repo's Swift ABI plans prove ownership and selected value/async lowering only; they do not yet define a supported call path for this Swift singleton plus property getter.

## Authorization and entitlement boundary

- The exact Family Controls entitlement key is `com.apple.developer.family-controls`. Apple's entitlement docs require it before `requestAuthorization` or `revokeAuthorization`; distribution also requires Apple's entitlement approval. This row's status property does not itself show whether a signed host has that entitlement.
- Apple does not state in the inspected docs whether reading `AuthorizationCenter.shared.authorizationStatus` requires `com.apple.developer.family-controls`, or what the getter returns in an app without that entitlement. Do not claim that a successful status read proves entitlement approval.
- Apple's separate Family Controls App and Website Usage entitlement is `com.apple.developer.family-controls.app-and-website-usage`. Apple requires that capability before access to `FamilyActivityData`; it is not evidence that a status-only query reads activity data.
- `.approved` means a person, parent, or guardian approved the request to provide parental controls. It does not prove App Store entitlement approval, data-access entitlement, token selection, DeviceActivity monitoring, or ManagedSettings enforcement. `.approvedWithDataAccess` has its own stronger data-access meaning and must not imply that a status-only crate reads those data.
- `requestAuthorization(for:)` can present a system alert and, for an individual, Face ID or Touch ID authorization. D67 does not call it, call revoke, show a picker, read activity data, schedule DeviceActivity, or apply ManagedSettings.

## Feasibility blockers

1. Establish a supported zero-Swift-source Rust call path for the Swift `AuthorizationCenter.shared` getter and `authorizationStatus` getter, with compiler-derived ABI signatures and device/Simulator evidence. Do not use guessed mangled calls, private metadata, or raw Swift object layouts.
2. Obtain Apple documentation or framework guidance for whether the read-only status property is usable without `com.apple.developer.family-controls` and for its behavior when the entitlement is absent. Keep that separate from entitlement approval for distribution.
3. Define a main-queue contract compatible with this framework's static Rust API and preserve `.approvedWithDataAccess` as a distinct value on iOS 26.4+. Do not add an implicit executor, global runtime, or hidden queue hop.
4. Keep the contract limited to app authorization status. A `true`/approved result must not assert Screen Time data access, entitlement approval, or effective parental-control capability.

## Deferred work

- No portable facade, Rust backend, Swift shim, Objective-C shim, or B/G validation gate is authorized by D67.
- Row 074 remains `X`; row 075 DeviceActivity/ManagedSettings remains separate and out of scope.
- No workspace/lockfile, CI, canonical matrix, aggregate plan, or shared index edit is part of D67.

## Apple and repository references

- [AuthorizationCenter](https://developer.apple.com/documentation/familycontrols/authorizationcenter)
- [authorizationStatus](https://developer.apple.com/documentation/familycontrols/authorizationcenter/authorizationstatus)
- [AuthorizationStatus](https://developer.apple.com/documentation/familycontrols/authorizationstatus)
- [AuthorizationStatus.approvedWithDataAccess](https://developer.apple.com/documentation/familycontrols/authorizationstatus/approvedwithdataaccess)
- [Family Controls entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.family-controls)
- [Configuring Family Controls](https://developer.apple.com/documentation/xcode/configuring-family-controls)
- [Requesting the Family Controls entitlement](https://developer.apple.com/documentation/familycontrols/requesting-the-family-controls-entitlement)
- [PLAN_SWIFT_ABI.md](PLAN_SWIFT_ABI.md)
