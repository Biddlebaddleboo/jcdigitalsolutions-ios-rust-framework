# PLAN_VALIDATION_IOS_ICLOUD_DRIVE_IDENTITY.md — Workstream G29: iCloud Drive Identity Validation

## Objective

Provide focused package-local validation for D30/B35 without editing shared CI or root metadata.

## Write scope

- `platform/ios/ios-cloud/check.sh`
- This validation plan

Do not edit shared CI, root workspace configuration or lockfile, capability status JSON/counts,
aggregate plans, shared documentation indexes, or `tools/xtask`.

## Required gates

The package-local script runs:

- `cargo fmt --all -- --check`
- `cargo test --locked -p framework-cloud`
- `cargo check --locked -p framework-cloud --no-default-features`
- `cargo check --locked -p ios-cloud --target aarch64-apple-ios`
- `cargo check --locked -p ios-cloud --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo xtask docs-check` for the shared docs check and zero-Swift-source gate
- `git diff --check`

The locked package commands require root integration to add the two local package entries to
`Cargo.lock`; the validation worktree resolves those entries before running the locked script. These
gates do not configure an iCloud entitlement, sign or launch an app, exercise a device account,
prove token availability, or test runtime identity changes. Simulator and device commands are
compile/lint evidence only.

## Handoff

Report exact commands and results, SDK/binding evidence, absent-token ambiguity, host setup, API
floor, and any root lockfile integration requirement. Do not claim a matrix-count change; the
orchestrator owns that metadata.
