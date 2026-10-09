# PLAN_CAPABILITIES_HEALTH_AUTHORIZATION.md — Workstream D26: Portable Health Authorization Contract

## Objective

Define a bounded portable `no_std` contract for checking HealthKit data-store availability and explicitly requesting type-scoped read/share authorization. This contract does not expose authorization state or access health samples.

## Scope

- `crates/framework-health-authorization/**`
- `docs/capabilities/health-authorization.md`
- this named subplan

Do not edit the root workspace configuration or lockfile, shared capability manifest/counts, CI, aggregate plans, docs indexes, `tools/xtask`, C bindings, or unrelated workstreams. Root owns workspace/lock/index/capability reconciliation after handoff.

## Required semantics

- Use a genuine `#![no_std]` contract with no third-party dependency and no Apple types.
- Represent HealthKit type families and borrowed identifiers in framework-owned values. Separate read and share lists and reject empty identifiers, NUL identifiers, empty requests, and non-sample share types.
- Model request acceptance separately from its asynchronous completion. Completion success means only that native request processing reported success; it is not permission.
- Do not model a read grant or share grant. HealthKit hides read denial, and its authorization-status API reports sharing status only.
- Use a statically selected backend trait, no boxed backend, registry, global state, hidden initialization, or executor.
- Keep HealthKit queries, writes, clinical records, document/series types, per-object authorization, observers, and background delivery out of scope.

## Documentation and validation

Document the borrowed request ownership, type-family limits, status privacy caveat, and all omitted operations. Add deterministic value/validation tests and a static fake backend. Verify `cargo test -p framework-health-authorization`, `cargo check -p framework-health-authorization --no-default-features`, formatting, and `git diff --check`.

## Status and evidence

Implemented in isolated worktree `/private/tmp/d26-healthkit-worktree` from base `7b5513fa2a4864d21a594cbf1fbd43951427155`. Five focused tests pass; the portable crate passes `--no-default-features`. No root integration or commit is included in this handoff.
