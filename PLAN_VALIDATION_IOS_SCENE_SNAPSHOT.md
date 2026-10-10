# PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md — B81/B454 package gate

## Gate

`sh platform/ios/ios-app-scenes/check.sh`

The gate runs five deterministic host regressions, checks only the `ios-app-scenes` library for arm64 iOS and arm64 iOS Simulator, then runs strict Clippy and rustdoc. It builds and inspects one Release link/import probe for each target but does not execute either probe or perform a UIKit action

## Locked Cargo edges

Root-owned `Cargo.lock` now contains the `ios-app-scenes` package and these edges:

- `ios-app-scenes -> ios-runtime`
- `ios-app-scenes -> block2 0.6.2`
- `ios-app-scenes -> objc2 0.6.2`
- `ios-app-scenes -> objc2-foundation 0.3.2`
- `ios-app-scenes -> objc2-ui-kit 0.3.2`

No registry package beyond the current workspace dependency set is required. The device link probe uses iOS 13.0 minos; Rust's arm64 Simulator target emits minos 14.0 in this toolchain even when `IPHONEOS_DEPLOYMENT_TARGET=13.0`, so the Simulator probe uses 14.0. This probe choice does not raise the iOS 13.0 public API floor

## Results

On Xcode 26.6 build `17F113`, iOS device SDK 26.5, iOS Simulator SDK 26.5, and Rust 1.94.1, `sh platform/ios/ios-app-scenes/check.sh` passed

- Five host regressions passed for exact notification mapping, queue overflow/count, UTF-16 identity, weak-state teardown, and panic containment
- Registered `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-app-scenes` passed 41 checks with 0 failures, 0 blocked checks, and 2 runtime checks skipped
- Locked `cargo check` passed for `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Strict locked Clippy passed for both targets
- Locked rustdoc passed and generated `target/aarch64-apple-ios/doc/ios_app_scenes/index.html`
- Device and Simulator Release link/import probes built and were inspected, not executed
- Both probe import sets are `Foundation.framework`, `UIKit.framework`, `libSystem.B.dylib`, and `libobjc.A.dylib`
- `nm -u` found `_objc_msgSend`; the probe string table contains `UIApplication`, `connectedScenes`, and `activationState`; no Swift runtime import was found
- `vtool -show-build` reports device minos 13.0 and Simulator minos 14.0, both with SDK 26.5. Simulator minos is the Rust target default in this toolchain and does not raise the API floor
- The link probe references all six public scene notification symbols, the session persistent-identifier getter, UTF-16 copying, observer registration/removal, and no Swift runtime import
- Rust formatting, shell syntax, JSON parse, focused whitespace/conflict, and focused Markdown path checks passed

No notification delivery, probe execution, UI action, app launch, sign, or device runtime check occurred
