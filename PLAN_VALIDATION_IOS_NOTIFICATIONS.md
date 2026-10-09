# PLAN_VALIDATION_IOS_NOTIFICATIONS.md — Workstream G5: iOS Notification Target Gates

## Objective

Keep device and simulator build gates for the B4 local-notification backend and separate B12 response bridge in macOS CI

## Dependencies

- `PLAN_IOS_NOTIFICATIONS.md` and `ios-notifications` are integrated
- `PLAN_IOS_NOTIFICATION_RESPONSES.md` and `ios-notification-responses` are integrated
- The existing CI target installation and validation docs are available
- Add no dependency

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`
- `PLAN_VALIDATION_IOS_NOTIFICATIONS.md`
- G5 entry in `PLAN_VALIDATION.md`

Do not alter notification API semantics, iOS backend code, the global capability manifest, or unrelated CI jobs.

## Required gates

- On macOS, run locked `cargo check` for `ios-notifications` and `ios-notification-responses` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- On macOS, run Clippy with `--all-targets -- -D warnings` for both crates on both targets
- Add or run no tests; make no live permission, delivery, or response claim
- Update validation docs to separate compile/lint gates from device/simulator permission and delivery evidence

## Status and evidence

- G5's eight locked check/Clippy gates are present in `.github/workflows/ci.yml` under macOS conditions, after CI installs both iOS Rust targets
- Pass: `cargo check --locked -p ios-notifications --target aarch64-apple-ios`; `cargo check --locked -p ios-notifications --target aarch64-apple-ios-sim`; `cargo clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Pass: `cargo check --locked -p ios-notification-responses --target aarch64-apple-ios`; `cargo check --locked -p ios-notification-responses --target aarch64-apple-ios-sim`; `cargo clippy --locked -p ios-notification-responses --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-notification-responses --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Pass: `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`; `cargo xtask docs-check`; `git diff --check`
- `docs/VALIDATION.md` already lists the exact commands and compile/lint limits; no shared-doc edit was needed. No tests ran, no live permission, notification delivery, or response callback was exercised, and no runtime claim is made
- These local target checks do not record a CI workflow run or establish device/simulator runtime behavior; the shared validation doc notes that the available Xcode is below the Xcode 27.x plan baseline

## Validation and handoff

- Verify workflow YAML and run target commands locally where the installed Apple SDK permits; B12 device/simulator checks are already confirmed
- Run `cargo xtask docs-check` and `git diff --check`
- Report changed paths, exact checks, deviations, and unresolved assumptions
