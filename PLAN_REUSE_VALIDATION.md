# PLAN_REUSE_VALIDATION.md — R2: reusable capability validation

## Implementation scope

Inspect `tools/xtask/src/main.rs::{run,help_text}`, `tools/xtask/src/audits.rs`, `tools/xtask/src/sdk_inventory.rs`, `.github/workflows/ci.yml`, `PLAN_VALIDATION.md` and check scripts for `ios-homekit-identify-status`, `ios-photogrammetry-status`, `ios-alarmkit-status`, and `ios-activitykit-status`. Proposed harness under `tools/validation-harness/**` or small `xtask` modules.

## Verified facts

Pilot scripts repeat fmt, locked checks, Clippy, rustdoc, device/simulator, documentation and zero-Swift checks. The HomeKit pilot additionally asserts the getter, feature set, availability and forbidden operations. Swift pilots separately audit imports and compiler-generated ABI. Existing xtask implements other useful audits; parity is not yet runnable.

## Required behavior

- Declarative validation specification: crate, exact targets, deployment floor, expected public imports, ABI fixture, forbidden imports/operations and required documentation gates.
- Retain all per-capability semantic/negative checks and compiler-oracle assumptions. Generalize only common mechanical execution.
- Add `--list`, `--explain <id>`, `--capability <id>`, explicit-base `--changed`, `--all`. Changed-file selection must account for reverse dependencies and conservatively validate all ABI consumers after shared helper changes.
- Machine-readable and human-readable results distinguish pass, fail and skipped-with-reason. Distinguish compile, link, import scan, simulator execution and device proof; missing toolchains never count as passes.
- Keep existing CI coverage, public-API and no-shipping-Swift constraints; migrate only four pilots before broader rollout.

## Deterministic tests and validation

Use fake command runner, fake SDK/tool availability, import fixtures, dependency graph and explicit expected command snapshots. Test missing import, forbidden symbol, incorrect OS floor, skipped!=passed, shell escaping and reverse-dependency effects. Compare each pilot's old/new checks. Run locked xtask check, Clippy and unit tests, fmt, all pilot scripts where macOS is available, docs-check, zero-swift-source and relevant CI commands. Record static versus executed evidence precisely.

## Ownership and handoff

R2 exclusively edits `tools/xtask/**`, CI and harness schema; R1 owns build helper and build.rs, R3 owns Swift internals. Report commands, results, before/after assertions, skipped tests, unsupported scripts and SHA.
