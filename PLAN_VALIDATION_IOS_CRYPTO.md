# PLAN_VALIDATION_IOS_CRYPTO.md — Workstream G60: CommonCrypto SHA-256 Gates

## Objective

Gate D60/B66's narrow iOS-only `CC_SHA256` wrapper for device and Simulator compile/lint and exact Release link imports.

## Dependencies

- B66 `ios-crypto` and its focused link/import script
- G1 shared macOS CI

## Required gates

- `cargo check --locked -p ios-crypto --target aarch64-apple-ios`
- `cargo check --locked -p ios-crypto --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-crypto --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-crypto --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `sh platform/ios/ios-crypto/check-link-imports.sh`
- Format, rustdoc, docs-index, zero-Swift-source, and diff checks

The link script builds Release probes but does not execute them. It requires only
`libSystem.B.dylib`, checks `_CC_SHA256`, and verifies the device/Simulator deployment metadata.
These gates do not validate a digest result, parity, security review, certification, live device
behavior, or performance.

## Status and evidence

Status: complete in the integrated checkout. Rust 1.94.1 device and Simulator checks, strict
all-target Clippy for both targets, and the Release link/import script passed on Xcode 26.6 build
17F113 / iOS SDK 26.5. Release probes import only `libSystem.B.dylib` and `_CC_SHA256`; `vtool`
reports iOS 10.0 device and iOS 14.0 Simulator link floors. The SDK header separately marks
`CC_SHA256` available from iOS 2.0. The probes were built and inspected, not executed. No tests ran.

The focused commands are wired in `.github/workflows/ci.yml`. No passing CI workflow run is
recorded, and the local Xcode 26.6 host remains below the plan's Xcode 27.x baseline.
