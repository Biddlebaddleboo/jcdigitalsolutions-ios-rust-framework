# PLAN_IOS_MPS_STATUS.md — Workstream B68: iOS MPS Preferred-Device Query

## Status

`platform/ios/ios-mps-status` exposes one iOS-only safe Rust query:
`preferred_mps_device_available() -> bool`. It calls
`MPSGetPreferredDevice(MPSDeviceOptions::Default)` and drops the retained result after mapping it to
a Boolean.

## SDK and binding evidence

- Xcode 26.6 / iPhoneOS SDK 26.5 declares `MPSGetPreferredDevice` and
  `MPSDeviceOptionsDefault` available from iOS 12.2.
- `objc2-metal-performance-shaders` 0.3.2 exposes the function under `MPSCore`; its binding returns
  an owned retained optional Metal device.
- Default binding features remain disabled. The focused package enables only `MPSCore`.
- Device and Simulator probes use deployment floors 12.2 and 14.0. The Simulator value is a link
  setting, not the API availability floor.

Primary references:

- [Apple MPS functions](https://developer.apple.com/documentation/metalperformanceshaders/metalperformanceshaders-functions)
- [Apple `MPSGetPreferredDevice`](https://developer.apple.com/documentation/metalperformanceshaders/mpsgetpreferreddevice%28_%3A%29)
- [`objc2-metal-performance-shaders` 0.3.2 binding](https://docs.rs/objc2-metal-performance-shaders/0.3.2/objc2_metal_performance_shaders/fn.MPSGetPreferredDevice.html)

## Scope and limits

- The query reports whether MPS returns a preferred device with default options at call time.
- It does not submit GPU work or establish support for a specific operation, model, shader, or
  workload.
- It returns no native object, device identity, or handle.
- Runtime cost and thread-affinity requirements are not claimed.
- No permission, Info.plist key, entitlement, or portable contract is added.
- Probe compilation/linkage is not runtime validation; probes are build-only.

## Validation

Run `sh platform/ios/ios-mps-status/check.sh` and
`sh platform/ios/ios-mps-status/check-link-imports.sh`. The first gate checks format, host/device/
Simulator compilation and strict Clippy, rustdoc, `MPSCore` feature isolation, and docs. The second
builds Release device/Simulator probes and audits imports, symbols, and deployment metadata; it does
not execute them.
