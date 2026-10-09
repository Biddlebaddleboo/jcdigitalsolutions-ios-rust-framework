# PLAN_CAPABILITIES_DOCKKIT.md — D78: DockKit feasibility for row 097

## Objective

Audit row `097-maps-ar-spatial-dockkit` for a small, honest Rust-accessible DockKit API slice, including API floors, SDK/binding support, host setup, privacy, actor/callback, and lifecycle limits

## Status

The installed public API has two potentially useful state surfaces: `DockAccessoryManager.isSystemTrackingEnabled` is a synchronous setting value, and `DockAccessoryManager.accessoryStateChanges` is an asynchronous sequence of dock/undock events. `DockKit` is Swift-only in the installed generated-binding inventory. B245 now implements the synchronous setting snapshot on physical iOS through compiler-derived `swiftcall`; the Simulator backend returns `NativeApiUnavailable`. It does not prove a dock is connected, tracking is active, camera access is granted, or the device can operate a compatible accessory. The event sequence still requires an async lifecycle bridge and cannot be assumed to provide an initial snapshot

## SDK and Rust binding evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- The iPhoneOS SDK exposes `System/Library/Frameworks/DockKit.framework/Headers/DockKit.h`, `Modules/module.modulemap`, `Modules/DockKit.swiftmodule/arm64e-apple-ios.swiftinterface`, and `DockKit.tbd` under `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk`
- `DockKit.h` imports only Foundation; its public declarations do not expose `DockAccessoryManager`, `DockAccessory`, or their state values to Objective-C or C
- `DockAccessoryManager`, `DockAccessory`, `DockKitError`, and their public members are available from iOS 17.0. The sync property `isSystemTrackingEnabled` and throwing `accessoryStateChanges` getter share this floor
- `accessoryEvents` and its event values are available from iOS 17.4. `trackingStates` and `batteryStates` are available from iOS 18.0; these streams and the associated tracking/battery values are not needed for a minimal status facade
- The installed `objc2` 0.6.5 catalog at `src/topics/about_generated/list_unsupported.md` marks `DockKit` Swift-only. No `objc2-dock-kit` source is present in the local Cargo registry cache or root `Cargo.lock`
- The Swift interface declares `DockAccessoryManager` and `DockAccessory` as public Swift classes, not `NSObject` subclasses or `@objc` interfaces. `DockKit.tbd` exposes Swift-mangled symbols, not a public C status function
- The inspected declarations have no `@MainActor` annotation. `DockAccessoryManager` and `DockAccessory` are marked `@unchecked Sendable`; `accessoryStateChanges` is an `AsyncSequence`. The declarations do not name a callback queue or promise that events arrive on the main thread

## Candidate status and limits

### `isSystemTrackingEnabled`

Apple defines this Boolean as whether system tracking is enabled. `DockAccessoryManager.shared.isSystemTrackingEnabled` is the smallest synchronous state value in the SDK and does not itself call the setter or start accessory control

This reports a system-tracking setting only. `true` is not evidence that an accessory is connected, a camera stream is active, a subject is being tracked, or that the device/accessory supports a requested operation. Do not name or map it as `dockkit_supported`, `accessory_connected`, `tracking_active`, or camera authorization

The getter is Swift-only and depends on obtaining `DockAccessoryManager.shared`; no safe generated Rust call path is installed. Direct calls to observed Swift-mangled symbols would need compiler-derived ABI and object-lifetime proof. The current repo’s Swift ABI plan does not establish a reusable DockKit object/property adapter

### `accessoryStateChanges`

Apple defines this value as a stream of dock/undock state changes. Each `DockAccessory.StateChange` carries an optional `DockAccessory`, a state, and a tracking-button flag. Apple says a dock/undock notification occurs when someone docks or removes an iPhone from a compatible dock accessory

This stream can support an honest event API after a Rust/Swift async bridge exists. It is not a synchronous hardware-support query, not a documented initial-state snapshot, and not a direct Objective-C callback. A future facade must define task creation/cancellation, sequence errors, accessory-handle ownership, and state updates after undock. The repo’s Swift ABI async audit does not establish a supported public task-entry/context/resume contract

`DockKitError.notSupportedByDevice` means a requested operation is unsupported by the device; it is an operation error, not a side-effect-free availability query. Do not invoke motion, tracking, or camera operations as a way to probe capability

## Entitlement, camera, and host constraints

- The reviewed DockKit public headers and Apple DockKit API pages do not document a DockKit-specific entitlement or DockKit-only `Info.plist` usage key. This is limited to the reviewed public surface, not a claim about private system policy
- DockKit integrates with camera-enabled apps. Any app that accesses the camera through AVFoundation must include `NSCameraUsageDescription` and request camera authorization; that is a separate camera permission flow, not an entitlement-free guarantee for every DockKit API
- `DockKitError.cameraTCCMissing` specifically means the person has not accepted the camera terms and conditions required for DockKit camera access. Apple directs the app to show an alert for those terms; this is distinct from `AVAuthorizationStatus` and the `NSCameraUsageDescription` permission prompt
- Apple describes system tracking as starting when a person docks an iPhone to a compatible motorized stand and launches the Camera app. DockKit app control also requires an actual compatible DockKit accessory; an iOS version check or framework link cannot prove that hardware is present
- Apple’s DockKit camera-app sample states that Simulator cannot access device cameras or connect to a DockKit device, and that the sample needs an iPhone running iOS 18 or later. This sample/runtime requirement does not raise the SDK-declared iOS 17 API floor
- `DockAccessoryManager.setSystemTrackingEnabled(_:)` changes tracking behavior and is async/throwing. It is not part of a read-only status API; do not call it as a probe or silently change the app/system setting

## Recommendation

