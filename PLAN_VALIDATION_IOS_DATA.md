# PLAN_VALIDATION_IOS_DATA.md — Workstream G13: iOS Data Gates

## Status

CI wiring, the B19 `ios-data` package, and its focused link/import script are present. On
2026-10-08, the device/Simulator checks, strict Clippy gates, and import script passed locally
on Rust 1.94.1 with Xcode 26.6 build 17F113 and SDK 26.5. The linked probes import exactly
CoreFoundation.framework and `/usr/lib/libSystem.B.dylib`; they were not executed. No passing CI
workflow run or live `CFData` behavior is claimed

## Objective

Add macOS CI and local evidence for the D13 / B19 raw-byte `CFData` bridge on iOS device and
Simulator targets

## Dependencies

- D13 `framework-data` is integrated
- B19 `ios-data` is integrated; see `PLAN_IOS_DATA.md`
- G1 validation tooling and CI are integrated

## Write scope

- `PLAN_VALIDATION_IOS_DATA.md`
- `.github/workflows/ci.yml`
- concise G13 evidence in `PLAN_VALIDATION.md` and `docs/VALIDATION.md`

Do not edit B19 APIs or platform source, Cargo manifests/lockfile, the shared capability matrix,
or other target gates. Root owns capability counts and shared indexes

## Required gates

- On macOS, install `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run locked device and Simulator `cargo check -p ios-data` and strict all-target Clippy with
  `-D warnings` for both targets
- Run `sh platform/ios/ios-data/check-link-imports.sh`; verify exact CoreFoundation and system
  imports and reject Swift/Python runtime, Objective-C class/meta-class, Security, and unrelated
  framework imports
- Retain host formatting, workspace Clippy, `cargo xtask docs-check`, and
  `cargo xtask zero-swift-source` gates
- Keep link probes build-only; do not add runtime, endpoint, file I/O, Keychain, or performance
  claims to CI

## Validation and handoff

- These checks passed after B19 integration and Cargo.lock/workspace reconciliation on Xcode
  26.6 build 17F113 / SDK 26.5:
  `cargo +1.94.1 check --locked -p ios-data --target aarch64-apple-ios`,
  `cargo +1.94.1 check --locked -p ios-data --target aarch64-apple-ios-sim`,
  `cargo +1.94.1 clippy --locked -p ios-data --all-targets --target aarch64-apple-ios -- -D warnings`,
  `cargo +1.94.1 clippy --locked -p ios-data --all-targets --target aarch64-apple-ios-sim -- -D warnings`,
  and `sh platform/ios/ios-data/check-link-imports.sh`
- The arm64 device and Simulator probe artifacts are
  `target/aarch64-apple-ios/release/examples/ios_data_link_import_probe` and
  `target/aarch64-apple-ios-sim/release/examples/ios_data_link_import_probe`. `otool -L` reports
  exactly CoreFoundation.framework and `/usr/lib/libSystem.B.dylib` for both. `vtool -show-build`
  reports device `LC_VERSION_MIN_IPHONEOS` 10.0 and Simulator `IOSSIMULATOR` minimum 14.0; both
  use SDK 26.5. The script's forbidden-symbol scan passed; `nm -u` includes the expected
  `_CFDataCreate`, `_CFDataGetBytes`, `_CFDataGetLength`, and `_CFRelease` imports alongside
  C/runtime symbols
- The available Xcode 26.6 toolchain is below the planned Xcode 27.x baseline. The probe
  executables were not run; compile/link/import checks do not establish live app use, `NSData`
  runtime behavior, parity, allocation behavior under memory pressure, or performance. No
  passing CI workflow run is recorded
- Report compile/link/import evidence only. It does not establish live app use, `NSData` runtime
  behavior, parity, allocation behavior under memory pressure, or performance
- Run `cargo xtask docs-check`, `cargo xtask zero-swift-source`, workflow YAML parsing, and
  `git diff --check`; do not edit the shared capability totals
