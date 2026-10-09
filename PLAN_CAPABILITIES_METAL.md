# PLAN_CAPABILITIES_METAL.md — Workstream D36: Portable Metal Device-Presence Snapshot

## Objective

Add a small portable `no_std` contract for a one-time report of whether the selected backend can obtain a system-default Metal device object. This is a partial facet of row 058, not a GPU compute or rendering API.

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 portable API conventions are available
- The iOS implementation is a separate B41 workstream

## Write scope

- `crates/framework-metal/**`
- `docs/capabilities/metal.md`

Do not edit the root workspace manifest, canonical capability status manifest, aggregate plans, CI, documentation indexes, or iOS backend crates. The lockfile may change only for package/dependency resolution required by these workspace-member crates. The orchestrator owns workspace integration and capability status.

## Required contract

- Build as a portable `#![no_std]` crate with no third-party dependencies
- Expose only framework-owned `MetalDevicePresence` values; do not expose Apple or Metal types
- Distinguish `Unknown`, `Present`, and `Absent` in a fixed-width value
- Expose a caller-owned static `MetalDevicePresenceBackend` and thin `MetalDeviceQuery<B>` facade
- Make `snapshot` synchronous, point-in-time, and free of global lookup, hidden initialization, dynamic dispatch, or executor requirements
- State that device presence does not establish GPU-work support, feature support, performance, or completed work
- Add deterministic fake-backend coverage for all status variants and facade ownership

## Validation and handoff

- Run `cargo fmt --all -- --check`, `cargo test -p framework-metal`, `cargo check -p framework-metal --no-default-features`, and `git diff --check`
- Inspect the public API for `std`, platform types, allocation, dynamic dispatch, hidden initialization, and broader GPU claims
- Report changed files, exact checks, limitations, and whether row 058 remains partial
