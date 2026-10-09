# PLAN_CAPABILITIES_NFC.md — Workstream D32: Portable NFC Reader-Support Snapshot

## Objective

Add an honest `no_std` portable contract for a point-in-time NFC reader-support query. This slice does not implement NFC sessions or tag operations and must not be described as full NFC support.

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 portable API conventions are available
- The iOS implementation is a separate B37 workstream

## Write scope

- `crates/framework-nfc/**`
- `docs/capabilities/nfc.md`

Do not edit the root workspace manifest, canonical capability status manifest, aggregate plans, CI, documentation indexes, or iOS backend crates. The lockfile may change only for package/dependency resolution required by these workspace-member crates. The orchestrator owns workspace integration and capability status.

## Required contract

- Build as a portable `#![no_std]` crate with no third-party dependency beyond `framework-core`
- Expose framework-owned `NfcReaderAvailability` values only; do not expose Apple types
- Distinguish `Unknown`, `Supported`, and `Unsupported` in a fixed-width value
- Expose a caller-owned static `NfcReaderAvailabilityBackend` and thin `NfcReader<B>` facade
- Make `snapshot` synchronous, point-in-time, and non-prompting; require no global registry, hidden initialization, dynamic dispatch, or executor
- State that reader support does not establish permission, entitlement, usable session, scan success, tag presence, tag reads, or background support
- Add a deterministic fake-backend test for all status variants and facade ownership

## Validation and handoff

- Run `cargo fmt --all -- --check`, `cargo test -p framework-nfc`, `cargo check -p framework-nfc --no-default-features`, and `git diff --check`
- Inspect the public API for `std`, platform types, allocation, dynamic dispatch, hidden initialization, and broader NFC claims
- Report changed files, exact checks, limitations, and whether the broader row 040 remains unsupported
