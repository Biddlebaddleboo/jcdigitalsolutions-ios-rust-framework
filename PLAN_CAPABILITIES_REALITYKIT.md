# PLAN_CAPABILITIES_REALITYKIT.md — D80: RealityKit audit for row 095

## Scope

Assess row `095-maps-ar-spatial-realitykit-native-swift-residual-coverage-that-is-realistically-supportable` for a bounded Rust-accessible API in the installed iOS SDK, including Objective-C runtime and public C/Metal header surfaces

This is an audit only. It does not edit source, package manifests, global plans, capability status, indexes, or CI

## Status and recommendation

Keep row 095 X for a Rust-owned RealityKit scene/rendering capability. The local generated-binding catalog labels both `RealityKit` and `RealityFoundation` Swift-only. The SDK's useful scene model and operations are Swift APIs, while its C-style public headers are Metal shader interfaces guarded for Metal compilation

There is a limited Objective-C-interoperable `ARView` class and an `init(frame:)` initializer, available from iOS 13 and isolated to the main actor. That creates a view shell, not a usable Rust RealityKit feature: the `scene` property has Swift type `RealityFoundation.Scene`, and adding content requires `Scene`, `AnchorEntity`, `Entity`, components, and their Swift collection/protocol APIs. The view-only initializer does not expose those operations to Objective-C or Rust. Do not count an empty `UIView` subclass as RealityKit implementation

`RealityView` is a SwiftUI `View` with main-actor, generic closure initializers; on iOS its camera-content initializer starts at iOS 18. A useful Rust path would need a deliberate Swift/SwiftUI interop layer or broader support for RealityKit's Swift scene model. No smaller direct C/Objective-C value or status API was found in this audit

## SDK and binding evidence

Inspection used Xcode 26.6 build 17F113 and iPhoneOS 26.5 SDK

- `RealityKit.framework` has `RealityKit.tbd`, a Swift module interface, and public headers. It has no Objective-C `@interface` declarations or Objective-C module map in its `Headers` directory
- `RealityFoundation.framework` has a Swift module interface and `.tbd`, but no public `Headers` directory
- `RealityKit.h` imports `RealityKitGeometryModifier.h` and `RealityKitSurfaceShader.h`. Those headers and their included material/texture/type headers place their declarations under `#if defined(__METAL_VERSION__)`; they describe Metal shader functions/types, not host-side C or Objective-C calls from Rust
- The inspected RealityKit `.tbd` exports include Swift-mangled symbol names such as `_$s10RealityKit...` and re-export `RealityFoundation`. No stable C function surface for scene creation, entity insertion, or rendering was found
- The cached `objc2` 0.6.5 generated-framework catalog at `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_unsupported.md` labels both `RealityKit` and `RealityFoundation` Swift-only. No local generated Rust binding crate for either framework was found
- `ARView` is explicitly `@objc` in the Swift interface and Apple docs. Its `init(frame:)` is an Objective-C-compatible `UIView` initializer, deprecated on iOS since its iOS 13 introduction in favor of the Swift initializer that takes `cameraMode` and `automaticallyConfigureSession`. `ARView` itself is `@MainActor`; its scene/environment properties and useful scene operations are not Objective-C declarations in the interface
- The local `RealityFoundation` interface marks `Entity`, `Scene`, and `AnchorEntity` as `@MainActor`. Their scene/anchor/hierarchy APIs are Swift classes, protocols, values, and collections, not a C ABI
- Relevant floors differ by feature: `ARView` and core RealityFoundation scene types start at iOS 13; custom materials start at iOS 15; iOS `RealityView` starts at iOS 18. There is no single support floor for row 095 because no facade is implemented

## Required host configuration and runtime limits

