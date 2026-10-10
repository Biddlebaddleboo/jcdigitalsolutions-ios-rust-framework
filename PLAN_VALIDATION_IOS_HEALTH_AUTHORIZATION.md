# PLAN_VALIDATION_IOS_HEALTH_AUTHORIZATION.md — Workstream G25: Health Authorization Validation

## Shared-tooling execution (current)

The R1/R2 PATH executables are complete; use `docs/SHARED_TOOLING.md` and schema-v1 specifications rather than new build/validation machinery. This workstream has **no existing registered pilot** in `tools/validation/specs/validation-v1.json`. Retain its recorded local commands and unique HealthKit permission/entitlement and portable tests tests. An executor may declare a new validation profile and optional focused Python adapter; remove or replace existing checks only after equivalent positive, negative, cross-target and failure/skip parity is established. Do not claim an unregistered `ios-rust-validate --capability` run, infer runtime proof from static checks, or retrieve shared engine source. Escalate confirmed engine defects with `BUG_REPORT_*.md`.

## Objective

Validate D26's portable request semantics and B31's public iOS bindings without treating compilation or request-flow completion as HealthKit authorization evidence.

## Scoped checks

- Format the worktree with `cargo +1.94.1 fmt --all`; then verify with `cargo +1.94.1 fmt --all -- --check`.
- Run `cargo +1.94.1 test -p framework-health-authorization`.
- Run `cargo +1.94.1 check -p framework-health-authorization --no-default-features`.
- Run strict Clippy for the portable crate and the iOS crate on device and simulator targets with `-D warnings`.
- Run `cargo +1.94.1 check -p ios-health-authorization --target aarch64-apple-ios` and `cargo +1.94.1 check -p ios-health-authorization --target aarch64-apple-ios-sim`.
- Run `git diff --check`; inspect the public contract for Apple types, `std`, dynamic dispatch, grant-state leakage, and hidden initialization; verify no Swift source was added.

## Evidence limits

Cargo target checks type-check iOS device and Simulator configurations against the local SDK. They do not create or launch a consuming app, verify final app linkage/imports, show HealthKit's authorization UI, validate a provisioning entitlement or purpose-string metadata in a host app, or exercise callback timing and user choices. Simulator sample Health Records documented by Apple are not evidence for real sensor data or every HealthKit type. No permission, query, or sample read/write runtime check is in scope.

## Status and evidence

All scoped checks pass with Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5:

- Five `framework-health-authorization` tests pass; `--no-default-features` check passes.
- Strict Clippy passes for the portable crate, iOS device target, and iOS Simulator target.
- iOS device (`aarch64-apple-ios`) and Simulator (`aarch64-apple-ios-sim`) target checks pass.
- `cargo doc --no-deps` passes for the portable crate and iOS device target.
- Formatting, `git diff --check`, source/privacy review, and zero-Swift-source audit pass.

The scoped Cargo commands resolved a temporary local lock update for the two new packages and `objc2-health-kit 0.3.2`; the isolated `Cargo.lock` is restored to base. Root must resolve the new packages before any root `--locked` check. No app consumer was linked or launched, no device/Simulator permission flow ran, and no health sample was read or written. The worktree base is `7b5513fa2a4864d21a594cbf1fbd43951427155`; no commit or push is included.
