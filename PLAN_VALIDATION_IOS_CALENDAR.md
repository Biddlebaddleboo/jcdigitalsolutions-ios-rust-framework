# PLAN_VALIDATION_IOS_CALENDAR.md — Workstream G24: Calendar Authorization Gates

## Installed shared-tooling boundary

The R1/R2 build and validation engines are already installed, pinned PATH tools; use `docs/SHARED_TOOLING.md` rather than reimplementing their internals. This workstream is **not a registered pilot** in the current `tools/validation/specs/validation-v1.json`, so do not claim a passing `ios-rust-validate --capability` command for it. The existing focused package scripts, CI checks and command/evidence records below remain required until a schema-v1 declarative profile demonstrably reproduces all applicable checks, including EventKit authorization and release import probes. Only remove duplicate commands after proving positive, deliberately failing negative, dependency-selection and fail/skip parity. Optional Python adapters must be narrowly scoped and cannot substitute for a real compiler/link or device gate. Tool engine defects are reported in sanitized `BUG_REPORT_*.md`; ordinary API work never retrieves historical engine source.

## Objective

Provide a package-local validation gate for D25/B30 that verifies portable behavior and iOS target compilation without claiming live authorization behavior.

## Dependencies

- `PLAN_CAPABILITIES_CALENDAR.md`
- `PLAN_IOS_CALENDAR.md`
- Xcode iOS SDK and Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim`

## Write scope

- `platform/ios/ios-calendar/check.sh`
- `PLAN_VALIDATION_IOS_CALENDAR.md`
- `docs/ios/calendar.md` for evidence limits only

Root CI, global validation docs, workspace manifests and lockfile, aggregate plans, canonical manifest, and `tools/xtask` remain root-owned.

## Required gates

- Format the workspace and run portable calendar contract tests.
- Run host `ios-calendar` conversion and callback-state tests without EventKit runtime.
- Check `framework-calendar` without default features.
- On macOS, check `ios-calendar` for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run strict Clippy (`-D warnings`) for all targets of `ios-calendar` on both iOS targets.
- Build a temporary release `cdylib` probe for device and simulator; confirm EventKit and Foundation are the only Apple framework imports and no Swift runtime import is present.
- Run rustdoc with warnings denied for the portable and iOS crates.
- Run `cargo run --locked --quiet -p xtask -- docs-check` for shared documentation index and zero-Swift-source checks.
- Run `git diff --check`.
- Keep the consuming host's `NSCalendarsFullAccessUsageDescription` requirement and iOS 17.0 API floor explicit.

## Evidence limits

Target compile/lint checks do not run an EventKit prompt, select a user's response, inspect calendar contents, validate host property-list configuration, or establish device runtime behavior. Do not claim device permission behavior based on simulator or target compilation.

## Status

Complete. `platform/ios/ios-calendar/check.sh` passed, including format, 3 portable tests, no-default-features check, 4 host `ios-calendar` tests, rustdoc with warnings denied, device/simulator target checks, strict Clippy for both targets, release import probes for both targets, `cargo run --locked --quiet -p xtask -- docs-check`, and `git diff --check`. Checks used a temporary generated worktree lock containing `objc2-event-kit` 0.3.2; the root-owned `Cargo.lock` is restored and must receive that resolver entry before the locked package gate runs in the integrated root. These checks establish compilation and linked imports only, not live permission behavior.
