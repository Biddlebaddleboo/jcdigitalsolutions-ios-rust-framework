# PLAN_CAPABILITIES_DEVICE_ACTIVITY.md — D71: DeviceActivity / ManagedSettings feasibility

## Objective

Audit row `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists` for a bounded, non-prompting Rust-callable slice. Inspect public API exposure, thread/UI behavior, Family Controls approval and entitlement requirements, and the privacy/data boundary. This is a feasibility audit only

## Status

The iOS 26.5 SDK exposes one potentially Rust-callable Objective-C status surface, `DeviceActivityAuthorization.isAuthorized`, from iOS 17.0. `objc2` can call a typed public Objective-C class without Swift ABI, but Apple documentation does not define the status property's exact semantics, threading guarantees, or whether this query alone requires the Family Controls entitlement. A raw Boolean may be callable but is not yet an honest portable capability contract; keep row 075 at `X` pending those facts

## Narrow candidate and unresolved contract

`DeviceActivityAuthorization` is a public `NSObject` subclass conforming to the public `@objc DeviceActivityAuthorizing` protocol. The iOS 26.5 Swift interface marks the class and protocol available from iOS 17.0 and exposes:

- `@objc static var isAuthorized: Bool { get }`
- `@objc static func isAuthorized(_ bundleIdentifier: String) -> Bool`

The class also exposes `authorizedClientIdentifiers`, `sharingEnabled`, and mutable `isOverridden`; do not include these in a minimal status slice. The class has an Objective-C runtime class entry in `DeviceActivity.tbd`, so a deliberately minimal `objc2` class binding is technically possible without a generated framework crate or Swift ABI. No binding or call was implemented in D71

The no-argument getter is the narrowest candidate: it is synchronous, read-only, has no UI or request parameter, and carries no `@MainActor` annotation in the public interface. The `bundleIdentifier` overload may query other clients and is excluded. Apple places the class under “Authorize access” and documents the getter signature, but the inspected docs do not say what authorization the Boolean represents, whether it reflects Family Controls authorization, whether the getter can trigger system UI, or what it returns when the host lacks entitlements. The absence of a prompt parameter and actor annotation is not a documented no-prompt or thread-safety guarantee

## Other API surfaces and exclusions

- `DeviceActivityCenter` is a Swift value type available from iOS 15.0. `activities`, `schedule(for:)`, and `events(for:)` inspect configured monitoring state; `startMonitoring` and `stopMonitoring` mutate system monitoring. `startMonitoring` can cause a `DeviceActivityMonitor` extension to receive callbacks as soon as the call if the interval is active. The API uses Swift `DeviceActivityName`, `DeviceActivitySchedule`, `DeviceActivityEvent`, and throws values, and has no generated Rust binding
- `ManagedSettingsStore` is a Swift class available from iOS 15.0. Its settings properties mutate access restrictions on the device. Apple states that the system determines the effective state and does not guarantee that specified settings govern device behavior. Do not wrap setting mutation as a status query or claim enforcement from a successful write
- `DeviceActivityData.activityData(filteredBy:using:)` is an async Swift API added in iOS 26.4. It returns activity data and has an `unauthorized` error state. App and website usage data has a distinct privacy boundary and entitlement; it is not a candidate for this status-only audit
- `FamilyControls.AuthorizationCenter.authorizationStatus` belongs to row 074 and its separate D67 audit. Do not duplicate it or infer its semantics from `DeviceActivityAuthorization.isAuthorized`
- `FamilyActivityPicker`, `DeviceActivityReport`, and ManagedSettings UI are SwiftUI/MainActor presentation APIs. No picker, report, extension callback, or UI path is in scope

## SDK, binding, and execution-context evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- `FamilyControls.framework`, `DeviceActivity.framework`, and `ManagedSettings.framework` provide public `.swiftinterface` and `.swiftdoc` files, but no public framework C headers or module maps in the iPhoneOS SDK
- The DeviceActivity interface declares `DeviceActivityAuthorization` as `@objc public class ... : ObjectiveC.NSObject, DeviceActivityAuthorizing`; its `isAuthorized` members carry `@objc`. The framework stub lists its Objective-C runtime class name `_TtC14DeviceActivity27DeviceActivityAuthorization`
- The installed `objc2` 0.6.5 generated-framework inventory marks `DeviceActivity`, `FamilyControls`, and `ManagedSettings` Swift-only, and the local Cargo cache has no `objc2-device-activity`, `objc2-family-controls`, or `objc2-managed-settings` crate. This does not erase the specific `@objc` class above; it means any Rust call would need a minimal typed binding, not a generated crate
- The inspected `DeviceActivityAuthorization`, `DeviceActivityCenter`, and `ManagedSettingsStore` declarations have no `@MainActor` annotation. The API documentation inspected also does not promise thread safety or specify a required queue. Do not infer unrestricted thread safety from the missing annotation
- The device `DeviceActivity.tbd` lists `arm64e-ios`; no compile/link or runtime probe was run. Compatibility with the repository's usual `aarch64-apple-ios` target remains unverified

## Entitlement, approval, and privacy boundaries

