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

## B194 follow-up: no Rust-only RealityKit scene operation

Rechecked row 095 against the iOS 26.5 SDK and Apple's current RealityKit docs. `ARView.init(frame:)`
is an Objective-C-compatible `UIView` initializer, but the class is `@MainActor`, and its scene
property is the Swift `RealityFoundation.Scene` type. The useful follow-on calls—constructing an
`AnchorEntity`/`Entity`, attaching components, and inserting anchors into a scene—remain Swift
types and APIs. The available initializer can construct a view shell, but it does not provide a
Rust-owned scene, content, or rendering operation.

The installed public RealityKit headers remain shader interfaces guarded by `__METAL_VERSION__`,
not host-side C functions for scene or entity operations. `RealityFoundation` has no public header
directory or module map, and the generated binding inventory marks both `RealityKit` and
`RealityFoundation` Swift-only. A Metal shader alone cannot attach itself to a RealityKit scene;
Apple's custom-material path still constructs material and entity objects through the Swift scene
model. There is no source-backed Objective-C/C slice beyond the unusable view shell.

B194 therefore adds no view wrapper, Swift bridge, package dependency, or source code. Row
`095-maps-ar-spatial-realitykit-native-swift-residual-coverage-that-is-realistically-supportable`
remains `X`. This inspection used Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5; the Xcode 27.x
baseline caveat remains unchanged.

Primary API evidence: [ARView](https://developer.apple.com/documentation/realitykit/arview),
[ARView `init(frame:)`](https://developer.apple.com/documentation/realitykit/arview/init%28frame%3A%29),
[Scene](https://developer.apple.com/documentation/realitykit/scene),
[Entity](https://developer.apple.com/documentation/realitykit/entity),
[RealityView](https://developer.apple.com/documentation/realitykit/realityview),
and [custom RealityKit materials](https://developer.apple.com/documentation/realitykit/modifying-realitykit-rendering-using-custom-materials)

No tests, builds, ARView creation, app launch, camera access, AR session, scene mutation, render,
or device query was performed for B194

## B291 follow-up: one Photogrammetry hardware-support snapshot

The iOS 26.5 `RealityFoundation.swiftinterface` declares `PhotogrammetrySession.isSupported` as a synchronous `static Bool` on the iOS 17.0+ `PhotogrammetrySession` class. The type is not `@MainActor`; its getter is a plain scalar read. Apple's documentation says this value reports whether current hardware supports Object Capture and advises clients to check it before use. This supports one precise point-in-time hardware-support query without `PhotogrammetrySession` construction, image input, reconstruction, capture, or UI.

B291 adds `platform/ios/ios-photogrammetry-status` and `docs/ios/photogrammetry-status.md`. The native C thunk uses the compiler-derived class metadata accessor and `swiftcall` getter, with one `u8` result and no Rust-owned Swift value. The public Rust function returns only the native `isSupported` Boolean or a bridge/target error. The package links `RealityFoundation` and has an iOS 17.0 minimum.

This is a narrow RealityFoundation partial only. `true` does not guarantee that particular images can be reconstructed or that a future session/process call will succeed. It does not add ObjectCaptureSession image capture, PhotogrammetrySession creation or processing, `ObjectCaptureView`, RealityKit scene/entity/render support, or ARView capability. D80/B194's full RealityKit scene/entity/render caveat remains. Root owns aggregate row `095`; no matrix/count edit is made here. Xcode 27.x remains the required repository baseline; local compiler evidence is Xcode 26.6 build `17F113` / iOS SDK 26.5.

Primary evidence: iOS 26.5 `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/RealityFoundation.framework/Modules/RealityFoundation.swiftmodule/arm64e-apple-ios.swiftinterface` and `RealityFoundation.tbd`; [Apple `PhotogrammetrySession.isSupported`](https://developer.apple.com/documentation/realitykit/photogrammetrysession/issupported), [Apple `PhotogrammetrySession`](https://developer.apple.com/documentation/realitykit/photogrammetrysession), and [Apple Object Capture overview](https://developer.apple.com/documentation/realitykit/realitykit-object-capture).

No tests, session creation, image processing, capture, UI, app launch, Simulator run, or device query was performed for B291

## B298 follow-up: Photogrammetry input-limit snapshot

The iOS 26.5 `RealityFoundation.swiftinterface` declares `PhotogrammetrySession.Limits` and `static let limits` at iOS 17.0+, with two synchronous `Swift.Int` getters: `maximumInputImageDimension` and `maximumNumberOfInputImages`. The interface does not mark `Limits` `@frozen`, so the Rust boundary does not assume or copy its layout. Apple documents `limits` as device-specific constant hardware limits; the dimension value is the max allowed input width or height, and the image-count value is the max images or samples usable for reconstruction. Input beyond either limit is ignored with an `.invalidSample` output.

B298 extends `platform/ios/ios-photogrammetry-status` with the Rust-owned `PhotogrammetrySessionLimits` snapshot and `photogrammetry_session_limits()`. It returns the two exact Swift `Int` values as signed `i64` fields. The C `swiftcall` thunk uses the public `Limits` metadata accessor and value-witness table to get dynamic size/alignment, allocates opaque storage, invokes the compiler-derived `sret` getter and both scalar getters, calls the value-witness destroy function, then frees the storage. It does not use guessed field offsets or add general resilient Swift value support.

The scope adds only these two input-limit reads. It does not create a session, read images, start reconstruction, capture camera data, present UI, or guarantee successful reconstruction or quality. B291's separate `isSupported` result remains only a hardware support Boolean. Row `095` remains partial; full RealityKit scene/entity/render support, ARView, and Object Capture flows remain unsupported. Root owns aggregate status and matrix edits.

Compiler-oracle checks match device and Simulator IR for the `Limits` metadata accessor, opaque indirect result, both `i64` getters, metadata-sized/aligned allocation, and value-witness destruction. Static link checks verify the public imports and iOS 17.0 deployment floor; the example is not run. No tests, session creation, image processing, reconstruction, capture, UI, app launch, Simulator run, or device query was performed for B298.

Primary evidence: iOS 26.5 `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/RealityFoundation.framework/Modules/RealityFoundation.swiftmodule/arm64e-apple-ios.swiftinterface` and `RealityFoundation.tbd`; [Apple `PhotogrammetrySession.Limits`](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct), [Apple `maximumInputImageDimension`](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct/maximuminputimagedimension), [Apple `maximumNumberOfInputImages`](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.struct/maximumnumberofinputimages), and [Apple `PhotogrammetrySession.limits`](https://developer.apple.com/documentation/realitykit/photogrammetrysession/limits-swift.type.property).

The Xcode 27.x repository baseline caveat remains; local compiler evidence is Xcode 26.6 build `17F113` / iOS SDK 26.5
