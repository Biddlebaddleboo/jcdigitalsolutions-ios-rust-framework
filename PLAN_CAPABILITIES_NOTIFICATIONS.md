# PLAN_CAPABILITIES_NOTIFICATIONS.md — Workstream D3: Local Notification Contract

## Objective

Add a portable Rust contract for local user notifications, with no iOS backend or remote-push claim.

## Dependencies

Requires `PLAN_FOUNDATION.md` and the integrated D1 capability conventions.
The contract must remain usable before a platform backend exists.

## Write scope

- `crates/framework-notifications/**`
- `docs/notifications.md`
- `Cargo.lock` only for central workspace resolution

Do not modify D1-owned crates, the global support manifest, iOS backend crates, Swift ABI code, or C ABI files.

## Required contract

- Provide owned, portable notification content and caller-supplied notification identifiers.
- Provide a bounded one-shot trigger model: immediate or an absolute Unix timestamp in milliseconds.
- Model authorization state without exposing `UNAuthorizationStatus` or another platform type.
- Expose a statically selected `NotificationBackend` and a thin client for availability, authorization query/request, schedule, and cancel operations.
- Use `no_std` plus `alloc` only as required; do not require a global executor or service registry.
- State future-drop, cancellation, replacement-by-ID, duplicate-ID, and exactly-once completion semantics in rustdoc.
- Preserve native escape hatches for a later iOS backend without adding Apple types to this portable crate.

## Explicit non-goals

- No iOS backend, `UserNotifications` linkage, permission prompt, or runtime delivery claim.
- No remote push/APNs/PushKit, notification response delegate, categories/actions, attachments, repeat/calendar triggers, critical alerts, or badge APIs.
- No C/C++/Python API.

## Validation and handoff

- Add contract tests for IDs, trigger bounds, authorization states, and fake-backend request/cancel behavior.
- Run `cargo fmt --all -- --check`, `cargo test -p framework-notifications`, `cargo check -p framework-notifications --no-default-features`, and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions.
