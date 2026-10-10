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

## D32 reconciliation — 2026-10-09

- The portable snapshot contract is complete for this slice. `framework-nfc` remains `#![no_std]`, forbids unsafe code, and depends only on `framework-core`; its public contract is the one-byte `#[repr(u8)]` `NfcReaderAvailability`, `NfcReaderAvailabilityBackend`, and generic `NfcReader<B>` facade.
- `NfcReader<B>` owns the explicitly supplied backend and uses static generic dispatch. `snapshot` is synchronous and point-in-time; construction and query do not initialize a global service, prompt, start a session, scan, read a tag, or require an executor. No Apple type is in the portable crate.
- The enum distinguishes `Unknown = 0`, `Supported = 1`, and `Unsupported = 2`. Rustdoc and the capability guide state that the snapshot does not establish permission, usage-description or entitlement configuration, session readiness, scan success, tag presence or reads, or background NFC support.
- The existing deterministic fake-backend test covers all three statuses, facade forwarding, borrowed backend access, and returned backend ownership. Its name now describes those assertions. No new test behavior was needed.
- Rust 1.94.1 checks passed: `cargo +1.94.1 fmt --all -- --check`; `cargo +1.94.1 test --locked --offline -p framework-nfc` (1 passed); `cargo +1.94.1 check --locked --offline -p framework-nfc --no-default-features`; `cargo +1.94.1 tree --locked --offline -p framework-nfc` (only `framework-core`); and `git diff --check`.
- The pinned `ios-rust-build` and `ios-rust-validate` binaries reported version 0.1.0 and source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. `ios-rust-validate --list` has no D32 profile; it lists only the four current iOS pilot profiles. The source/API audit found no `std`, `alloc`, dynamic dispatch, global state, Apple types, or NFC session/tag API in `framework-nfc`.
- No native NFC query, hardware behavior, session, scan, or tag operation was run. Canonical capability row 040 remains `partial`; this slice does not claim full NFC session/tag support.
