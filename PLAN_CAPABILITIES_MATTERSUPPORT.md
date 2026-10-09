# PLAN_CAPABILITIES_MATTERSUPPORT.md — D91: Row 105 Feasibility Gate

## Status

D91 found one narrow, non-prompting candidate: `MatterAddDeviceRequest.isSupported`, an iOS 17+ Swift property. Its signature does not require an ecosystem topology or a setup request, but Apple defines its result only as support for use of that request. It is not a generic Matter, Thread, Bluetooth, accessory, entitlement, or commissioning-readiness result. It could serve as a preflight for an app that already offers Matter setup, but the installed Rust binding catalog has no MatterSupport path. Keep row `105-extension-entitlement-capabilities-mattersupport` at `X` until a supported Rust interop path and a scoped contract exist. No implementation or matrix change is part of D91.

## Objective

Determine whether row 105 has a narrow public Rust-callable operation useful to a general app, independent of its Matter ecosystem, accessory setup, entitlement-dependent payload choice, or user-present lifecycle.

## Installed SDK and binding evidence

- Inspected Xcode 26.6 build `17F113` and the iPhoneOS 26.5 SDK. The installed MatterSupport Swift interface was built with Swift 6.3.2.
- `MatterSupport.framework/Headers/MatterSupport.h` imports only Foundation. Its module map exports the framework module; the public API is declared in `MatterSupport.framework/Modules/MatterSupport.swiftmodule/arm64e-apple-ios.swiftinterface`.
- `MatterAddDeviceRequest` is available from iOS 16.1. Its `static var isSupported: Bool` is available from iOS 17.0. Apple describes the property as a flag for support of `MatterAddDeviceRequest` usage; it does not expose a result for an arbitrary app or Matter accessory.
- `MatterAddDeviceRequest.perform() async throws` is available from iOS 16.1. Apple documents that it launches the user interface to set up a Matter device in the ecosystem.
- The request takes `MatterAddDeviceRequest.Topology`, which contains an ecosystem name and homes, plus optional Matter setup payload and device criteria. The iOS 16.1 `MatterAddDeviceExtensionRequestHandler` subclass supplies room data, validates device credentials, selects Wi-Fi or Thread association, and implements the device commissioning/configuration hooks. The handler is an Objective-C class, but its request hooks in the installed interface are Swift `async` methods that use Swift values; the class annotation alone is not a C/Rust-callable request API.
- The installed `objc2` 0.6.5 generated-framework catalog at `src/topics/about_generated/list_unsupported.md` classifies `MatterSupport` as Swift-only. No generated MatterSupport binding or MatterSupport source was found in the local Cargo registry or repository Rust/C/Objective-C sources.
- The Apple setup guide requires the app to describe Matter discovery services in `NSBonjourServices` (`_matter._tcp`, `_matterc._udp`, and `_matterd._udp`) and register its handler as the `NSPrincipalClass` for extension point `com.apple.matter.support.extension.device-setup`.

## Entitlement, host, and user-present boundary

- Apple documents `com.apple.developer.matter.allow-setup-payload` as the entitlement for an app that supplies a Matter setup payload to the request. The SDK `HMAccessorySetupRequest.matterPayload` comment states that this entitlement is required when that payload is non-null. This is conditional; D91 does not claim it is required for every MatterSupport request.
- Apple's setup guide shows an `.accessDenied` error and advises the host to check entitlements and permissions, but the inspected sources do not establish a complete universal entitlement or permission list for all MatterSupport uses. Do not record absent metadata as proof that no host requirement exists.
- The `isSupported` property is the sole narrow, non-prompting candidate found. Its static getter can be called without constructing a topology, request, or extension handler; this is inferred from the published signature. It answers only whether this request API is supported on the device. It does not establish an ecosystem, permission, entitlement, home access, accessory presence, Thread network state, or successful future setup. Apple does not state the complete entitlement contract for this getter in the inspected documentation. The property is Swift-only in the inspected SDK and has no public Objective-C or C declaration.
- `perform()` starts system UI. A usable setup integration also needs the app's ecosystem topology and the registered extension handler. The extension callbacks may return ecosystem-specific rooms, validate credentials, choose network association, commission the accessory, and configure it; these are not a general app status service.
- Do not infer MatterSupport from an OS version, framework presence, CoreBluetooth authorization, a BLE scan, or a low-level Matter controller. None reproduces this system-mediated ecosystem setup flow.

## Matter and CoreBluetooth are separate surfaces

- `MatterSupport` coordinates a system-mediated add-device flow into an app's ecosystem.
- `Matter.framework` has a distinct Objective-C API, including `MTRDeviceController`. The inspected `MTRDeviceController.h` declares the class from iOS 16.1 and `setupCommissioningSessionWithPayload:newNodeID:error:` from iOS 16.2. These APIs establish a lower-level Matter commissioning path, not a MatterSupport request, an ecosystem membership result, or general device support.
- `CoreBluetooth` exposes separate BLE discovery and connection APIs. `CBCentralManager.scanForPeripheralsWithServices:options:` scans for peripherals advertising Bluetooth services; that operation alone does not establish Matter protocol support or commission an accessory.
- The APIs may participate in a larger accessory flow, but they have different contracts and lifecycle. Do not combine them into one generic MatterSupport Boolean.

## Feasibility result and next evidence

