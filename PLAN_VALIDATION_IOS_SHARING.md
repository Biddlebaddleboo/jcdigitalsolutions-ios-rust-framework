# PLAN_VALIDATION_IOS_SHARING.md — Workstream G7: iOS Sharing CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for `ios-sharing`

Record that these checks do not prove live pasteboard privacy UX or share UI behavior

## Dependencies

- G1 shared CI is integrated
- B6 `ios-sharing` and `docs/ios/sharing.md` are integrated
- D5 and D6 `framework-sharing` contracts are integrated

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_CLIPBOARD.md`
- `PLAN_CAPABILITIES_CLIPBOARD.md`
- `PLAN_CAPABILITIES_SHARE.md`
- `docs/VALIDATION.md`
- `docs/ios/sharing.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit iOS backend code, portable contracts, root Cargo config, `Cargo.lock`, capability status, or runtime behavior. Do not add live pasteboard or share UI actions, signing, or device execution claims

## Required CI

- Add locked `cargo check` gates for `ios-sharing` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets
- Run B7's share-only Release link/import/minos gate on macOS; build and inspect but never execute its example probe
- Run the gates on macOS runners with both Rust targets and Xcode SDKs
- Update `docs/VALIDATION.md` with exact commands and the limit: compile/lint only, no live pasteboard, privacy prompt, share UI, activity result, or callback/drop race proof

## Status and evidence

- G7's four locked check/Clippy gates are present in `.github/workflows/ci.yml` under macOS conditions after CI installs both iOS Rust targets
- Pass: `cargo check --locked -p ios-sharing --target aarch64-apple-ios`; `cargo check --locked -p ios-sharing --target aarch64-apple-ios-sim`; `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Pass: `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`; `cargo xtask docs-check`; `git diff --check`
- `docs/VALIDATION.md` already records all four commands and limits the evidence to compile/lint. No tests, live pasteboard access, privacy prompt, share UI, activity result, callback/drop-race test, signing, or device execution ran; no runtime claim is made
- G116's `sh platform/ios/ios-sharing/check-share-link-imports.sh` passed locally after it was added to macOS CI. It selects only `share`, imports CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, and records minos 10.0/14.0. It does not execute its probes or claim live share UI behavior; no passing CI workflow run is recorded
- The local target gates do not record a CI workflow run or prove device/simulator behavior; the sharing guide records Xcode 26.6 and SDK 26.5 evidence below the Xcode 27.x plan baseline

## Validation and handoff

- Run all four target-specific commands
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`
- Report changed files, commit SHA, exact commands/results, deviations, and open assumptions. Do not push
