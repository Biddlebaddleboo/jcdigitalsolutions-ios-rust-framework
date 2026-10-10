# PLAN_VALIDATION_IOS_TRACKING_AUTHORIZATION.md — Workstream G34: ATT Status Gates

## Objective

Add package-local repeatable validation gates for the D35/B40 status-only App Tracking Transparency slice. Do not add shared CI.

## Dependencies

- D35 `framework-auth`
- B40 `ios-auth`
- Workspace lockfile reconciliation at integration time

## Write scope

- `platform/ios/ios-auth/check.sh`
- this validation plan

Do not edit shared CI, root Cargo configuration, capability manifests, docs indexes, aggregate plans, or `tools/xtask`.

## Required gates

- `cargo fmt --package framework-auth --package ios-auth -- --check`
- `cargo test --locked -p framework-auth`
- `cargo check --locked -p framework-auth --no-default-features`
- strict Clippy for `framework-auth`
- `cargo check --locked -p ios-auth` and strict Clippy for `aarch64-apple-ios`
- `cargo check --locked -p ios-auth` and strict Clippy for `aarch64-apple-ios-sim`
- `git diff --check`

These are compile, lint, and portable unit checks only. They do not read a live system status, display an authorization prompt, record a user choice, or execute any tracking operation.

## Hosted CI requalification

[GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The `Rust checks (macos-15)` and `Rust checks (xcode-27)` jobs passed `Link and audit iOS authentication imports` and `Check App Tracking Transparency status contract and iOS backend`. The latter runs `sh platform/ios/ios-auth/check.sh`; its log shows both framework-auth tests passing and the script's format, no-default-feature, strict Clippy, device/Simulator check, and diff-check gates completing successfully. The link/import step runs `sh platform/ios/ios-auth/check-link-imports.sh`; it built Release link probes for `aarch64-apple-ios` and `aarch64-apple-ios-sim` and passed its expected-import and symbol-exclusion audits. The expected linked libraries are `AppTrackingTransparency`, `Foundation`, `LocalAuthentication`, `libSystem.B.dylib`, and `libobjc.A.dylib`; the script rejects Swift runtime and Keychain symbol imports. Xcode 27 recorded build `27A266a`, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. The auth package/workflow and shared Cargo inputs for these gates are unchanged through current base `4fd6482f11ceaa9d2e1b8971fbe6b63eeb80d5c5`. This evidence does not query live tracking status, display a prompt, store a user choice, or execute tracking behavior.
