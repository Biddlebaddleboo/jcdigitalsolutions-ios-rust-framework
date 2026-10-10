# PLAN_VALIDATION_IOS_DATA.md — Workstream G13: iOS Data Gates

## Installed shared-tooling boundary

The R1/R2 build and validation engines are already installed, pinned PATH tools; use `docs/SHARED_TOOLING.md` rather than reimplementing their internals. This workstream is **not a registered pilot** in the current `tools/validation/specs/validation-v1.json`, so do not claim a passing `ios-rust-validate --capability` command for it. The existing focused package scripts, CI checks and command/evidence records below remain required until a schema-v1 declarative profile demonstrably reproduces all applicable checks, including CFData bridge exact CoreFoundation/libSystem imports and absence of objc/Swift dependencies. Only remove duplicate commands after proving positive, deliberately failing negative, dependency-selection and fail/skip parity. Optional Python adapters must be narrowly scoped and cannot substitute for a real compiler/link or device gate. Tool engine defects are reported in sanitized `BUG_REPORT_*.md`; ordinary API work never retrieves historical engine source.

## Status

CI wiring, the B19 `ios-data` package, and its focused link/import script are present. On
2026-10-08, the device/Simulator checks, strict Clippy gates, and import script passed locally
on Rust 1.94.1 with Xcode 26.6 build 17F113 and SDK 26.5. The linked probes import exactly
CoreFoundation.framework and `/usr/lib/libSystem.B.dylib`; they were not executed. Hosted run
38075483431 later passed G13's device, Simulator, Clippy, and link/import steps at source SHA
`85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; see below. No live `CFData` behavior is claimed.

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
- The local Xcode 26.6 toolchain is below the planned Xcode 27.x baseline. The probe
  executables were not run; compile/link/import checks do not establish live app use, `NSData`
  runtime behavior, parity, allocation behavior under memory pressure, or performance.
- Report compile/link/import evidence only. It does not establish live app use, `NSData` runtime
  behavior, parity, allocation behavior under memory pressure, or performance
- Run `cargo xtask docs-check`, `cargo xtask zero-swift-source`, workflow YAML parsing, and
  `git diff --check`; do not edit the shared capability totals

## 2026-10-10 current-main recheck

At `790a18db95d8a9b8a8ee5ae331222135f0957d26`, Rust 1.94.1, Xcode 26.6 build 17F113, and iOS
SDK 26.5:

- PASS: `cargo +1.94.1 check --locked -p ios-data --target aarch64-apple-ios`
- PASS: `cargo +1.94.1 check --locked -p ios-data --target aarch64-apple-ios-sim`
- PASS: `cargo +1.94.1 clippy --locked -p ios-data --all-targets --target aarch64-apple-ios -- -D warnings`
- PASS: `cargo +1.94.1 clippy --locked -p ios-data --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- PASS: `sh platform/ios/ios-data/check-link-imports.sh` for device and Simulator. Both probes import exactly CoreFoundation.framework and `/usr/lib/libSystem.B.dylib`; device minos is 10.0 and Simulator minos is 14.0; both use SDK 26.5. The forbidden-symbol scan passes, with expected `_CFDataCreate`, `_CFDataGetBytes`, `_CFDataGetLength`, and `_CFRelease` imports.
- PASS: `cargo +1.94.1 clippy --locked --workspace --all-targets --all-features -- -D warnings`, `cargo +1.94.1 fmt --all -- --check`, `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, Ruby CI YAML parse, and `git diff --check`.

No test, probe execution, app, live `CFData` use, parity, memory-pressure allocation query, or performance measurement ran. These are compile, Clippy, static link/import, format, and docs results only. The hosted Xcode 27.0 CI evidence below is for historical source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`, not this local Xcode 26.6 recheck at `790a18db95d8a9b8a8ee5ae331222135f0957d26`.

## 2026-10-10 hosted Xcode 27 CI evidence

[GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The `Rust checks (ubuntu-24.04)`, `Rust checks (macos-15)`, and `Rust checks (xcode-27)` jobs all concluded `success`. The Xcode 27 job's `Select and record Xcode 27.x` and `Fingerprint Xcode 27 runner and iOS SDKs` steps succeeded and recorded Xcode 27.0 build 27A266a, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0.

- In both macOS jobs, `Check iOS data device target`, `Check iOS data simulator target`, `Clippy iOS data device target`, `Clippy iOS data simulator target`, and `Link and audit iOS data imports` concluded `success`.
- The Ubuntu job concluded `success`; those Apple-targeted G13 steps were skipped there.
- The link/import script built device and Simulator probes, checked the exact `CoreFoundation` and `libSystem.B.dylib` linked-library set, expected CoreFoundation data symbols, forbidden imports, and zero-Swift boundary. The probes were not executed.

This hosted evidence establishes compilation, strict Clippy, and static link/import checks for the recorded source SHA only. It does not establish runtime CFData behavior, live app use, data parity, allocation behavior under memory pressure, or performance, and it does not requalify commits after `85db105`.
