# iOS Metal default-device presence

**ios-metal** implements the narrow `framework-metal` presence snapshot with the public Metal function `MTLCreateSystemDefaultDevice`. The installed iPhoneOS 26.5 SDK marks the function nullable and retained, available from iOS 8.0. This is the API floor, not a minimum Xcode baseline claim. Apple documents the function as selecting the system-default Metal device; iOS has one GPU. Apple also documents that Simulator returns a device instance connected to Simulator, so a positive Simulator snapshot is not evidence about the physical device's GPU or performance.

```rust
use framework_metal::MetalDeviceQuery;
use ios_metal::CoreMetalDeviceBackend;

let query = MetalDeviceQuery::new(CoreMetalDeviceBackend);
let presence = query.snapshot();
```

Backend construction is inert. On each `snapshot`, the typed `objc2-metal` 0.3.2 binding returns `Option<Retained<ProtocolObject<dyn MTLDevice>>>`. The adapter checks only whether the value is `Some` and immediately drops the retained object. It exposes no Apple type or native handle and creates no command queue, buffer, shader, encoder, texture, MetalKit view, or GPU work. Repeated queries may repeat native object acquisition; this slice makes no cost or performance claim.

## Framework and app requirements

The iOS backend uses `objc2-metal` 0.3.2 with default features disabled and only `MTLDevice`. That feature's generated dependency closure includes Metal device/resource/library declarations but does not enable command-queue, command-buffer, encoder, or MetalKit features. The binding crate's top-level documentation says users of `MTLCreateSystemDefaultDevice` must link CoreGraphics; the backend satisfies that with `objc2-core-graphics` 0.3.2, default features disabled and only `CGBase`. `objc2-core-graphics` also carries its declared CoreFoundation dependency. The exact device and Simulator Release probe imports are Metal.framework, Foundation.framework, `libobjc.A.dylib`, and `libSystem.B.dylib`; CoreGraphics and CoreFoundation do not remain as dynamic load commands in this function-only probe.

No Metal permission prompt, user privacy key, or Metal-specific entitlement is documented for this device-factory query. App signing and target integration remain the application's responsibility. The query does not prove the app can create every Metal resource or submit work.

The API is available on iOS from 8.0. On macOS, Apple notes that obtaining the default device can cause automatic graphics switching; this backend is iOS-only, where Apple documents a single GPU. No performance, physical GPU identity, feature set, compute, rendering, MetalKit, or Simulator/device parity claim is made.

See [PLAN_IOS_METAL.md](../../PLAN_IOS_METAL.md) and [PLAN_VALIDATION_IOS_METAL.md](../../PLAN_VALIDATION_IOS_METAL.md) for the scoped gates and boundaries.
