# PLAN_CAPABILITIES_TRANSFER.md — Workstream D10: Durable HTTP Downloads

## Status

D10's portable contract and guide are integrated. A follow-up audit made ID reuse, terminal metadata
retention, absent-ID behavior, and preservation of error kind/platform code explicit. Locked
no-default-features check, strict Clippy, formatting, docs, zero-Swift, and diff checks pass. A
read-only B13 spot-check found no contract mismatch. No tests were added or run.

## Objective

Add a portable durable HTTP file-download contract for B13. A static backend can retain task state across app relaunch; no executor or global registry is part of the API

## Dependencies

- D1 `framework-network` and `framework-files` contracts are integrated
- B13 uses the D10 contract and B14's separate `ios-files` temp-file adoption operation

## Write scope

- `PLAN_CAPABILITIES_TRANSFER.md`
- `PLAN_CAPABILITIES.md` for this decomposition link
- `crates/framework-transfer/**`
- `docs/capabilities/transfer.md`

Do not edit B13 or B14 plans, `ios-files`, iOS backends, the shared capability manifest, or bindings

## Required contract

- Add a `no_std` + `alloc` `framework-transfer` crate with a compile-time-selected `TransferBackend`
- Add `TransferId(u128)` with nonzero validation; the app assigns each stable ID and retains it across app relaunch
- Treat an ID with a stored durable record as a conflict; do not replace its task; permit reuse only after terminal `forget`
- Add GET-only `DownloadRequest` with `TransferId`, HTTP(S) `HttpUrl`, borrowed request `Header` values, and destination `AppPath`
- Add synchronous `start_download`, `status`, `cancel`, and `forget` methods; `start_download` returns success only after durable task acceptance, not network start. `DuplicateId` leaves the existing task unchanged; any other error means this call accepted no new task record. Outcomes after acceptance are observed through `status`; a retained terminal failure uses `TransferStatus::Failed`
- Make a query after app relaunch return the last durable state and any retained terminal metadata until `forget`
- Represent queued, active, succeeded, failed, and cancelled states; success holds HTTP `StatusCode` and owned `ResponseHeader` values
- Treat HTTP status as result metadata; a non-2xx response remains a completed HTTP response
- Make `cancel` a durable stop request, not proof that the task stopped; status may stay queued/active until terminal state; a terminal result is a no-op success and may win a race
- Permit `forget` only for terminal state; remove the task record, not a completed destination file
- Require a full-file atomic destination commit before success; readers never see partial new bytes; failed/cancelled tasks leave the prior whole file or no file; atomic commit does not promise crash durability
- Keep D1 validation; document that native response-header order and duplicate fidelity may be weaker than the portable vector form
- Add no upload, method override, inline body, progress stream, executor, global registry, callback, permission API, native API, or manifest claim
- Add no tests and run no tests for this bounded change

## Explicit boundaries

- `HttpUrl` checks scheme, authority presence, and forbidden ASCII whitespace/control bytes; it does not normalize or fully parse a URL
- Request `Header` values retain app order and duplicate names; the backend must copy data needed past `start_download` return
- Response `ResponseHeader` values use D1 validation and owned byte storage; a backend must document any normalization, omission, or order/duplicate limits in its native source
- `AppPath` remains a semantic app directory plus relative path; the backend must enforce sandbox containment and its symlink policy
- A completed download's body resides only at its destination; terminal status retains the HTTP code and observed response headers, not body bytes
- Durable task state means query after process/app relaunch; it does not imply power-loss durability or background continuation after platform policy cancels work
- If a backend cannot meet durable state or atomic destination semantics, its availability must report `Availability::Unsupported`; a request-specific limit may return `framework_core::ErrorKind::Unsupported` before task acceptance

## Validation and handoff

- Run no-default-features `cargo check` for `framework-transfer`
- Run strict Clippy for `framework-transfer` with warnings denied; do not execute tests
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Report public API, exact checks, commit SHA, scope changes, limits, and any B13 contract mismatch
