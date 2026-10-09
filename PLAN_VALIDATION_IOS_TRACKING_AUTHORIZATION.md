# PLAN_VALIDATION_IOS_TRACKING_AUTHORIZATION.md — Workstream G34: ATT Status Gates

## Objective

Add package-local repeatable validation gates for the D35/B40 status-only App Tracking Transparency slice. Do not add shared CI.

## Dependencies

- D35 `framework-auth`
- B40 `ios-auth`
- Workspace lockfile reconciliation at integration time

## Write scope

- `platform/ios/ios-auth/check.sh`
- this validation plan

Do not edit shared CI, root Cargo configuration, capability manifests, docs indexes, aggregate plans, or `tools/xtask`.

## Required gates

- `cargo fmt --package framework-auth --package ios-auth -- --check`
- `cargo test --locked -p framework-auth`
- `cargo check --locked -p framework-auth --no-default-features`
- strict Clippy for `framework-auth`
- `cargo check --locked -p ios-auth` and strict Clippy for `aarch64-apple-ios`
- `cargo check --locked -p ios-auth` and strict Clippy for `aarch64-apple-ios-sim`
- `git diff --check`

These are compile, lint, and portable unit checks only. They do not read a live system status, display an authorization prompt, record a user choice, or execute any tracking operation.
