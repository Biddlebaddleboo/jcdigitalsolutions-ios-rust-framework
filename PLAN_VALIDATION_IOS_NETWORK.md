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

## Status and evidence

- CI has locked device/simulator `cargo check`, strict all-target Clippy, and `sh platform/ios/ios-network/check-link-imports.sh` gates for `ios-network` under macOS conditions after both Apple Rust targets are installed
- Pass: `cargo check --locked -p ios-network --target aarch64-apple-ios`; `cargo check --locked -p ios-network --target aarch64-apple-ios-sim`
- Pass: `cargo clippy --locked -p ios-network --all-targets --target aarch64-apple-ios -- -D warnings`; `cargo clippy --locked -p ios-network --all-targets --target aarch64-apple-ios-sim -- -D warnings`. The link-only example now explicitly drops the unpolled future without a `let _` binding; no URLSession task starts
- Pass: `sh platform/ios/ios-network/check-link-imports.sh`; it links device and simulator probes and verifies direct imports `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` while rejecting the script's selected Swift/Python/runtime and unrelated-capability symbol patterns. The probes do not run
- Pass: `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/ci.yml"); puts "CI YAML parse passed"'`; `cargo xtask docs-check`; `git diff --check`
- `cargo test --locked -p ios-network` passed 10 deterministic host unit tests for conversions and operation completion/drop races. No endpoint request, local-server differential, URLSession runtime, or HTTP parity claim is made; G4 probes are link-only and were not executed
- `docs/VALIDATION.md` records exact target commands and compile/link limits

## Parity and measurement evidence blocker

No Apple runtime reference result can be recorded by the current harness. The host tests exercise
portable conversions and completion-cell races; the device/Simulator checks compile and lint, and
the Release probes link but are not executed. [`docs/ios/network.md`](docs/ios/network.md#validation-status)
records that this crate has no integration-test app/runner. The recorded Xcode 26.6 environment is
below the repository's Xcode 27.x baseline in [`PLAN_IOS_NETWORK.md`](PLAN_IOS_NETWORK.md), and
[`E1`](PLAN_REPLACEMENTS_HTTP.md) found no replacement candidate to compare. A future parity result
requires a named candidate plus an Apple runner that exercises both the current URLSession baseline
and candidate against the same deterministic local fixture. Performance evidence additionally
requires the predeclared, representative physical-device Release A/B gate in E1. Until those
prerequisites exist, a new harness without observed URLSession reference outputs would assert no
parity and is not added.

When that blocker clears, the separate manual runtime record must identify the runner/app revision,
Xcode and SDK, device or Simulator model and OS build, fixture revision, exact request inputs,
observed response status/headers/body, portable error result where applicable, and any expected
normalization. Record candidate-versus-URLSession parity separately from timing; do not promote
compile, Clippy, or link/import results to runtime evidence. Keep the fixture local and deterministic;
do not add a live endpoint or timing-dependent test to CI.

## Validation and handoff

- Verify the workflow YAML and run the same cargo commands locally where the installed Apple SDK permits.
- Run `cargo xtask docs-check` and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions.
