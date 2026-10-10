# PLAN_VALIDATION_IOS_NEARBY.md — Workstream G33: Nearby Interaction Capability Gates

## Installed shared-tooling boundary

The R1/R2 build and validation engines are already installed, pinned PATH tools; use `docs/SHARED_TOOLING.md` rather than reimplementing their internals. This workstream is **not a registered pilot** in the current `tools/validation/specs/validation-v1.json`, so do not claim a passing `ios-rust-validate --capability` command for it. The existing focused package scripts, CI checks and command/evidence records below remain required until a schema-v1 declarative profile demonstrably reproduces all applicable checks, including Nearby Interaction hardware availability versus compiled status-only query. Only remove duplicate commands after proving positive, deliberately failing negative, dependency-selection and fail/skip parity. Optional Python adapters must be narrowly scoped and cannot substitute for a real compiler/link or device gate. Tool engine defects are reported in sanitized `BUG_REPORT_*.md`; ordinary API work never retrieves historical engine source.

## Objective

Add a package-local repeatable validation script for the integrated D34/B39 status-only capability snapshot. Do not add shared CI or change root workspace configuration.

## Dependencies

- D34 `framework-nearby`
- B39 `ios-nearby`
- A reconciled workspace lockfile at integration time

## Write scope

- `platform/ios/ios-nearby/check.sh`
- this validation plan

Do not edit shared CI, root Cargo files, capability manifests, docs indexes, aggregate plans, or `tools/xtask`.

## Required gates

- `cargo fmt --all -- --check`
- `cargo test --locked -p framework-nearby`
- `cargo check --locked -p framework-nearby --no-default-features`
- strict Clippy for `framework-nearby`
- `cargo check --locked -p ios-nearby` and strict Clippy for `aarch64-apple-ios`
- `cargo check --locked -p ios-nearby` and strict Clippy for `aarch64-apple-ios-sim`
- `git diff --check`

All iOS gates are compile/lint checks only. They do not execute the device-capability query or prove simulator/device runtime behavior, session authorization, session start, peer discovery, or ranging.
