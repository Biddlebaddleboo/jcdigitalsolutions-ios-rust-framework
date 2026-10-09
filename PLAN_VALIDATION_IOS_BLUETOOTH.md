# PLAN_VALIDATION_IOS_BLUETOOTH.md — Workstream G26: iOS Bluetooth Package Gates

## Objective

Provide package-local compile/lint gates for D27/B32. Central discovery gates are in [PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md](PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md). Central CI integration remains an orchestrator-owned follow-up.

## Dependencies

- D27 `framework-bluetooth` and B32 `ios-bluetooth` are present
- The integrated workspace lockfile resolves `objc2-core-bluetooth` 0.3.2
- Both Apple Rust targets and Xcode SDKs are installed on the macOS validation host

## Write scope

- `PLAN_VALIDATION_IOS_BLUETOOTH.md`
- Package-local scripts under `platform/ios/ios-bluetooth/scripts/**`
- `docs/ios/bluetooth.md`

Do not edit `.github/workflows/ci.yml`, root workspace files or lockfile, canonical capability data, aggregate plans/indexes, `tools/xtask`, or Bluetooth runtime behavior.

## Required gates

- Portable contract: `cargo test -p framework-bluetooth` and `cargo check -p framework-bluetooth --no-default-features`
- iOS device and simulator: locked `cargo check -p ios-bluetooth` and strict Clippy with `-D warnings` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run formatting, docs/link checks, dependency-feature review, and `git diff --check`
- State that these gates do not instantiate a manager, show permission UI, verify authorization on hardware, test the radio, or perform scans/connections
- Do not add live prompt, signing, simulator UI, Bluetooth-device, or device-consent automation

## Handoff

Report exact commands/results, the lockfile prerequisite for locked target gates, API floor, usage-description key, changed files, deviations, and unresolved assumptions. CI and shared validation-index integration are separate owner work.
