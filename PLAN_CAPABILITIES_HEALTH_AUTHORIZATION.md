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

Revalidated against current root commit `6e388c4` (`fix(calendar): clarify lazy auth contract`); D26 implementation files are already present there. On Rust `1.94.1`, these focused checks pass:

- `cargo +1.94.1 fmt --all -- --check`
- `cargo +1.94.1 test --locked --offline -p framework-health-authorization` — five unit tests pass; zero doctests
- `cargo +1.94.1 check --locked --offline -p framework-health-authorization --no-default-features`
- `cargo +1.94.1 tree --locked --offline -e normal -p framework-health-authorization --depth 4` — only the internal `framework-core` dependency; no third-party dependency
- `git diff --check`

The portable contract is `#![no_std]`; its API uses borrowed identifiers and separate read/share slices, validates empty/NUL identifiers, empty requests, and non-sample share types, distinguishes request acceptance from completion, and exposes no permission/grant state. HealthKit query/write/sample access, privacy-sensitive clinical records, observers, and background delivery remain excluded. These checks do not establish native HealthKit runtime behavior, permission UI, entitlement validity, or sample access.
