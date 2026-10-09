# PLAN_VALIDATION_IOS_METAL.md — Workstream G35: Metal Presence Query Gates

## Objective

Define focused compile, lint, format, and documentation gates for the narrow B41 Metal default-device presence query. These gates do not prove GPU execution or performance.

## Dependencies

- `PLAN_CAPABILITIES_METAL.md`, `PLAN_IOS_METAL.md`, `framework-metal`, and `ios-metal` are integrated
- iOS device and simulator Rust targets and Xcode SDKs are installed
- Shared CI integration remains orchestrator-owned

## Write scope

- `PLAN_VALIDATION_IOS_METAL.md`
- package-local source and guides only when completing D36/B41 work

Do not edit `.github/workflows/ci.yml`, `docs/VALIDATION.md`, root workspace files, capability matrix, aggregate plans, documentation indexes, or unrelated source. The lockfile may contain only package/dependency entries required by D36/B41.

## Required gates

- `cargo fmt --all -- --check`
- `cargo test -p framework-metal`
- `cargo check -p framework-metal --no-default-features`
- `cargo check -p ios-metal --target aarch64-apple-ios --locked`
- `cargo check -p ios-metal --target aarch64-apple-ios-sim --locked`
- `cargo clippy -p ios-metal --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy -p ios-metal --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo xtask docs-check`
- Build the iOS release library and inspect dependency/framework imports; verify no `.swift` source and no Swift runtime symbol
- `git diff --check`

These gates provide source/target compilation, lint, and documentation evidence only. They do not measure device presence on physical hardware, initialize a rendering or compute pipeline, submit GPU work, validate Metal features, or establish throughput, power use, latency, or Simulator/device parity.