- Apple documents the Family Controls entitlement key as `com.apple.developer.family-controls`. The host app and each Screen Time API extension need the Family Controls capability configured; App Store distribution requires a separate Apple approval request for the app and included extensions
- Apple explicitly requires this entitlement before `AuthorizationCenter.requestAuthorization(for:)` or `revokeAuthorization(completionHandler:)`. Apple does not document whether calling only `DeviceActivityAuthorization.isAuthorized` requires the entitlement or what the getter returns without it; do not claim the query is entitlement-free
- App and website usage data has the separate `com.apple.developer.family-controls.app-and-website-usage` entitlement. Apple requires it before access to app and website usage information through Family Controls and Device Activity, and explicit person authorization remains required. Do not read, enumerate, persist, or expose tokens or activity records in a minimal status slice
- `authorizedClientIdentifiers` can expose client identifiers and is excluded. `sharingEnabled` and `isOverridden` have no sufficiently documented scope for this workstream and are excluded
- An authorization Boolean must not claim entitlement approval, user/parent consent, access to activity records, monitoring success, active restrictions, or effective device enforcement

## Recommendation and next step

Do not implement a Rust facade or change row 075 yet. First obtain primary documentation or Apple framework guidance that defines `DeviceActivityAuthorization.isAuthorized`, its prompt behavior, required entitlement, and threading contract. If those are established, the smallest implementation candidate is an iOS-only typed `objc2` binding for the no-argument getter only, returning an explicitly named point-in-time Boolean and claiming no monitoring, data access, or enforcement. Require device and Simulator compile/link evidence before integration

## Apple primary documentation

- [Device Activity framework](https://developer.apple.com/documentation/deviceactivity)
- [DeviceActivityAuthorization](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorization)
- [DeviceActivityAuthorizing](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorizing)
- [DeviceActivityAuthorizing.isAuthorized](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorizing/isauthorized)
- [DeviceActivityAuthorizing.isAuthorized(_:)](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorizing/isauthorized(_:))
- [DeviceActivityCenter](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter)
- [ManagedSettingsStore](https://developer.apple.com/documentation/managedsettings/managedsettingsstore)
- [Managed Settings](https://developer.apple.com/documentation/managedsettings)
- [Requesting the Family Controls entitlement](https://developer.apple.com/documentation/familycontrols/requesting-the-family-controls-entitlement)
- [Configuring Family Controls](https://developer.apple.com/documentation/xcode/configuring-family-controls)
- [Family Controls App and Website Usage entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.family-controls.app-and-website-usage)
- [Family Controls authorization](https://developer.apple.com/documentation/familycontrols)

No tests, builds, link probes, prompts, monitoring, or settings changes were performed

## B176 follow-up: no next truthful status slice

Rechecked the public iOS 26.5 declarations and Apple API pages for a distinct typed, read-only
operation beyond the authorization candidate in D71. No operation has a sufficiently defined,
privacy-safe contract for this Rust-only row. Keep row 075 unsupported (`X`); do not add an FFI
binding based solely on Objective-C callability.

- `DeviceActivityAuthorization.isAuthorized` remains the only direct authorization Boolean.
  Apple documents its `@objc static var isAuthorized: Bool { get }` signature but gives no
  discussion of what is authorized, no-prompt behavior, entitlement requirements for the query,
  or thread-safety guarantees. The separate `AuthorizationCenter.authorizationStatus` API has
  documented parental-control status semantics, but belongs to Family Controls row 074 and is
  not evidence for this Device Activity property.
- `DeviceActivityAuthorization.sharingEnabled` is another `@objc static` Boolean, but Apple
  documents only its signature, not what is shared or which sharing state it represents. Exposing
  it would give callers an ungrounded label, not a useful capability snapshot.
- `DeviceActivityAuthorization.authorizedClientIdentifiers` returns identifiers. Apple does not
  document its scope or a need for a caller to enumerate those identities for this status-only
  goal; omit it to avoid needless identity disclosure.
- `DeviceActivityAuthorization.isOverridden` is mutable. It is excluded from a read-only status
  facade; the public declaration alone does not define a safe purpose or contract for writes.
- `DeviceActivityCenter` monitoring state and operations are Swift value-type APIs with schedule
  and extension lifecycle semantics. `DeviceActivityData` exposes private activity information
  subject to the separate app-and-website-usage entitlement and explicit authorization. Neither
  is an interchangeable status query.

This is a source-backed no-go, not a claim that these symbols are unavailable to Objective-C. The
SDK contains the Objective-C runtime class `_TtC14DeviceActivity27DeviceActivityAuthorization`, but
the local generated `objc2` framework inventory marks DeviceActivity Swift-only and no dedicated
`objc2-device-activity` crate is available in the local Cargo registry. A handwritten binding
would therefore depend on precisely the undocumented semantics above. No dependency or code was
added, and no tests, builds, runtime calls, probes, or device queries were run for B176.

Primary API evidence: [DeviceActivityAuthorizing.isAuthorized](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorizing/isauthorized),
[DeviceActivityAuthorization.sharingEnabled](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorization/sharingenabled),
[DeviceActivityAuthorization.isOverridden](https://developer.apple.com/documentation/deviceactivity/deviceactivityauthorization/isoverridden),
[Family Controls authorization status](https://developer.apple.com/documentation/familycontrols/authorizationstatus),
and [Family Controls App and Website Usage entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.family-controls.app-and-website-usage)
