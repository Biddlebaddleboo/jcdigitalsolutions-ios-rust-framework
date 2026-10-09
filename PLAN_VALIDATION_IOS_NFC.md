# PLAN_VALIDATION_IOS_NFC.md — Workstream G31: Core NFC Snapshot Gates

## Objective

Define compile, lint, format, and documentation gates for the narrow B37 Core NFC reader-support query. These gates do not prove live device NFC support or tag operations.

## Dependencies

- `PLAN_CAPABILITIES_NFC.md`, `PLAN_IOS_NFC.md`, `framework-nfc`, and `ios-nfc` are integrated
- iOS device and simulator Rust targets and Xcode SDKs are installed
- Shared CI integration remains orchestrator-owned

## Write scope

- `PLAN_VALIDATION_IOS_NFC.md`
- package-local source and guides only when completing D32/B37 work

Do not edit `.github/workflows/ci.yml`, `docs/VALIDATION.md`, root workspace files, capability matrix, aggregate plans, documentation indexes, or unrelated source. The lockfile may contain only the package and dependency entries required by D32/B37.

## Required gates

- `cargo fmt --all -- --check`
- `cargo test -p framework-nfc`
- `cargo check -p framework-nfc --no-default-features`
- `cargo check -p ios-nfc --target aarch64-apple-ios --locked`
- `cargo check -p ios-nfc --target aarch64-apple-ios-sim --locked`
- `cargo clippy -p ios-nfc --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy -p ios-nfc --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo xtask docs-check`
- Verify the iOS crate introduces no `.swift` source and inspect Release dependency/framework imports: CoreNFC plus Foundation required by `objc2-core-nfc`, no unrelated capability framework or Swift runtime
- `git diff --check`

The gates prove source/target compilation, lint, and documentation consistency only. They do not establish the device's NFC hardware result, an app's provisioning/Info.plist configuration, actual session startup, prompts, scans, tag reads/writes, background reading, or simulator NFC behavior.
