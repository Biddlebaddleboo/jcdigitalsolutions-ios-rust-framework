# PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md — Workstream D47: CloudKit Account-Status Contract

## Objective

Add one portable, owned snapshot contract for CloudKit account status. Do not add CloudKit database, record, query, subscription, sync, or account-change behavior.

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 conventions for a portable `#![no_std]` facade and static backend selection are integrated
- B52 provides the separate iOS adapter

## Write scope

- `Cargo.toml` and `Cargo.lock` for workspace dependency/package resolution
- `crates/framework-cloud/**`
- `docs/capabilities/cloudkit-account-status.md`
- D47 row in `docs/capabilities/capability-status.json` and related support-summary counts
- this plan and the D47 reference in `PLAN_CAPABILITIES.md`

Do not add an account database or user identifier API, global runtime, native type to the portable crate, or Swift source. CloudKit data APIs and later account-change observation need separate named workstreams.

## Required contract

- Add an independently usable `framework-cloud` crate with `#![no_std]`, no third-party dependency, no allocation requirement, and `#![forbid(unsafe_code)]`.
- Preserve the documented account-status cases as semantic Rust variants and retain a future raw value in fixed-width `i64` form.
- Return an owned, copyable status snapshot with an optional framework-owned error. Do not retain `NSError`, its domain, description, or any borrowed callback memory.
- Expose a statically selected `CloudAccountBackend`, associated `core::future::Future`, explicit availability, and caller-owned `CloudAccount<B>` facade. Require no global lookup, service registry, or executor.
- Start the selected backend operation on first poll; define exactly-once completion and backend-specific future-drop behavior. The facade itself does not assert native cancellation.
- State that one status snapshot is time-local, does not watch account changes, and does not prove access to any CloudKit database or data.

## Validation

- `cargo check --locked -p framework-cloud --no-default-features`
- `cargo clippy --locked -p framework-cloud --all-targets --no-default-features -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `git diff --check`
- Do not add or run tests in this slice.

## Acceptance boundary

The portable contract is complete when it compiles without default features, exposes no Apple/platform dependency types, and documents ownership, error, and operation semantics. It does not claim Apple parity or live iCloud-account behavior.

## D47 evidence — 2026-10-10

The portable `framework-cloud` contract and row 076 were already present at the base. This slice made the `CloudAccountBackend` sizing and exactly-once completion contract explicit, documented result discard versus cancellation on future drop, and added the D47 reference to `PLAN_CAPABILITIES.md`. Row 076 remains `implemented` for the portable contract and `partial` for iOS; its support summary matches the 114 rows, 36 implemented portable contracts and 22 partial portable contracts. `framework-cloud` depends only on workspace crate `framework-core`; no allocation or Apple/platform type appears in its portable source.

- `cargo check --locked -p framework-cloud --no-default-features` — PASS
- `cargo clippy --locked -p framework-cloud --all-targets --no-default-features -- -D warnings` — PASS
- `cargo fmt --all -- --check` — PASS
- `cargo xtask docs-check` — PASS; shared documentation index and zero-Swift checks passed
- `git diff --check` — PASS
- No tests were added or run. The all-target Clippy command compiled applicable targets only. No live account probe, Apple parity, or CloudKit data behavior is claimed.
