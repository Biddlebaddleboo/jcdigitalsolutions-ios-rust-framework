# PLAN_VALIDATION_IOS_BACKGROUND_EXECUTION.md — Workstream G22: UIKit Background-Execution Gates

## Shared-tooling execution (current)

R1/R2 tooling is complete and installed on PATH; consult `docs/SHARED_TOOLING.md`. This capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`; do not claim an `ios-rust-validate --capability ios-background-execution` PASS. Retain the existing focused shell/Cargo checks below, especially UIKit background lifecycle and no_std/feature tests. When implementing new acceptance coverage, add schema-v1 declarative rules and, only if necessary, bounded Python adapters; replace repeated formatting/lint orchestration **only after** matching the original positive and negative gates. Existing recorded commands remain requirements/evidence as labeled. Never inspect or change the historical shared engine source; report engine defects in a sanitized `BUG_REPORT_*.md`.

## Status

G22 implements and runs D23/B28 no-std, target compile/link, lint, docs, format, source, and diff gates. It does not prove a live UIKit task.

## Objective

Define exact package and generated-binding checks for the portable lease and UIKit adapter.

## Dependencies

- D23 `PLAN_CAPABILITIES_BACKGROUND_EXECUTION.md`
- B28 `PLAN_IOS_BACKGROUND_EXECUTION.md`
- Rust 1.94.1
- Xcode 26.6 build 17F113, iPhoneOS and iPhoneSimulator SDK 26.5

## Required checks

- `cargo +1.94.1 check --locked -p framework-background-execution --no-default-features`
- `cargo +1.94.1 clippy --locked -p framework-background-execution --all-targets -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-background-execution --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-background-execution --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-background-execution --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-background-execution --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 fmt --manifest-path crates/framework-background-execution/Cargo.toml -- --check`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-execution/Cargo.toml -- --check`
- `cargo +1.94.1 doc --locked -p framework-background-execution -p ios-background-execution --no-deps --target aarch64-apple-ios`
- `sh platform/ios/ios-background-execution/check-link-imports.sh`
- `git diff --check`

After root integrates shared documentation paths and lockfile, root may also run `cargo +1.94.1 --locked xtask docs-check` and `cargo +1.94.1 --locked xtask zero-swift-source`.

The package-local link script builds device and Simulator probes without running them. It checks direct UIKit/system imports, selected selectors, no Swift/Python and no unrelated framework imports, and the recorded deployment metadata. The iOS 4.0 API floor comes from the SDK header; local rustc emits a device deployment minimum of iOS 10.0, and the Simulator probe targets iOS 14.0.

## Runtime boundary

No app launch, app-extension behavior, task acceptance duration, expiry event, work completion, physical-device run, or Simulator run is part of these checks. Compile/link evidence does not prove UIKit runtime behavior. The local Xcode 26.6 version is below the repository plan's Xcode 27.x baseline.

## Completed local results

Run in the isolated D23/B28 worktree on 2026-10-08 with Rust 1.94.1, Xcode 26.6 build 17F113, and iPhoneOS/iPhoneSimulator SDK 26.5:

- Portable no-default-features check, portable strict Clippy, both iOS target checks, and strict Clippy for both iOS targets passed with `--locked`
- Both package-scoped rustfmt checks and the two-crate rustdoc build passed
- `sh platform/ios/ios-background-execution/check-link-imports.sh` passed for device and Simulator
- `cargo +1.94.1 --locked xtask zero-swift-source` passed
- `cargo +1.94.1 --locked xtask docs-check` passed
- `git diff --cached --check` passed for the scoped files
- No runtime launch or callback test was run

## Handoff

Record exact commands and results, Rust/Xcode/SDK versions, direct imports, API floor, changed paths, and each failed or skipped gate.
