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

## B306 follow-up: `DeviceActivityCenter.activities` is not a support snapshot

Audited the distinct read-only `DeviceActivityCenter.activities` property beyond B176's authorization values. The iOS 26.5 interface declares `DeviceActivityCenter` as a Swift `struct`, available from iOS 15.0, with `activities: [DeviceActivityName] { get }`; `DeviceActivityName` is also a Swift struct with a `String` raw value. Apple's API page defines the result as activities that the application's extension currently monitors. It is the host app's configured monitoring state, not device or framework support, authorization, activity-data access, or proof that restrictions take effect.

The installed `DeviceActivity.framework` has a Swift module interface and `DeviceActivity.tbd`, but no public C headers or module map for this property. The local `objc2` generated-framework catalog marks `DeviceActivity` Swift-only, and no `objc2-device-activity` crate is present in the local registry. A typed Objective-C binding cannot call this Swift struct property. The repository's current Swift ABI boundary has no Rust-owned adapters for `[DeviceActivityName]` or its Swift string values; no array layout, element ownership, or Swift call ABI is inferred here. A future bridge would require a separate, compiler-derived Swift value/array ownership design and an explicit host use case for the configured activity-name snapshot.

Apple's Screen Time documentation says the Family Controls entitlement `com.apple.developer.family-controls` enables access to the Managed Settings and Device Activity frameworks; distribution use requires Apple's approval for the app and included Screen Time extensions. The separate `com.apple.developer.family-controls.app-and-website-usage` entitlement and explicit person authorization apply to access to app and website usage data; this property returns activity names, not that data. Neither entitlement is a general Device Activity support bit.

