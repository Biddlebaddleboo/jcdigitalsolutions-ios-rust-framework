# PLAN_VALIDATION_IOS_CONNECTIVITY.md — Workstream G12: iOS Connectivity Gates

## Status

CI wiring, the B18 `ios-connectivity` crate, and its focused link/import script are present. On 2026-10-08, the device/simulator checks, strict Clippy gates, and link/import script passed independently on Rust 1.94.1 with Xcode 26.6 build 17F113 and SDK 26.5. The direct imports are exactly Network.framework and `/usr/lib/libSystem.B.dylib`; the probes were linked, not executed. G12 local validation is complete, but this is not evidence of a passing CI workflow run or live connectivity/path behavior

## Objective

Add persistent macOS CI checks for B18's iOS device/simulator compilation, lint, link/import, and
zero-Swift-source boundaries without claiming live path observation

## Dependencies

- G1 validation tooling and CI are integrated
- B18 `ios-connectivity` is integrated; see `PLAN_IOS_CONNECTIVITY.md`
- D15 portable contract is integrated; see `PLAN_CAPABILITIES_CONNECTIVITY.md`

## Write scope

- `PLAN_VALIDATION_IOS_CONNECTIVITY.md`
- `.github/workflows/ci.yml`
- concise G12 evidence in `PLAN_VALIDATION.md` and `docs/VALIDATION.md`

Do not edit the D15/B18 APIs, platform backend, Cargo manifests/lockfile, shared capability matrix,
or other target gates. Root owns integration and shared documentation indexes

## Required gates

- On macOS, install the `aarch64-apple-ios` and `aarch64-apple-ios-sim` Rust targets
- Run `cargo check --locked -p ios-connectivity --target aarch64-apple-ios`,
  `cargo check --locked -p ios-connectivity --target aarch64-apple-ios-sim`,
  `cargo clippy --locked -p ios-connectivity --all-targets --target aarch64-apple-ios -- -D warnings`,
  and `cargo clippy --locked -p ios-connectivity --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Run `sh platform/ios/ios-connectivity/check-link-imports.sh` and verify its exact direct
  framework/system imports; reject Swift/Python runtime and unrelated capability imports
- Retain host `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and formatting gates
- Do not add runtime path or endpoint tests to CI; compile/link/import checks do not establish
  path-change delivery, network availability, request success, or cancellation timing

## Validation and handoff

- After package/lock reconciliation, these checks passed on Xcode 26.6 build 17F113 / SDK 26.5:
  `cargo +1.94.1 check --locked -p ios-connectivity --target aarch64-apple-ios`,
  `cargo +1.94.1 check --locked -p ios-connectivity --target aarch64-apple-ios-sim`,
  `cargo +1.94.1 clippy --locked -p ios-connectivity --all-targets --target aarch64-apple-ios -- -D warnings`,
  `cargo +1.94.1 clippy --locked -p ios-connectivity --all-targets --target aarch64-apple-ios-sim -- -D warnings`,
  and `sh platform/ios/ios-connectivity/check-link-imports.sh`. An earlier pre-reconciliation device check failed at the then-unresolved `framework_core` import and was superseded by these passing final-state checks
- The script-built arm64 device and Simulator link probes at
  `target/ios-connectivity-link-aarch64-apple-ios/aarch64-apple-ios/release/examples/ios_connectivity_link_import_probe`
  and
  `target/ios-connectivity-link-aarch64-apple-ios-sim/aarch64-apple-ios-sim/release/examples/ios_connectivity_link_import_probe`
  are the checked artifacts. `vtool -show-build` reports device minimum iOS 12.0 and Simulator
  minimum iOS 14.0; both use SDK 26.5. `otool -L` reports exactly Network.framework and
  `/usr/lib/libSystem.B.dylib` for both. The script also verifies those deployment targets and
  its forbidden-symbol scan passed. The device and Simulator executables were not run; the
  checks do not establish path-change delivery, network availability, endpoint reachability,
  request success, or cancellation timing
- The available Xcode 26.6 toolchain is below the planned Xcode 27.x baseline
- Validate YAML, docs index, zero-Swift-source, and `git diff --check`
- Report changed files, exact commands/results, imports, and runtime evidence limits; do not edit
  shared capability totals
