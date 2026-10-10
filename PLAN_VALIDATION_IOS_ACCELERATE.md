# PLAN_VALIDATION_IOS_ACCELERATE.md — Workstream G59: Accelerate Compile and Link Gates

## Shared-tooling execution (current)

R1/R2 tooling is complete and installed on PATH; consult `docs/SHARED_TOOLING.md`. This capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`; do not claim an `ios-rust-validate --capability ios-accelerate` PASS. Retain the existing focused shell/Cargo checks below, especially vDSP public Accelerate imports and equal-length wrapper safety. When implementing new acceptance coverage, add schema-v1 declarative rules and, only if necessary, bounded Python adapters; replace repeated formatting/lint orchestration **only after** matching the original positive and negative gates. Existing recorded commands remain requirements/evidence as labeled. Never inspect or change the historical shared engine source; report engine defects in a sanitized `BUG_REPORT_*.md`.

## Objective

Gate B65's direct C `vDSP_vadd` call and safe equal-length slice wrapper without running a numerical
probe

## Required gates

- `cargo +1.94.1 check --locked -p ios-accelerate --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-accelerate --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-accelerate --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-accelerate --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `sh platform/ios/ios-accelerate/check-link-imports.sh`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-accelerate/Cargo.toml -- --check`
- `cargo +1.94.1 doc --locked -p ios-accelerate --no-deps`
- `cargo +1.94.1 xtask docs-check`
- `cargo +1.94.1 xtask zero-swift-source`
- `git diff --check`

The focused link script builds Release consumers for arm64 iOS device and Simulator. It checks the
direct framework allowlist, `_vDSP_vadd`, absence of unrelated Apple and language runtimes, and
deployment floors with `vtool -show-build`. The probe does not execute `vDSP_vadd`. The inspected
SDK declaration is iOS 4.0; the link floors are iOS 10.0 for device and iOS 14.0 for Simulator

## Non-claims

No tests, numerical result, Apple parity, benchmark, live-device execution, or performance
improvement is claimed

## Integrated result

G59 passed in the root checkout: locked device/Simulator check and strict Clippy, the focused
Release link/import script, rustdoc, formatting, docs-check, zero-Swift-source, diff, and JSON gates
passed. Both probes import only Accelerate and `libSystem.B.dylib`, retain `_vDSP_vadd`, and report
device minos 10.0 and Simulator minos 14.0 (SDK 26.5). The probes were not executed; no tests,
numerical result, Apple parity, live-device behavior, or performance claim is made
