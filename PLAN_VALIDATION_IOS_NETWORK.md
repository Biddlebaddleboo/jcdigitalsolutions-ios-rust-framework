# PLAN_VALIDATION_IOS_NETWORK.md — Workstream G4: iOS Network Target Gates

## Objective

Keep the iOS foreground HTTP backend's device and simulator build contracts in macOS CI.

## Dependencies

Requires `PLAN_IOS_NETWORK.md` and the integrated `ios-network` crate.
Uses the existing Rust target installation and CI toolchain; add no dependencies.

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not alter HTTP API semantics, iOS backend code, the global capability manifest, or unrelated CI jobs.

## Required gates

- On macOS, run locked `cargo check` for `ios-network` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- On macOS, run Clippy with `-D warnings` for all targets of `ios-network` on both targets.
- Keep host workspace tests as the only source of host runtime-test claims; do not add endpoint/network-dependent CI tests.
- Update validation docs to separate compile/lint gates from device/simulator URLSession runtime evidence.

## Validation and handoff

- Verify the workflow YAML and run the same cargo commands locally where the installed Apple SDK permits.
- Run `cargo xtask docs-check` and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions.
