# PLAN_VALIDATION_IOS_NEARBY.md — Workstream G33: Nearby Interaction Capability Gates

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
