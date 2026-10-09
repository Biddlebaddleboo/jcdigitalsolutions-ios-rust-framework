# G55 — RoomPlan support-status gate

## Scope

Gate D56/B61's `RoomCaptureSession.isSupported` query for row 096. The API reports device support
only; it does not start a scan or access sensor data.

## Commands

- `sh platform/ios/ios-roomplan/check.sh`
- `sh platform/ios/ios-roomplan/check-swiftcall.sh`
- `sh platform/ios/ios-roomplan/check-link-imports.sh`

## Evidence boundary

The package gate covers portable `no_std`, host/device/arm64 Simulator compilation, strict Clippy,
rustdoc, compiler-oracle ABI matching, and exact Release link/import inspection. Device and Simulator
probes are built and inspected, never executed. The temporary Swift oracle stays outside the
checkout; the package and shipping build contain no Swift source. No live device query, permission
flow, camera/LiDAR frame access, or scan result is claimed. No tests are added or run by this gate.

## Status

Passed in both the isolated worktree and integrated checkout on Rust 1.94.1, Xcode 26.6 build
17F113, Swift 6.3.3, Apple Clang 21.0.0, and iOS SDK 26.5. The oracle matched the
`swift_context`/`swiftself` getter argument on
device and arm64 Simulator. Both Release probes import only RoomPlan.framework and
`libSystem.B.dylib`, declare `minos 16.0`, and import the expected public symbols. Neither probe was
executed. The repository's Xcode 27.x baseline remains for CI to verify.