Do not add a portable contract or label this as general MatterSupport readiness. A narrowly named `MatterAddDeviceRequest` API-support query is semantically precise for an app that offers the setup flow, even before it creates a request. It is not independently useful as a generic app capability, does not prove the ecosystem or setup path will work, and is not Rust-callable through the current generated-binding catalog. Reconsider a backend only after a supported Rust/Swift interop path is accepted; do not replace or imply the user-facing add-device flow.

Before a later implementation, require:

1. Product scope for an app-owned Matter ecosystem and a defined home/topology source.
2. An accepted Swift interoperability path or a generated Rust binding for the Swift request/value and async APIs; do not guess Swift ABI symbols.
3. A host app extension target whose principal class inherits `MatterAddDeviceExtensionRequestHandler`, plus its `Info.plist` extension-point metadata and `NSBonjourServices` values.
4. A precise entitlement and permission audit for the selected flow. Add `com.apple.developer.matter.allow-setup-payload` only when the host supplies the setup payload programmatically; do not imply this entitlement grants general Matter use.
5. Device validation of the system UI, cancellation, access-denied, unsupported, commissioning, and extension callback lifecycle. A compile or symbol check cannot prove those behaviors.

## Deferred work

- No portable `framework-home` contract, iOS MatterSupport backend, Rust binding, Swift bridge, extension target, host plist change, entitlement, commissioning operation, Bluetooth discovery change, or canonical capability manifest update.
- No tests, builds, link probes, entitlement request, user prompt, Matter setup flow, BLE scan, or runtime probe.
- No Cargo/workspace/lockfile, CI, aggregate plan, global docs index, or matrix edit.

## Apple and binding references

- [MatterSupport framework](https://developer.apple.com/documentation/mattersupport)
- [MatterAddDeviceRequest](https://developer.apple.com/documentation/mattersupport/matteradddevicerequest)
- [MatterAddDeviceRequest.isSupported](https://developer.apple.com/documentation/mattersupport/matteradddevicerequest/issupported)
- [MatterAddDeviceRequest.perform()](https://developer.apple.com/documentation/mattersupport/matteradddevicerequest/perform())
- [MatterAddDeviceExtensionRequestHandler](https://developer.apple.com/documentation/mattersupport/matteradddeviceextensionrequesthandler)
- [Adding Matter support to your ecosystem](https://developer.apple.com/documentation/mattersupport/adding-matter-support-to-your-ecosystem)
- [Matter Allow Setup Payload entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.matter.allow-setup-payload)
- [MTRDeviceController](https://developer.apple.com/documentation/matter/mtrdevicecontroller)
- [Onboarding a Matter device](https://developer.apple.com/documentation/matter/onboarding-a-matter-device)
- [CBCentralManager](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager)
- [objc2 generated framework catalog](https://github.com/madsmtm/objc2/blob/main/crates/objc2/src/topics/about_generated/list_unsupported.md)

## Evidence and checks

- Read-only SDK inspection: `xcode-select -p` returned `/Applications/Xcode.app/Contents/Developer`; `xcodebuild -version` returned Xcode 26.6, build `17F113`.
- Read-only local source checks searched `MatterSupport`, `MatterAddDeviceRequest`, and `MatterAddDeviceExtensionRequestHandler` across repository Rust/C/Objective-C sources and local Cargo registry sources. The only binding-catalog result was `MatterSupport | Swift-only`; no local generated MatterSupport Rust binding was found.
- Inspected SDK files: `iPhoneOS.sdk/System/Library/Frameworks/MatterSupport.framework/Headers/MatterSupport.h`, `Modules/module.modulemap`, and `Modules/MatterSupport.swiftmodule/arm64e-apple-ios.swiftinterface`; plus `Matter.framework/Headers/MTRDeviceController.h` and `HomeKit.framework/Headers/HMAccessorySetupRequest.h`.
- No tests, builds, link probes, or runtime probes ran.

## B233 implementation: request API support only

B233 adds `ios-matter-support-status::matter_add_device_request_is_supported()` as a narrow
Rust-callable snapshot of `MatterAddDeviceRequest.isSupported`. The crate uses one
compiler-derived C `swiftcall` thunk with an exact weak import; it has no Swift source, Objective-C
selector, generated binding, or new external Rust dependency. The getter is a direct Swift `Bool`
static property, so the call has no Swift object ownership or async ABI

Apple defines this value only as whether MatterSupport supports use of `MatterAddDeviceRequest` on
the current device. `Ok(true)` is not generic Matter, Thread, accessory, entitlement, ecosystem,
home, permission, commissioning, or future setup readiness. The call creates no request or
topology, starts no scan, and presents no UI. iOS 16.1 through 16.x without the iOS 17 getter and
non-iOS targets return `MatterSupportError::NativeApiUnavailable`

The compiler oracle confirms the getter call as `swiftcc i1` on arm64 iOS and arm64 Simulator. The
C object carries an undefined weak external import for
`_$s13MatterSupport0A16AddDeviceRequestV11isSupportedSbvgZ`. The package's local toolchain is Xcode
26.6 / iOS 26.5, below the repository's Xcode 27.x baseline; no runtime or device call was made

The B233 focused gate runs package format, host/device/Simulator `cargo check`, strict Clippy,
rustdoc, `docs-check`, and compiler-derived Swift/C ABI checks. It runs no tests, app, probe, setup
flow, or runtime call

Changed paths: `platform/ios/ios-matter-support-status/`, `docs/ios/matter-support-status.md`, and
this focused plan only. B233 changes row 105 to a narrow partial; aggregate status is recorded in `PLAN.md`, `PLAN_CAPABILITIES.md`, and the capability manifest
