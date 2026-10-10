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
executed. Xcode 27 static gate: see note below; no live device query or scan result

## Hosted Xcode 27 static-gate evidence (2026-10-10)

Run: [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431); Xcode 27 lane; source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`

Toolchain: Xcode 27.0 build 27A266a; iPhoneOS and iPhoneSimulator SDK 27.0; Rust 1.94.1

Pass step: `Check RoomPlan device-support status`; command `sh platform/ios/ios-roomplan/check.sh`

Coverage: portable, host, device, and Simulator check/strict Clippy/rustdoc; both compiler-oracle checks; both Release link/import checks; formatter; `git diff --check`

Gate paths: `crates/framework-roomplan/`, `platform/ios/ios-roomplan/`, workspace manifest/lockfile, and `xtask/`; no diff from run SHA to `origin/main` `7bd07d8`. Workflow delta adds only Core NFC link/import audit; G55 step definition unchanged

Limit: static support-query, ABI-oracle, and link/import proof only; no probe run; no live device query, permission flow, sensor access, or scan result
