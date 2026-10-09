# PLAN_VALIDATION.md — active cross-cutting validation

## Canonical implementation and historical evidence
Entry points: `tools/xtask/src/{main,audits,codegen,no_std_link,sdk_inventory}.rs`, `.github/workflows/ci.yml`, `docs/VALIDATION.md`, package-local `check.sh` and `check-link-imports.sh`, and `PLAN_REUSE_VALIDATION.md`. G1–G131 historical gate-by-gate report, including its evidence and unresolved notes, is preserved at Git `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_VALIDATION.md`. Historic green local static proofs are **not** device/runtime execution. Xcode 26.6/SDK 26.5 does not establish required Xcode 27.x qualification.

## Reusable validation workstream
Follow `PLAN_REUSE_VALIDATION.md`: shared deterministic command harness; preserve capability-specific negative API assertions, exact imports/exports, availability, ABI size/alignment/ownership, forbidden Swift source, no_std, feature isolation and device/simulator split. Changed-file selection must include reverse dependencies; nonexistent SDK/device cannot count as passed. CI must remain at least as strict during migration.

## Required remaining validation
- Verify all portable `no_std` and minimal-linked binaries; inspect unneeded runtime/framework linkage, dependencies and binary size.
- Preserve compile/lint/link/import **versus executed** evidence distinction; keep required iOS device, simulator, signing, permissions, entitlement and real-background validations open.
- Maintain independent C11/C++17 ABI consumer compile/link, symbol/version/header parity, callbacks, no unwind through FFI, pointer/provenance/ownership and async cancellation.
- Parity harness `xtask parity` currently unavailable until Apple reference and Rust candidate suite exist; do not claim parity or performance speedups from scaffolding.
- Validate Xcode 27.x support separately from earlier Xcode 26.6 archive smoke.
- Preserve documentation freshness, capability matrix, source/ABI/public-API audits and relevant hardware benchmark evidence.

## Scope ownership and tests
R2 exclusively owns xtask/CI; capability workstreams own their package-level semantic tests. Run `cargo +1.94.1 fmt --all -- --check`, workspace locked check/tests/strict Clippy, no-std checks and link probe, docs-check, zero-swift-source, target-linked checks, examples and archive smoke as available. Final independent audit reviews current main, feature isolation, linked frameworks, unsafe assumptions, unsupported rows, full diff and physical-device evidence. Report commands, platform, checks, skips, blockers and SHA; no fabricated successes.
