# PLAN_VALIDATION_IOS_LOCATION.md — Workstream G6: Core Location CI Gates

## Shared-tooling execution (current)

The R1/R2 PATH executables are complete; use `docs/SHARED_TOOLING.md` and schema-v1 specifications rather than new build/validation machinery. This workstream has **no existing registered pilot** in `tools/validation/specs/validation-v1.json`. Retain its recorded local commands and unique CoreLocation authorization, availability and import tests tests. An executor may declare a new validation profile and optional focused Python adapter; remove or replace existing checks only after equivalent positive, negative, cross-target and failure/skip parity is established. Do not claim an unregistered `ios-rust-validate --capability` run, infer runtime proof from static checks, or retrieve shared engine source. Escalate confirmed engine defects with `BUG_REPORT_*.md`.

## Status

G6's locked device/Simulator checks, strict Clippy, and link/import gate are integrated in macOS
CI. Local checks passed for both targets; `check-link-imports.sh` confirmed only the expected
CoreLocation/Foundation/system imports and exactly `CLLocation`, `CLLocationManager`, and
`CLLocationManagerDelegate` in the device and Simulator `objc2-core-location` feature trees. Probe
binaries were inspected, not executed. The gate emitted a rustc warning that
`IPHONEOS_DEPLOYMENT_TARGET` was 9.0 while rustc supports a minimum of 10.0; final device probe
minos is 10.0 and Simulator minos is 14.0. Workflow parsing, docs-check, and diff checks passed.
Xcode 26.6 / SDK 26.5 is below the plan's Xcode 27.x baseline; live permission and location
behavior remain unverified.

A Rust source-surface guard was added to the B5 link/import script after the recorded target and
probe run. This follow-up checks its shell syntax and source predicates only; the full gate was not
rerun because it builds Release probes.

## 2026-10-10 focused recheck

At `8e756ba0287a547ff769e55e649460282de6296a`, Rust 1.94.1, Xcode 26.6 build 17F113, and iOS
SDK 26.5:

- PASS: `cargo +1.94.1 check --locked -p ios-location --target aarch64-apple-ios`
- PASS: `cargo +1.94.1 check --locked -p ios-location --target aarch64-apple-ios-sim`
- PASS: `cargo +1.94.1 clippy --locked -p ios-location --all-targets --target aarch64-apple-ios -- -D warnings`
- PASS: `cargo +1.94.1 clippy --locked -p ios-location --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- PASS: `sh platform/ios/ios-location/check-link-imports.sh` for device and Simulator. Both probes import exactly CoreLocation, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; both feature trees contain only `CLLocation`, `CLLocationManager`, and `CLLocationManagerDelegate`. Device minos is 10.0; Simulator minos is 14.0; both use SDK 26.5.
- PASS: `cargo +1.94.1 xtask docs-check`; Ruby CI YAML parse; `git diff --check`.

No probe, test, app, permission query, or location request ran. This is compile, Clippy, static link/import, and docs evidence only; Xcode 27.x qualification and live permission/location behavior remain open.

## Objective

Add persistent iOS device and simulator compile/lint gates for the integrated B5 Core Location backend. Record precisely that these checks do not prove live permission, GPS, cancellation, or positioning behavior.

## Dependencies

- G1 shared CI is integrated
- B5 `ios-location` and `docs/ios/location.md` are integrated
- the workspace lockfile includes `objc2-core-location`

Read first:
- `PLAN_VALIDATION.md`
- `PLAN_IOS_LOCATION.md`
- `docs/VALIDATION.md`
- `docs/ios/location.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit the iOS backend, portable location contract, root Cargo workspace/dependencies, lockfile, capability matrix, or runtime behavior. Do not add live permission/GPS tests, endpoint-dependent checks, signing, or device execution claims.

## Required behavior

- Add locked `cargo check` gates for `ios-location` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets.
- Link a probe for both targets; require exact direct imports `CoreLocation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`, and reject Swift runtime or unrelated capability symbols.
- Require the link/import script to assert the one-shot request and foreground-authorization calls, confine `requestWhenInUseAuthorization()` to `start_authorization_request`, and reject out-of-scope Core Location operations in B5 Rust source/examples; this guard does not constrain host calls through the borrowed native manager.
- Run the gates only on macOS runners with both Rust targets and Xcode SDKs installed.
- Update `docs/VALIDATION.md` to list the exact compile/lint and link/import commands, state that probes are not executed, and clarify that live permission UI, GPS delivery, fix quality/freshness, and cancellation races remain untested.

## Validation and handoff

- Run all four target-specific commands.
- Run `sh platform/ios/ios-location/check-link-imports.sh`.
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`.
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push.
