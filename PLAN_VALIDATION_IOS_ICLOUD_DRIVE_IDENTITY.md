# PLAN_VALIDATION_IOS_ICLOUD_DRIVE_IDENTITY.md — Workstream G29: iCloud Drive Identity Validation

## Installed shared-tooling boundary

The R1/R2 build and validation engines are already installed, pinned PATH tools; use `docs/SHARED_TOOLING.md` rather than reimplementing their internals. This workstream is **not a registered pilot** in the current `tools/validation/specs/validation-v1.json`, so do not claim a passing `ios-rust-validate --capability` command for it. The existing focused package scripts, CI checks and command/evidence records below remain required until a schema-v1 declarative profile demonstrably reproduces all applicable checks, including iCloud entitlement, cloud-token ambiguity, no-default-feature checks. Only remove duplicate commands after proving positive, deliberately failing negative, dependency-selection and fail/skip parity. Optional Python adapters must be narrowly scoped and cannot substitute for a real compiler/link or device gate. Tool engine defects are reported in sanitized `BUG_REPORT_*.md`; ordinary API work never retrieves historical engine source.

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
