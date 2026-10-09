# PLAN_IOS_METAL.md — Workstream B41: iOS Metal Default-Device Presence Query

## Objective

Implement the D36 scalar presence snapshot using the public `MTLCreateSystemDefaultDevice` function. Do not implement MetalKit, rendering, compute, or any GPU work submission.

## API and dependency evidence

- Installed environment: Xcode 26.6 build 17F113, iPhoneOS and iOS Simulator SDK 26.5
- The local `MTLDevice.h` marks `MTLCreateSystemDefaultDevice` nullable, retained, and `API_AVAILABLE(ios(8.0))`; iOS 8.0 is the API floor, not an Xcode baseline claim
- Apple documents `MTLCreateSystemDefaultDevice()` as returning the system-default Metal device. Apple documents iOS devices as having one GPU and documents a simulator device instance for iOS Simulator
- `objc2-metal` 0.3.2 provides the typed safe signature `MTLCreateSystemDefaultDevice() -> Option<Retained<ProtocolObject<dyn MTLDevice>>>`, gated by feature `MTLDevice`
- The `objc2-metal` crate-level docs instruct consumers of this function to link CoreGraphics. Use `objc2-core-graphics` 0.3.2 with default features disabled and `CGBase` only to satisfy that link dependency
- No Metal permission, privacy key, or entitlement is documented for this device factory; do not infer one

## Dependencies

- Foundation A, iOS runtime B, and D36 `framework-metal` are integrated
- Review `PLAN_CAPABILITIES_METAL.md`, `docs/OBJC_INTEROP.md`, `docs/OWNERSHIP.md`, and `docs/UNSAFE.md`
- Apple references: [MTLCreateSystemDefaultDevice](https://developer.apple.com/documentation/metal/mtlcreatesystemdefaultdevice%28%29), [Getting the default GPU](https://developer.apple.com/documentation/metal/getting-the-default-gpu), and [Metal apps in Simulator](https://developer.apple.com/documentation/metal/developing-metal-apps-that-run-in-simulator)

## Write scope

- `platform/ios/ios-metal/**`
- `docs/ios/metal.md`

Do not edit the portable D36 crate, root workspace manifest, capability status manifest, CI, aggregate plans, documentation indexes, Swift, C bindings, or unrelated backends. The lockfile may change only for these workspace packages and the `objc2-metal` / `objc2-core-graphics` dependencies.

## Required implementation

- Use `objc2-metal` 0.3.2 with default features disabled and only feature `MTLDevice`; do not use raw ABI calls
- Satisfy the binding's CoreGraphics link note with `objc2-core-graphics` 0.3.2, default features disabled, feature `CGBase`
- Map the optional retained return to `Present` or `Absent`; release the temporary native device object before returning
- Keep Metal and CoreGraphics types out of the portable API; expose no retained native handle
- Do not create command queues/buffers, shaders, encoders, textures, views, or MetalKit objects; do not submit work or claim performance
- Document that `Present` means only a default device object was returned, and that Simulator's device is not evidence of physical-device hardware or performance

## Validation and handoff

- Run device and simulator `cargo check` and Clippy gates from `PLAN_VALIDATION_IOS_METAL.md`
- Run focused portable tests, format, docs, zero-Swift, release dependency/import, and diff checks
- Document iOS 8.0 API floor, transient native ownership, linked framework set, absence of known permission/Info.plist/entitlement needs for this operation, and live-device limits
- Report exact commands, results, public symbols, limits, and unresolved requirements
