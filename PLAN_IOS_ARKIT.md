# B54 — ARKit world-tracking support snapshot

## Objective

Expose only `ARWorldTrackingConfiguration.isSupported` as an iOS query that returns a portable,
owned `WorldTrackingSupport` result. Do not create or run an AR session.

## Dependencies and API evidence

- D49 defines `framework-maps::WorldTrackingSupport`.
- Apple documents `ARConfiguration.isSupported` as a class property that reports support for the
  concrete session configuration class. `ARWorldTrackingConfiguration` inherits it.
- Xcode 26.6 build 17F113 / iPhoneOS SDK 26.5 headers mark both classes available from iOS 11.0.
- `objc2-ar-kit` 0.3.2 exposes typed `ARConfiguration.isSupported` and
  `ARWorldTrackingConfiguration` under `ARConfiguration` + `objc2` features. The Rust generated
  method is attached to the base class, so the backend sends the selector to the typed subclass
  class receiver to query the exact concrete configuration.
- Apple documents the camera prompt for first AR-session run or other camera use, not this
  pre-session support property. This slice makes no authorization or active-tracking claim.

## Write scope

- `platform/ios/ios-maps/**`
- `docs/ios/arkit.md`
- this plan and the D49 plan
- row 094 only in `docs/capabilities/capability-status.json` plus summary counters
- one macOS CI package step and the portable-crate list in `tools/xtask/src/main.rs`

Do not edit shared root plans, other capability rows, `docs/VALIDATION.md`, unrelated backend
crates, `examples/ios-minimal`, Swift files, or tests. Do not change root Cargo manifests; existing
workspace globs discover both packages. Cargo.lock in this isolated branch is local integration
evidence only and must be reconciled by the orchestrator.

## API and bounds

- `ios_maps::world_tracking_support() -> Option<framework_maps::WorldTrackingSupport>`.
- iOS API floor: 11.0, guarded with `objc2::available!(ios = 11.0, ..)`; non-iOS returns `None`.
- Call only the `isSupported` class property with `ARWorldTrackingConfiguration` as receiver.
- Linked frameworks: `ARKit` and `Foundation`; typed binding `objc2-ar-kit` 0.3.2, defaults disabled, features exactly
  `ARConfiguration` and `objc2`.
- No session, camera capture, frame read, permission request, Info.plist usage string, entitlement,
  native handle, tracking claim, or Swift ABI.
- If an application later starts AR, camera consent and `NSCameraUsageDescription` are still its
  responsibility.

## Gates

- `sh platform/ios/ios-maps/check.sh`: formatting, portable no-std/host checks, strict Clippy,
  device/simulator checks and strict Clippy, docs, and the feature-surface gate.
- `sh platform/ios/ios-maps/check-link-imports.sh`: build-only device and simulator Release links;
  assert the exact direct import set `ARKit`, `Foundation`, `libSystem.B.dylib`, `libobjc.A.dylib`, the class name
  and selector, target `minos`, and absence of unrelated camera/session/frame/UI/Swift imports.
- CI runs the package gate on macOS. No link artifact is executed.

## Status and evidence

Status: bounded status query implemented; row 094 remains partial ARKit support.

Apple primary docs:
[isSupported](https://developer.apple.com/documentation/arkit/arconfiguration/issupported),
[ARWorldTrackingConfiguration](https://developer.apple.com/documentation/arkit/arworldtrackingconfiguration),
and [device support and user permission](https://developer.apple.com/documentation/arkit/verifying-device-support-and-user-permission).
The pinned generated binding source is
[objc2-ar-kit 0.3.2 `ARConfiguration.rs`](https://docs.rs/objc2-ar-kit/0.3.2/x86_64-apple-ios/src/objc2_ar_kit/generated/ARConfiguration.rs.html).

Local SDK: Xcode 26.6 build 17F113, iPhoneOS SDK 26.5. Device and Simulator Release probes linked
and their actual import set matched ARKit, Foundation, libSystem, and libobjc; minimum-OS and
symbol/string gates passed. Neither probe was executed.

Passed commands:

- `cargo check -p ios-maps --lib --target aarch64-apple-ios` (first lock refresh and typed API compile).
- `cargo fmt --manifest-path crates/framework-maps/Cargo.toml -- --check`.
- `cargo fmt --manifest-path platform/ios/ios-maps/Cargo.toml -- --check`.
- `cargo check --locked --no-default-features -p framework-maps`.
- `cargo clippy --locked --lib --no-default-features -p framework-maps -- -D warnings`.
- `cargo doc --locked --no-deps -p framework-maps`.
- `cargo check --locked --lib -p ios-maps` and `cargo clippy --locked --lib -p ios-maps -- -D warnings` (host fallback).
- `cargo check --locked --lib -p ios-maps --target aarch64-apple-ios` and matching strict Clippy.
- `cargo check --locked --lib -p ios-maps --target aarch64-apple-ios-sim` and matching strict Clippy.
- `cargo doc --locked --no-deps -p ios-maps`.
- `cargo tree --locked -p ios-maps --target aarch64-apple-ios -e features`: only `ARConfiguration` and `objc2` are enabled for `objc2-ar-kit` (with its required `bitflags`); no default, `ARSession`, `ARFrame`, `ARCamera`, `ARKitUI`, `ARKitCore`, or `ARKitFoundation` feature is enabled.
- `cargo xtask no-std-check` and `cargo xtask docs-check`.
- `cargo fmt --all -- --check`.
- Ruby YAML parse of `.github/workflows/ci.yml`; `sh -n` for both package shell gates.
- Manifest summary/row validation and `git diff --check`.

`sh platform/ios/ios-maps/check.sh` passed after correcting its allowlist to include the observed
Foundation import. No tests, runtime queries, permission prompts, simulator executions, or device
tracking were run.
