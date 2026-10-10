# PLAN_VALIDATION_IOS_METAL.md — G35: Metal presence-query gates

## Scope
Gate the narrow D36/B41 default Metal device-presence query provided by `framework-metal` and `ios-metal`. Start with `PLAN_CAPABILITIES_METAL.md`, `PLAN_IOS_METAL.md`, package manifests, and focused native link/import checks. Do not broaden the work to GPU command queues or rendering.

## Ownership and installed tools
G35 owns this plan and package-local source/guides only when implementing D36/B41. CI, `docs/VALIDATION.md`, root configuration, global capability manifest and indexes remain with the orchestrator/integration owner. Lockfile edits must be limited to required scoped dependencies.

The external R1/R2 tooling is finished. See `docs/SHARED_TOOLING.md`; G35 is **not registered** in `tools/validation/specs/validation-v1.json`. Use existing checks as the source of required coverage; add a declarative profile only if it preserves those checks and their negative cases. Do not implement or inspect the shared engine.

## Required checks
```sh
cargo fmt --all -- --check
cargo test -p framework-metal
cargo check -p framework-metal --no-default-features
cargo check --locked -p ios-metal --target aarch64-apple-ios
cargo check --locked -p ios-metal --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-metal --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-metal --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo xtask docs-check
git diff --check
```
Build the relevant iOS Release-linked library/probe and inspect its dependency and framework imports. Assert zero shipping `.swift` source, no Swift-runtime symbol, expected public Metal dependencies and no unrelated framework linkage. Link/import inspection is a separate acceptance gate; `cargo check` does not satisfy it. Preserve platform availability and `no_std` constraints.

## Limits and handoff
These are source, compile, lint, documentation and link gates only. They do not measure actual device presence, construct a pipeline, submit GPU work, verify Metal feature sets, prove hardware power/latency/throughput, or establish simulator/device parity. Report checks, host/SDK/target, artifact imports, skips and SHA. A future validator profile must demonstrate equivalent positive and negative results before legacy script removal. Report engine defects in `BUG_REPORT_*.md`.