Decision: no B306 API or dependency change. Keep row `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists` at `X`; `activities` is useful only as configured activity-name state in a Screen Time host, not as a generic capability. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/DeviceActivity.framework/Modules/DeviceActivity.swiftmodule/arm64e-apple-ios.swiftinterface`; `DeviceActivity.tbd`; local `objc2` 0.6.5 `src/topics/about_generated/list_unsupported.md`; `PLAN_SWIFT_ABI.md`; Apple [`DeviceActivityCenter.activities`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter/activities), [Configuring Family Controls](https://developer.apple.com/documentation/xcode/configuring-family-controls), [Requesting the Family Controls entitlement](https://developer.apple.com/documentation/familycontrols/requesting-the-family-controls-entitlement), and [Family Controls App and Website Usage entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.family-controls.app-and-website-usage).

No source, manifest, dependency, lockfile, build, link probe, test, prompt, monitoring call, app launch, Simulator run, or device query was performed for B306

## B338 follow-up: `ManagedSettingsStore.isActive` is per-store state

Audited the iOS 26.5 `ManagedSettingsStore.isActive` getter as a distinct row 075 candidate. The installed public Swift interface declares `ManagedSettingsStore` from iOS 15.0, `init()`, and `isActive: Bool { get set }` from iOS 26.5. Apple defines this value as control over whether that specific store is active; an inactive store is excluded from effective-settings calculation, and the value defaults to `true`. This is a per-store config value, not a device/framework support or authorization snapshot, and it does not prove that the system enforces the store's settings.

The getter has no standalone Rust contract for row 075. A new default store starts with the documented `true` default, so it does not reveal useful host state. A meaningful read needs a concrete host-owned store and its settings lifecycle. Apple describes Managed Settings as part of the Screen Time suite, which uses Family Controls authorization; distribution requires Apple's Family Controls entitlement approval. The docs reviewed do not define a permission-free query contract for this property. The Swift interface has no `Sendable` or actor annotation for `ManagedSettingsStore`, and the mutable property has no concurrency or synchronization guarantee. A safe cross-host Rust API must not infer queue safety or the effective device state.

The property is Swift-only in the installed SDK: `ManagedSettings.framework/Headers` has no public headers, and the local registry/workspace has no `objc2-managed-settings` crate. Calling it from Rust would need a compiler-derived Swift ABI bridge as well as a selected host/store ownership and synchronization contract. Decision: no B338 facade or dependency change; keep row `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists` at `X`. Revisit only for a concrete Screen Time host that owns a named store and defines serialized access. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/ManagedSettings.framework/Modules/ManagedSettings.swiftmodule/arm64e-apple-ios.swiftinterface:381-405`; Apple [`ManagedSettingsStore.isActive`](https://developer.apple.com/documentation/managedsettings/managedsettingsstore/isactive), [`ManagedSettingsStore`](https://developer.apple.com/documentation/managedsettings/managedsettingsstore?changes=_3), [Configuring Family Controls](https://developer.apple.com/documentation/xcode/configuring-family-controls), and [Managed Settings authorization](https://developer.apple.com/documentation/managedsettings/connectionwithframeworks?changes=l__3__8).

No source, dependency, runtime call, authorization request, store creation, settings change, build, link probe, test, app launch, Simulator run, or device query was performed for B338

## B376 audit — per-name schedule retrieval is not a row capability snapshot

B376 audited the distinct read-only `DeviceActivityCenter.schedule(for:)` operation. The iOS 26.5 public Swift interface declares `DeviceActivityCenter` as an iOS 15.0 `struct` and the method as `schedule(for activity: DeviceActivityName) -> DeviceActivitySchedule?`. Both input and output are Swift value types; `DeviceActivitySchedule` contains `Foundation.DateComponents` values and its optional return is resilient Swift ABI data.

Apple documents the method as fetching the schedule for a named activity, while `DeviceActivityCenter.activities` separately reports names the app's extension currently monitors. A schedule is caller-app monitoring configuration, not Device Activity framework support, Family Controls authorization, activity-data access, proof that monitoring is active, or proof that the system invokes extension callbacks. Apple's schedule page does not define a standalone interpretation for the optional `nil` result; do not turn it into a generic `is_supported` or `is_monitoring` Boolean.

The installed DeviceActivity SDK has a Swift module interface and `DeviceActivity.tbd`, but no public C headers or module map for `DeviceActivityCenter.schedule(for:)`. The local `objc2` 0.6.5 generated-framework catalog marks DeviceActivity Swift-only, and no `objc2-device-activity` binding exists in the local registry. A Rust API would require a separately compiler-derived Swift ABI bridge for the method, `DeviceActivityName`, and the resilient optional `DeviceActivitySchedule` value; no layout or ownership is inferred. The existing audit does not select an app-owned monitoring use case or establish a callback queue/thread contract.

Decision: no B376 facade, manifest change, or dependency change. Keep row `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists` at `X`; this API is host configuration state and does not complete a standalone capability contract. Revisit only with a selected Screen Time host and compiler-derived value/ownership bridge. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/DeviceActivity.framework/Modules/DeviceActivity.swiftmodule/arm64e-apple-ios.swiftinterface:154-165`; `DeviceActivity.tbd`; local `objc2` 0.6.5 `src/topics/about_generated/list_unsupported.md`; Apple [`DeviceActivityCenter.schedule(for:)`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter/schedule%28for%3A%29), [`DeviceActivityCenter`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter), [`DeviceActivityCenter.activities`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter/activities), and [`DeviceActivitySchedule`](https://developer.apple.com/documentation/deviceactivity/deviceactivityschedule).

No source, manifest, dependency, lockfile, build, link probe, test, prompt, monitoring call, app launch, Simulator run, or device query was performed for B376

## B383 audit — `DeviceActivityCenter.events(for:)` is host configuration only

B383 audited `DeviceActivityCenter.events(for:)` as a distinct read-only query beyond B306's activity-name list and B376's schedule lookup. The iOS 26.5 Swift interface declares the method on the iOS 15.0 `DeviceActivityCenter` struct as `events(for activity: DeviceActivityName) -> [DeviceActivityEvent.Name: DeviceActivityEvent]`. Apple documents that it fetches the named activity's events, returns an empty dictionary when the app does not monitor that activity, and returns a static representation at call time.

A narrow host-specific candidate could count the configured events for one caller-supplied activity name and return only that integer, without exposing the event dictionary, `FamilyActivitySelection` tokens, or app/website usage data. That would report only the calling app's configured monitoring state; it would not establish framework support, authorization, live usage, effective restrictions, callback delivery, or monitoring success. No app-owned Screen Time host/use case is selected in this audit.

The method and its `DeviceActivityName`, `DeviceActivityEvent.Name`, and dictionary values are Swift-only value types. The installed SDK has no public DeviceActivity C header/module map, and the local `objc2` 0.6.5 generated-framework catalog marks DeviceActivity Swift-only; no `objc2-device-activity` binding is present. A count-only Rust facade would still need a compiler-derived Swift ABI bridge for the input string/name and method call, plus evidence for the host-compatible execution contract. Apple does not document a required executor or thread-safety guarantee for this query. Do not infer a stable dictionary layout or cross it into Rust.

Decision: conditional host-specific candidate only; no B383 source, dependency, or manifest change. Keep row `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists` at `X` because an event count is configured app state, not the row's general capability/support contract. Reconsider only for a selected host with a compiler-derived count-only bridge and an explicit execution/authorization boundary. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/DeviceActivity.framework/Modules/DeviceActivity.swiftmodule/arm64e-apple-ios.swiftinterface:154-165`; `DeviceActivity.tbd`; local `objc2` 0.6.5 `src/topics/about_generated/list_unsupported.md`; Apple [`DeviceActivityCenter.events(for:)`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter/events%28for%3A%29), [`DeviceActivityCenter`](https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter), and [Device Activity](https://developer.apple.com/documentation/deviceactivity).

No source, manifest, dependency, lockfile, build, link probe, test, prompt, monitoring call, app launch, Simulator run, or device query was performed for B383