- A caller can create an `ARView` view, but this does not itself establish a Rust scene contract, renderable entity, or useful AR experience
- For an ARKit-backed camera session, Apple requires `NSCameraUsageDescription` and user permission. The first attempt to run an AR session prompts for camera access; denial prevents camera-backed AR. If AR is core app functionality, Apple documents the `arkit` key in `UIRequiredDeviceCapabilities`; if it is secondary, check support for the specific `ARConfiguration` at runtime
- ARKit device and feature support depends on the selected configuration and device hardware. Do not make blanket device-compatibility claims from the RealityKit framework link alone
- This audit found no general RealityKit entitlement requirement in the inspected basic `ARView`/RealityKit docs. That does not imply that ARKit camera use, specialized anchors, scene understanding, or other RealityKit-associated services have no host requirements
- `ARView`, `Scene`, and entity operations are main-actor APIs. SwiftUI `RealityView` is also main-actor isolated and requires SwiftUI view/closure composition
- Metal shader headers allow custom shader code used by a `CustomMaterial`; they do not provide the scene/material construction or entity/render pipeline as a CPU-side Rust API. Apple describes the shaders as Metal functions passed into Swift-created RealityKit custom materials
- No runtime probe, camera permission prompt, AR session, scene render, or device capability query was run

## Candidate boundary review

The only plausible narrow Objective-C seam found was `ARView.init(frame:)`. A Rust adapter could at most construct a main-thread UIKit view handle through that initializer. It cannot select the modern Swift `cameraMode` initializer without crossing Swift values, access `scene`, create an `AnchorEntity`, create an `Entity` or mesh, attach components, insert anchors, load a model, or render content through a public C API. That shell is not a complete or independently useful RealityKit capability, so this workstream does not implement it

The Metal shader declarations are not an alternative host API. They execute inside RealityKit's render pipeline as MSL, while Apple documents creation of the shader objects, `CustomMaterial`, and the model entity in Swift. A Rust-only shader artifact would still lack the public host API that attaches it to a scene and renders it

## Acceptance boundary

This audit establishes only:

- row 095 remains X for Rust-owned RealityKit scene and rendering support
- `ARView` has a limited Objective-C-interoperable view initializer from iOS 13, but that initializer alone does not provide scene content or meaningful RealityKit behavior
- RealityKit and RealityFoundation have no generated objc2 crate in the inspected binding catalog and the latter has no public C/Objective-C header surface in the inspected SDK
- the public C-style RealityKit headers are Metal shader interfaces, not host-side Rust APIs
- ARKit-backed camera use has `NSCameraUsageDescription`, consent, and device/configuration support requirements

This workstream does not claim RealityKit parity, AR session startup, camera access, device tracking, entity creation, scene mutation, model loading, material creation, shader compilation, or rendered output

## Apple and local sources

- [ARView](https://developer.apple.com/documentation/realitykit/arview)
- [ARView init(frame:)](https://developer.apple.com/documentation/realitykit/arview/init%28frame%3A%29)
- [RealityView](https://developer.apple.com/documentation/realitykit/realityview)
- [Scene](https://developer.apple.com/documentation/realitykit/scene)
- [Entity](https://developer.apple.com/documentation/realitykit/entity)
- [Verifying Device Support and User Permission](https://developer.apple.com/documentation/arkit/verifying-device-support-and-user-permission)
- [CustomMaterial](https://developer.apple.com/documentation/realitykit/custommaterial)
- [Modifying RealityKit rendering using custom materials](https://developer.apple.com/documentation/realitykit/modifying-realitykit-rendering-using-custom-materials)
- Local RealityKit interface: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/RealityKit.framework/Modules/RealityKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- Local RealityKit headers: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/RealityKit.framework/Headers`
- Local RealityKit exports: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/RealityKit.framework/RealityKit.tbd`
- Local RealityFoundation interface: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/RealityFoundation.framework/Modules/RealityFoundation.swiftmodule/arm64e-apple-ios.swiftinterface`
- Local generated-binding support catalog: `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_unsupported.md`

## Audit record

The audit inspected the RealityKit and RealityFoundation SDK trees, public headers, Swift interfaces, `.tbd` exports, local generated-binding catalog, the row 095 status record, and Apple's primary RealityKit and ARKit documentation

`xcrun --sdk iphoneos --show-sdk-version` reported `26.5`; `xcodebuild -version` reported `Xcode 26.6` and build `17F113`. No code edit, package edit, test, build, link probe, or runtime probe was made
