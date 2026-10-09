# PLAN_CAPABILITIES_DOCKKIT.md — D78: DockKit feasibility for row 097

## Objective

Audit row `097-maps-ar-spatial-dockkit` for a small, honest Rust-accessible DockKit API slice, including API floors, SDK/binding support, host setup, privacy, actor/callback, and lifecycle limits

## Status

The installed public API has two potentially useful state surfaces, but no direct Rust binding: `DockAccessoryManager.isSystemTrackingEnabled` is a synchronous setting value, and `DockAccessoryManager.accessoryStateChanges` is an asynchronous sequence of dock/undock events. `DockKit` is Swift-only in the installed generated-binding inventory, and no generated DockKit crate is cached or present in `Cargo.lock`

Keep row 097 at `X` until a supported Swift ABI or C boundary exists and a concrete subset is implemented. The sync Boolean can be a separate narrow candidate if its exact semantics are acceptable; it does not prove a dock is connected, tracking is active, camera access is granted, or the device can operate a compatible accessory. The event sequence carries real accessory state but requires an async lifecycle bridge and cannot be assumed to provide an initial snapshot

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

No safe direct Rust DockKit slice is available from the installed public Rust bindings. Keep row 097 at `X`; do not infer DockKit support from iOS version, framework presence, camera authorization, or `isSystemTrackingEnabled`

If a separate approved Swift ABI task establishes a synchronous, ownership-safe path, `system_tracking_enabled()` may be considered as a deliberately limited scalar with the semantics above. If the desired value is actual dock presence, the candidate is `accessoryStateChanges`, which requires a supported async stream bridge and an explicit task/handle lifecycle contract. Neither candidate is implemented by this audit

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
