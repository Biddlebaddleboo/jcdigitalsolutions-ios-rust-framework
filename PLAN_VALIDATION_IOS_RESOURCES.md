# PLAN_VALIDATION_IOS_RESOURCES.md — Workstream G10: iOS Resources CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for the integrated `ios-resources` main-bundle file lookup backend.

## Dependencies

- G1 shared CI is integrated
- B10 `ios-resources` and `docs/ios/resources.md` are integrated
- `Cargo.lock` includes the B10 dependency graph

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_RESOURCES.md`
- `docs/VALIDATION.md`
- `docs/ios/resources.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit the iOS backend, portable contract, root Cargo workspace/dependencies, lockfile, capability matrix, or runtime behavior. Do not add a live resource-bundle app action or claim filesystem containment, localization, asset-catalog access, or runtime validation.

## Required CI

- Add locked `cargo check` gates for `ios-resources` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets.
- Run the gates on macOS runners with both Rust targets and Xcode SDKs.
- Update `docs/VALIDATION.md` with exact commands and the limit: compile/lint only, with no live bundle-resource lookup or app runtime proof.

## Validation and handoff

- Run all four target-specific commands.
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`.
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push.