No safe direct Rust DockKit slice is available from the installed public Rust bindings. B245 makes row 097 partial (`B`) for the physical-device system-tracking setting only; do not infer DockKit support from iOS version, framework presence, camera authorization, or `isSystemTrackingEnabled`

B245 implements the deliberately limited scalar `system_tracking_enabled()` with the semantics above. Actual dock presence remains a separate question; `accessoryStateChanges` requires a supported async stream bridge and an explicit task/handle lifecycle contract. Neither candidate is implemented by this audit

## Apple primary sources

- [DockKit framework](https://developer.apple.com/documentation/dockkit)
- [DockAccessoryManager](https://developer.apple.com/documentation/dockkit/dockaccessorymanager)
- [DockAccessoryManager.isSystemTrackingEnabled](https://developer.apple.com/documentation/dockkit/dockaccessorymanager/issystemtrackingenabled)
- [DockAccessoryManager.accessoryStateChanges](https://developer.apple.com/documentation/dockkit/dockaccessorymanager/accessorystatechanges)
- [DockAccessory.StateChange](https://developer.apple.com/documentation/dockkit/dockaccessory/statechange)
- [DockKitError.notSupportedByDevice](https://developer.apple.com/documentation/dockkit/dockkiterror/notsupportedbydevice)
- [DockKitError.cameraTCCMissing](https://developer.apple.com/documentation/dockkit/dockkiterror/cameratccmissing)
- [Controlling a DockKit accessory using your camera app](https://developer.apple.com/documentation/dockkit/controlling-a-dockkit-accessory-using-your-camera-app)
- [Requesting authorization to capture and save media](https://developer.apple.com/documentation/avfoundation/requesting-authorization-to-capture-and-save-media)
- [NSCameraUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nscamerausagedescription)

No tests, builds, link probes, runtime probes, camera prompts, or accessory control were performed

## B197 follow-up: DockKit state remains Swift-only

Rechecked row 097 against the installed iOS 26.5 headers and Swift interface plus Apple's current
API documentation. `DockKit.h` imports Foundation but declares no manager or state property.
`DockAccessoryManager.shared` and `isSystemTrackingEnabled` are Swift declarations; Apple's docs
define the Boolean only as whether system tracking is enabled. It does not report accessory
presence, active camera tracking, or device support. The corresponding Swift setter changes the
system-tracking mode and is excluded from a read-only contract.

`accessoryStateChanges` is the other plausible typed surface, but the public type is a throwing
Swift `AsyncSequence` of dock/undock changes, not an Objective-C callback or a documented initial
snapshot. It needs a task/sequence lifecycle and access to Swift `DockAccessory.StateChange`
values. The local generated-binding catalog still marks DockKit Swift-only, and the framework
stub exposes Swift-mangled symbols rather than C functions. Thus neither candidate is a
Rust-callable Objective-C/C operation within this audit's constraints.

B197 adds no ABI bridge, dependency, or code and leaves row `097-maps-ar-spatial-dockkit` at `X`.
The inspected toolchain is Xcode 26.6 build `17F113` with iPhoneOS SDK 26.5; the Xcode 27.x
baseline caveat remains open.

Primary API evidence: [DockAccessoryManager](https://developer.apple.com/documentation/dockkit/dockaccessorymanager),
[isSystemTrackingEnabled](https://developer.apple.com/documentation/dockkit/dockaccessorymanager/issystemtrackingenabled),
[accessoryStateChanges](https://developer.apple.com/documentation/dockkit/dockaccessorymanager/accessorystatechanges),
[DockAccessory.StateChanges](https://developer.apple.com/documentation/dockkit/dockaccessory/statechanges),
and [DockKit](https://developer.apple.com/documentation/dockkit)

No tests, builds, sequence iteration, state changes, camera access, accessory operations, app
launches, Simulator or device calls, or runtime probes were performed for B197

## B245 implementation: system-tracking setting only

B245 adds `ios-dockkit-status::system_tracking_enabled()` on physical iOS devices. Its sole result
is `DockAccessoryManager.shared.isSystemTrackingEnabled`: whether the system-tracking setting is
enabled. It does not report DockKit device support, dock presence, active subject tracking, camera
authorization, connected accessories, or operation success. It does not call the async setter or
observe the accessory event sequence

The public API is Swift-only. Compiler IR confirms metadata accessor → owned shared manager getter
→ synchronous Bool property getter → `swift_release` on arm64 iOS. The device bridge uses exact
weak-import `swiftcall` declarations and `swift-abi-core/apple-runtime`; it adds no Swift source,
Objective-C selector, or generated binding. No `@MainActor` annotation appears in the public
interface; this package claims no main-thread or queue guarantee

The installed iOS Simulator 26.5 SDK has no `DockKit.framework` or Swift module. The package
therefore compiles with a Simulator backend that returns `DockKitError::NativeApiUnavailable`; it
does not infer device state from Simulator. Device compiler-oracle and static link/import checks
pass at min iOS 17.0; no linked library was executed. The local toolchain remains Xcode 26.6 / iOS
26.5, below the repository's Xcode 27.x baseline

Capability-row impact: row `097-maps-ar-spatial-dockkit` is partial (`B`) for
this system-tracking setting snapshot only. The `B` claim excludes dock/undock events, accessory
identity, camera integration, control, and full DockKit parity. Root owns any aggregate matrix edit

The focused B245 gate runs format, host/device/Simulator `cargo check`, strict Clippy, rustdoc,
`docs-check`, device Swift/C compiler-oracle checks, and static DockKit link/import checks. It runs
no tests, app, accessory, camera, state change, or runtime query

Changed paths: `platform/ios/ios-dockkit-status/`, `docs/ios/dockkit-status.md`, and this focused
plan only. Root aggregate plans and capability manifest now record row 097 as partial (`B`)
