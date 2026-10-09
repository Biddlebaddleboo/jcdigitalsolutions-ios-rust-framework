# PLAN_VALIDATION_C_ABI_SPRITEKIT.md — G66: F11 SpriteKit C ABI gate

## Scope

Gate F11's opt-in `ios-spritekit` C feature over D64/B70's detached `SKNode.position` slice. The C boundary exposes only an opaque uniquely owned `FrameworkIosSpriteKitNode` and create/get/set/destroy functions. The gate checks feature isolation, manifest/header/archive symbols, locked host/device/Simulator compile and strict Clippy, C11/C++17 links, exact framework imports, and deployment metadata.

## Results

`sh bindings/c/check-ios-spritekit.sh` passed on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5 after the offline Cargo lock refresh. Device imports matched CoreFoundation, Foundation, SpriteKit, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imported `libc++.1.dylib`. Probe `minos` was 12.0/device and 14.0/Simulator. B70's iOS 7.0 API floor remains distinct.

The probes were built and inspected, not executed. No test, scene, rendering, hierarchy, animation, physics, or live SpriteKit behavior was exercised. This is FFI compile/link evidence, not parity or performance evidence.
