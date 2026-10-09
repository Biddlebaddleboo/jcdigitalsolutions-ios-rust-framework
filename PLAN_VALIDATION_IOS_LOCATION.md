# PLAN_VALIDATION_IOS_LOCATION.md — Workstream G6: Core Location CI Gates

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
- Run the gates only on macOS runners with both Rust targets and Xcode SDKs installed.
- Update `docs/VALIDATION.md` to list the exact compile/lint and link/import commands, state that probes are not executed, and clarify that live permission UI, GPS delivery, fix quality/freshness, and cancellation races remain untested.

## Validation and handoff

- Run all four target-specific commands.
- Run `sh platform/ios/ios-location/check-link-imports.sh`.
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`.
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push.
