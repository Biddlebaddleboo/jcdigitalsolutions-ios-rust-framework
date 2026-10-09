# PLAN_CAPABILITIES_CLIPBOARD.md — Workstream D5: Portable Clipboard Contract

## Status

D5's portable clipboard contract and guide are implemented in `framework-sharing`. Availability is
backend-reported, may be `Unknown`, does not guarantee a later operation will succeed, and has no
portable permission/privacy semantics. Six fake-backend tests cover owned reads/errors, writes,
clear, no work before first poll, read start on first poll, and pending read/write/clear result
suppression. Validation passes:
`cargo fmt --all -- --check`, `cargo test --locked --offline -p framework-sharing`,
`cargo check --locked --offline -p framework-sharing --no-default-features`,
`cargo doc --locked --offline -p framework-sharing --no-deps --no-default-features`, and
`git diff --check`.

## Objective

Add a small platform-agnostic Rust contract for plain-text clipboard read, write, and clear operations. This is one capability slice in `framework-sharing`; it does not implement native iOS access or share-sheet presentation.

## Dependencies

- Foundation A and D1 portable API conventions are integrated
- `framework-core` error and availability types are integrated
- a future iOS backend uses a separate named B subplan after D5 is integrated

Read first:
- `PLAN_CAPABILITIES.md`
- `AGENTS.md`
- `crates/framework-core/src/lib.rs`
- `crates/framework-location/src/lib.rs`
- `docs/capabilities/location.md`

## Write scope

- `crates/framework-sharing/**`
- `docs/capabilities/sharing.md`
- `Cargo.lock` only for central package resolution; the orchestrator reconciles it

Do not edit root workspace configuration, `tools/xtask`, the capability status manifest, D1-owned crates, iOS backend crates, Swift ABI, C bindings, or share-sheet APIs. Do not add platform-native or third-party dependency types to the portable surface.

## Required contract

- Implement as a portable `#![no_std]` crate; use `alloc` only for owned UTF-8 text.
- Define a statically selected backend contract and thin client for availability, read, write, and clear; no boxed trait object, global registry, executor, or hidden initialization.
- Read returns an owned `Option<String>`: `None` means no readable plain-text representation, while backend failure remains an explicit error.
- Write accepts plain UTF-8 text and clear removes the clipboard's plain-text value; do not claim exclusive access to shared clipboard state.
- State borrow, copy, allocation, and future-drop/cancellation semantics. A platform may finish a native operation after Rust drops its future; it must detach safely and suppress late results.
- Keep rich text, images, files, arbitrary pasteboard representations, share UI, and platform permission/privacy behavior out of the portable contract.
- Keep foreign bindings and the shared support manifest for orchestrator reconciliation.

## Documentation

Record text-only scope, owned return values, copy/allocation behavior, shared-state races, cancellation, unsupported rich representations, and the fact that platform privacy/UI behavior belongs to the native backend guide.

## Validation and handoff

- Add deterministic contract tests with a fake backend for read, write, clear, and cancellation/result semantics.
- Run `cargo fmt --all -- --check`, `cargo test -p framework-sharing`, `cargo check -p framework-sharing --no-default-features`, and `git diff --check`.
- Audit the portable API for `std`, platform/dependency types, dynamic dispatch, hidden initialization, and unrelated dependencies.
- Restore `Cargo.lock` before the isolated commit; root will add the workspace lock entry and portable `no_std` gate.
- Report changed files, commit SHA, exact checks/results, deviations, and unresolved assumptions. Do not push.
