# PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md — Workstream G46: CloudKit Account-Status Gates

## Installed shared-tooling boundary

The R1/R2 build and validation engines are already installed, pinned PATH tools; use `docs/SHARED_TOOLING.md` rather than reimplementing their internals. This workstream is **not a registered pilot** in the current `tools/validation/specs/validation-v1.json`, so do not claim a passing `ios-rust-validate --capability` command for it. The existing focused package scripts, CI checks and command/evidence records below remain required until a schema-v1 declarative profile demonstrably reproduces all applicable checks, including CloudKit entitlement, account-status, framework imports and link-only probes. Only remove duplicate commands after proving positive, deliberately failing negative, dependency-selection and fail/skip parity. Optional Python adapters must be narrowly scoped and cannot substitute for a real compiler/link or device gate. Tool engine defects are reported in sanitized `BUG_REPORT_*.md`; ordinary API work never retrieves historical engine source.

## Objective

Add repeatable portable, device, Simulator, and link/import gates for D47/B52. The gates must not
run a live account query or claim CloudKit data access.

## Required checks

- Check and strict-Clippy `framework-cloud` without default features
- Check and strict-Clippy `ios-cloud` for `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Build, but do not execute, the CloudKit link-check example for device and Simulator
- Audit for CloudKit, Foundation, libSystem, and libobjc imports, with no Swift runtime or unrelated
  capability framework
- Run formatting, docs-index, zero-Swift-source, and diff checks
- Do not add or run tests, launch a simulator, or query a live account

## Status and evidence

The gates are wired in macOS CI through `sh platform/ios/ios-cloud/check-cloudkit.sh`. Local
validation on Rust 1.94.1 / Xcode 26.6 build 17F113 / iPhoneOS and iPhoneSimulator SDK 26.5 passed:

- `cargo +1.94.1 check --locked -p framework-cloud --no-default-features`
- `cargo +1.94.1 clippy --locked -p framework-cloud --all-targets --no-default-features -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-cloud --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-cloud --target aarch64-apple-ios-sim`
- strict all-target Clippy for `ios-cloud` on both targets
- `sh platform/ios/ios-cloud/check-link-imports.sh device`
- `sh platform/ios/ios-cloud/check-link-imports.sh simulator`

Both link probes import CloudKit, Foundation, `/usr/lib/libSystem.B.dylib`, and
`/usr/lib/libobjc.A.dylib`; neither imports a Swift runtime or unrelated capability framework. The
examples were built but not executed. Formatting, docs-index, zero-Swift, and diff checks are
passed through the package gate. No live account query, entitlement validation, CloudKit data
operation, or simulator launch is claimed. The local Xcode 26.6 / SDK 26.5 result is below the
required Xcode 27.x plan baseline; hosted Xcode 27 static-gate evidence is recorded below.

## Acceptance boundary

These gates establish compilation, lint, and bounded link/import evidence only. They do not establish
live account status, signed entitlement configuration, CloudKit database access, account-change
observation, Apple parity, or performance.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check CloudKit account-status contract and iOS gates`, which ran `sh platform/ios/ios-cloud/check-cloudkit.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's portable check/strict Clippy, device/Simulator check/strict Clippy, device/Simulator link/import gates, formatter, docs-index, zero-Swift-source, and diff checks passed; both import-gate logs state that the binary was not executed. The `crates/framework-cloud/`, `platform/ios/ios-cloud/`, workspace manifest/lockfile, `xtask/`, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

This does not establish live account status, signed entitlement configuration, CloudKit database access, account-change observation, Apple parity, or performance.
