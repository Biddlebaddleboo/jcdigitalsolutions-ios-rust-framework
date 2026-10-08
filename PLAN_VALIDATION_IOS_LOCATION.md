# PLAN_VALIDATION_IOS_LOCATION.md — Workstream G6: Core Location CI Gates

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
- Run the gates only on macOS runners with both Rust targets and Xcode SDKs installed.
- Update `docs/VALIDATION.md` to list the exact commands and state that they provide compile/lint evidence only; live permission UI, GPS delivery, fix quality/freshness, and cancellation races remain untested.

## Validation and handoff

- Run all four target-specific commands.
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`.
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push.
