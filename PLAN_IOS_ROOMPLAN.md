# B61 — iOS RoomPlan support query

## Status

The bounded iOS query is complete in this isolated worktree. D56 owns the portable value. D55/B60
is reserved for StoreKit; B61 is for RoomPlan row 096. Parent integration remains separate.

## Platform contract

- iOS minimum: 16.0, the RoomPlan framework/property introduction floor.
- Result: `Some(RoomPlanDeviceSupport)` on iOS; `None` on non-iOS host targets.
- The iOS result is the framework's device-support predicate, documented as true when the device
  contains a LiDAR Scanner.
- No session initialization/run/stop, camera or LiDAR frame access, prompt, permission request,
  UI presentation, or scan result is part of this API.
- No Swift source is checked in or shipped. The tiny temporary Swift compiler-oracle fixture is
  only build evidence. A C `swiftcall` thunk is checked against that oracle for arm64 device and
  arm64 Simulator.

## Validation gates

`sh platform/ios/ios-roomplan/check.sh` runs portable/host/device/Simulator Rust checks, strict
Clippy, rustdoc, C/Swift ABI comparison, and Release link/import probes at iOS 16.0. The Release
probes are inspected but never executed. Expected direct imports are `RoomPlan.framework` and
`libSystem.B.dylib`; no Swift or Objective-C runtime import is expected.

## Evidence

`sh platform/ios/ios-roomplan/check.sh` passed on Xcode 26.6 build 17F113, Swift 6.3.3, Apple Clang
21.0.0, and iOS SDK 26.5. It passed portable `no_std`, host, iOS device, and arm64 Simulator Rust
checks; strict Clippy; rustdoc; shell parse; the Swift compiler oracle; and both Release link/import
probes. The oracle matched the getter's Clang `swift_context` parameter to Swift IR's `swiftself`.
Exact direct probe imports on device and Simulator are
`/System/Library/Frameworks/RoomPlan.framework/RoomPlan` and `/usr/lib/libSystem.B.dylib`. Both
probe binaries declare `minos 16.0` and import
`_$s8RoomPlan0A14CaptureSessionCMa` plus
`_$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ`. Neither probe was run; no physical-device
result, permission flow, or room scan is claimed. The SDK Swift interface and `RoomPlan.tbd` confirm
the iOS 16.0 class availability and both exported symbols; Objective-C headers contain no
declaration for this class/property. The repository baseline requires Xcode 27.x, so parent CI must
repeat this gate on that toolchain.
