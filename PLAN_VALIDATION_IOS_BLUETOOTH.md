# PLAN_VALIDATION_IOS_BLUETOOTH.md — G26: Bluetooth package gates

## Scope and dependencies
G26 covers D27 `framework-bluetooth` and B32 `ios-bluetooth` package-local validation. D29's central-discovery-specific checks are retained separately in `PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md`; do not collapse their semantics into a generic Bluetooth build result. Current prerequisites include the integrated `objc2-core-bluetooth` 0.3.2 lockfile dependency and installed Apple Rust device/simulator targets plus SDKs.

## Implementation ownership
Read the package-local scripts under `platform/ios/ios-bluetooth/scripts/**` and `docs/ios/bluetooth.md`. G26 may update this plan, those scripts and that guide. CI, workspace/lockfile, canonical capability data, aggregate plan/index, `tools/xtask`, shared engine source and Bluetooth runtime behavior are outside G26's write scope. CI integration is an orchestrator-owned follow-up.

## Using completed shared tooling
R1/R2 binaries are pinned installed tools (`docs/SHARED_TOOLING.md`). Bluetooth G26/D29 does **not** yet appear among the four registered validator pilots. Retain package-local test scripts as the accepted entrypoints. For an authorized new declarative profile, cover host portable tests, iOS compile/lint, dependency-feature and link/import checks, and D29-specific assertions; test a deliberate negative regression before retiring any old gate. Optional Python adapters are capability-specific and may not replace hardware proof.

## Required tests and evidence
- `cargo test -p framework-bluetooth`; `cargo check -p framework-bluetooth --no-default-features`.
- Locked `cargo check -p ios-bluetooth` and strict all-targets Clippy on both `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Formatting, documentation and package-local link/import checks, feature-tree inspection, `git diff --check`, and any additional D29 discovery-specific commands documented in its plan.
- Verify lockfile prerequisite, API availability floor and Bluetooth usage-description key; do not add live prompt/signing/simulator UI or device-consent automation.

These gates do **not** instantiate a manager, show permission UI, prove hardware authorization, scan or connect, test radio performance, or validate physical-device behavior. Record exact commands/results, SDK/toolchain, imports, skips, changed files, deviations, remaining assumptions and SHA. Suspected shared-engine defects use a sanitized `BUG_REPORT_*.md`, not an in-task engine repair.