# PLAN_VALIDATION_IOS_PRESENTATION.md — Workstream G9: iOS Presentation CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for the integrated partial `ios-presentation` acknowledgement-alert crate.

## Dependencies

- G1 shared CI is integrated
- B9 `ios-presentation` and `docs/ios/presentation.md` are integrated
- `Cargo.lock` includes the B9 dependency graph

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_PRESENTATION.md`
- `docs/VALIDATION.md`
- `docs/ios/presentation.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit the iOS backend, portable contracts, root Cargo workspace/dependencies, lockfile, capability matrix, or runtime behavior. Do not add live alert UI actions, signing, or device execution claims.

## Required CI

- Add locked `cargo check` gates for `ios-presentation` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets
- Run the gates on macOS runners with both Rust targets and Xcode SDKs
- Update `docs/VALIDATION.md` with exact commands and the limit: compile/lint only, with no live alert presentation, dismissal, or user-interaction proof

## Validation and handoff

- Run all four target-specific commands
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push

## Status and evidence

Status: complete. The existing macOS workflow contains all four locked device/simulator check and strict Clippy gates. `docs/VALIDATION.md` lists the exact commands and limits the evidence to compile/lint; `docs/ios/presentation.md` states that no live alert display, dismissal, or device/simulator behavior is claimed

Validation passed at repository HEAD `53ef4ef`:

- `cargo check --locked -p ios-presentation --target aarch64-apple-ios`
- `cargo check --locked -p ios-presentation --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-presentation --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`
- `cargo xtask docs-check`
- `git diff --check`

Only this plan's status/evidence was changed for G9; the workflow and validation documentation already met the acceptance criteria. No tests, app launch, live alert presentation/dismissal, user interaction, signing, or device runtime check was performed. No commit or push was made
