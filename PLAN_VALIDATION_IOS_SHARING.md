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
- G116's `sh platform/ios/ios-sharing/check-share-link-imports.sh` passed locally after it was added to macOS CI. It selects only `share`, imports CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, and records minos 10.0/14.0. It does not execute its probes or claim live share UI behavior; hosted CI evidence is recorded below
- The local recheck below used Xcode 26.6 and SDK 26.5, below the Xcode 27.x plan baseline; hosted qualification is recorded below. Neither proves device/simulator runtime behavior

### 2026-10-10 recheck

At `570461341d97df42200db0dba5ca1791b0f3abf6`, Rust 1.94.1, Xcode 26.6 build 17F113, and iOS
SDK 26.5:

- PASS: `cargo +1.94.1 check --locked -p ios-sharing --target aarch64-apple-ios`
- PASS: `cargo +1.94.1 check --locked -p ios-sharing --target aarch64-apple-ios-sim`
- PASS: `cargo +1.94.1 clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios -- -D warnings`
- PASS: `cargo +1.94.1 clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- PASS: `sh platform/ios/ios-sharing/check-share-link-imports.sh` for device and Simulator. The share feature excludes clipboard; both probes import CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos is 10.0 for device and 14.0 for Simulator.
- PASS: Ruby CI YAML parse, `cargo +1.94.1 xtask docs-check`, and `git diff --check`.

No tests, probe execution, app launch, live pasteboard access, privacy prompt, share UI, activity result, callback/drop-race behavior, or device execution ran. This local recheck is compile, Clippy, static link/import, and docs evidence only.

### Hosted Xcode 27 CI evidence — 2026-10-10

GitHub Actions run [38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully for source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. All three jobs (`Rust checks (ubuntu-24.04)`, `Rust checks (macos-15)`, and `Rust checks (xcode-27)`) concluded successfully. G7's macOS-only steps passed in both macOS jobs and were skipped in Ubuntu. The Xcode 27 runner reported Xcode 27.0 build `27A266a`, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0.

On `Rust checks (xcode-27)`, these exact steps passed:

- `Check iOS sharing device target` — `cargo check --locked -p ios-sharing --target aarch64-apple-ios`
- `Check iOS sharing simulator target` — `cargo check --locked -p ios-sharing --target aarch64-apple-ios-sim`
- `Clippy iOS sharing device target` — `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios -- -D warnings`
- `Clippy iOS sharing simulator target` — `cargo clippy --locked -p ios-sharing --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `B7 iOS share-only release link/import/minos gate` — `sh platform/ios/ios-sharing/check-share-link-imports.sh`

The Xcode 27 and macOS 15 link/import gates passed for device and Simulator. They build and inspect probes; they do not execute them. This is hosted compile, strict-Clippy, and static link/import evidence at the recorded source SHA, not proof of live pasteboard access, privacy prompt, share UI, activity result, callback/drop-race behavior, or device behavior. No sharing app or UI was run.

## Validation and handoff

- Run all four target-specific commands
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`
- Report changed files, commit SHA, exact commands/results, deviations, and open assumptions. Do not push
