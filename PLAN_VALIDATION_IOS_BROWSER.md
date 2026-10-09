# PLAN_VALIDATION_IOS_BROWSER.md — Workstream G11: iOS Browser CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for the integrated `ios-browser` HTTPS URL-handler backend.

## Dependencies

- G1 shared CI is integrated
- B11 `ios-browser` and `docs/ios/browser.md` are integrated
- `Cargo.lock` includes the B11 dependency graph

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_BROWSER.md`
- `docs/VALIDATION.md`
- `docs/ios/browser.md`
- `.github/workflows/ci.yml`

## Write scope

- `PLAN_VALIDATION_IOS_BROWSER.md`
- `PLAN_VALIDATION.md` — G11 link only
- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit the iOS backend, portable URI contract, root Cargo workspace/dependencies, lockfile, capability matrix, or runtime behavior. Do not add tests, Swift source, a live URL-handler call, a browser launch, or a network request.

## Required CI

- Add locked `cargo check` gates for `ios-browser` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Add strict all-target `cargo clippy` gates for the same package and targets with `-D warnings`.
- Run the gates on macOS runners with both Rust targets and Xcode SDKs.
- Update `docs/VALIDATION.md` with the exact commands and their limits: compile/lint only, with no live URL-handler call or app/browser runtime proof.

## Status and evidence

- G11 CI gates are present in `.github/workflows/ci.yml` on macOS for both `aarch64-apple-ios` and `aarch64-apple-ios-sim`; CI installs both Rust targets before the gates
- `docs/VALIDATION.md` records all four exact commands and limits the evidence to compile/lint; `docs/ios/browser.md` also states that no URL handler, browser, or network request was run
- Pass: `cargo check --locked -p ios-browser --target aarch64-apple-ios`; `cargo check --locked -p ios-browser --target aarch64-apple-ios-sim`; `cargo clippy --locked -p ios-browser --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-browser --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Pass: `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`; `cargo xtask docs-check`; `git diff --check`
- No tests, live URL-handler call, browser launch, or network request ran; no app/browser runtime result is claimed

## Validation and handoff

- Run all four target-specific check/lint commands.
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`.
- Do not add or run tests, emit Swift source, launch a browser, make a URL-handler call, or send a network request.
- Report changed files, commit SHA, exact commands/results, deviations, and unresolved assumptions. Do not push.
