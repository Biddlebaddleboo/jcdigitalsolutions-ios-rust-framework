# PLAN_VALIDATION_IOS_ACCESSIBILITY.md — Workstream G8: iOS Accessibility CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for the integrated `ios-accessibility` crate

Record that these checks do not prove live VoiceOver, focus, announcement, or user-experience behavior

## Dependencies

- G1 shared CI is integrated
- B8 `ios-accessibility` and `docs/ios/accessibility.md` are integrated
- `Cargo.lock` includes the B8 dependency graph

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_ACCESSIBILITY.md`
- `docs/VALIDATION.md`
- `docs/ios/accessibility.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit the iOS backend, portable contracts, root Cargo workspace/dependencies, lockfile, capability matrix, or runtime behavior. Do not add live VoiceOver actions, signing, or device execution claims

## Required CI

- Add locked `cargo check` gates for `ios-accessibility` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets
- Run the gates on macOS runners with both Rust targets and Xcode SDKs
- Update `docs/VALIDATION.md` with exact commands and the limit: compile/lint only, with no live VoiceOver, focus, announcement, or accessibility UX proof

## Status and evidence

- G8's four locked check/Clippy gates are present in `.github/workflows/ci.yml` under macOS conditions after CI installs both iOS Rust targets
- Pass: `cargo check --locked -p ios-accessibility --target aarch64-apple-ios`; `cargo check --locked -p ios-accessibility --target aarch64-apple-ios-sim`; `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Pass: `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`; `cargo xtask docs-check`; `git diff --check`
- `docs/VALIDATION.md` already records all four commands and limits evidence to compile/lint. No tests, live VoiceOver, focus movement, announcement delivery, accessibility UX, signing, or device execution ran; no runtime claim is made
- The local target gates do not record a CI workflow run or prove device/simulator behavior; `docs/ios/accessibility.md` records Xcode 26.6 / SDK 26.5 compile/link evidence below the Xcode 27.x plan baseline

## Validation and handoff

- Run all four target-specific commands
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push
