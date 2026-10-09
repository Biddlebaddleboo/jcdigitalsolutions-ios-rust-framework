# PLAN_VALIDATION_IOS_BACKGROUND_TASKS.md — Workstream G19: App Refresh Task Gates

## Status

G19 covers D20 and B25 compile, lint, docs, and link/import evidence only. It does not prove a live task run

## Objective

Define exact no-test checks for `framework-background` and `ios-background-tasks`, plus direct import checks for device and Simulator probes

## Dependencies

- D20 portable contract: `PLAN_CAPABILITIES_BACKGROUND_TASKS.md`
- B25 iOS adapter: `PLAN_IOS_BACKGROUND_TASKS.md`
- Rust 1.94.1, Xcode 26.6 build 17F113, iPhoneOS and iPhoneSimulator SDK 26.5

## Required checks

- `cargo +1.94.1 check --locked -p framework-background --no-default-features`
- `cargo +1.94.1 clippy --locked -p framework-background --all-targets -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-background-tasks --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-background-tasks --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-background-tasks --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-background-tasks --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 fmt --manifest-path crates/framework-background/Cargo.toml -- --check`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-tasks/Cargo.toml -- --check`
- `cargo +1.94.1 doc --locked -p framework-background -p ios-background-tasks --no-deps --target aarch64-apple-ios`
- `cargo +1.94.1 --locked xtask docs-check`
- `cargo +1.94.1 --locked xtask zero-swift-source`
- `sh platform/ios/ios-background-tasks/check-link-imports.sh`
- `git diff --check`

The link script builds device and Simulator probes but does not run them. It checks the direct import allowlist, selected BackgroundTasks symbols, and no-Swift/unrelated-framework exclusions

## Runtime boundary

No app launch, plist validation, schedule request, live task callback, expiry event, task finish, physical-device run, or Simulator run is part of these checks. Target compile and link evidence do not prove scheduler behavior or run timing

## Handoff

Record exact commands and results, Rust/Xcode/SDK versions, direct imports, public API floor, changed paths, and any failed or skipped gate. Keep the Xcode 27.x baseline gap explicit
