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

## D36 reconciliation evidence — 2026-10-10

Reconciled the existing implementation at base `24f46eab43434aca855bd0f5f6fdf84f01246e44`. The required portable contract was already present, so no product source or capability documentation change was needed:

- `crates/framework-metal` is `#![no_std]`, forbids unsafe code, and declares no third-party dependencies.
- `MetalDevicePresence` is a framework-owned `#[repr(u8)]` enum with fixed values for `Unknown`, `Present`, and `Absent`; its size is checked as one byte.
- `MetalDevicePresenceBackend` and `MetalDeviceQuery<B>` provide an explicitly supplied, caller-owned generic backend with direct static dispatch. `snapshot` is synchronous and delegates one point-in-time query without global lookup, allocation, an executor, or hidden initialization.
- The fake-backend test covers all three states and verifies facade ownership through `backend` and `into_backend`. `docs/capabilities/metal.md` limits `Present` to a returned default-device object and disclaims GPU-work, feature, performance, rendering, compute, and MetalKit support.
- Canonical capability row `058-graphics-gpu-metal-metalkit` marks the portable contract implemented but the full capability partial; B41 only reports a system-default device object.

Focused checks on 2026-10-10:

```text
tools/install-tools.sh --prefix "$PWD/target/ios-rust-tools" — PASS; pinned ios-rust-build and ios-rust-validate 0.1.0 installed
ios-rust-build --version --format json — PASS; source SHA 2289e6a73257b696f6ae5ecd61ee20fd16ab8b37
ios-rust-validate --version --format json — PASS; source SHA 2289e6a73257b696f6ae5ecd61ee20fd16ab8b37
cargo fmt --all -- --check — PASS
cargo test -p framework-metal — PASS; 1 unit test, 0 doc tests
cargo check -p framework-metal --no-default-features — PASS
git diff --check — PASS
```

No Metal validation profile is registered in `validation-v1.json`, so no `ios-rust-validate --capability` run applies. No iOS backend, device/Simulator runtime, GPU work, rendering, compute, MetalKit, feature support, or performance behavior was tested or claimed. Row 058 remains partial.
