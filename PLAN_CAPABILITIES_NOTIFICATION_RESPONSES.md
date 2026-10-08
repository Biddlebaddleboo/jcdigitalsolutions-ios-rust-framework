# PLAN_CAPABILITIES_NOTIFICATION_RESPONSES.md — Workstream D9: Notification Response Values

## Objective

Add portable owned values for the kind and data of a local-notification interaction response. This is a value-only extension to D3; B12 owns native response delivery.

## Dependencies

- D3's `framework-notifications` contract is integrated.
- Reuse D3's validated `NotificationId`; do not change its scheduling behavior.
- B4's iOS local-notification backend exists, but D9 adds no iOS implementation or dependency on its native types.
- B12 response delivery remains a separate workstream.

## Write scope

- `PLAN_CAPABILITIES_NOTIFICATION_RESPONSES.md`
- `PLAN_CAPABILITIES.md` for this decomposition entry
- `crates/framework-notifications/src/lib.rs` to expose the additive response module
- `crates/framework-notifications/src/response.rs`
- `docs/notification-responses.md`
- `docs/notifications.md` only for a link and a corrected scope note

Do not edit D3 scheduling values, trigger behavior, backend methods, the capability support manifest, B4 or B12 backends, iOS code, C ABI, or Swift ABI code.

## Required contract

- Add a `response` module with an owned `NotificationResponse` that reuses D3's validated `NotificationId`.
- Represent standard/default, dismiss, custom-action, and text-input interaction kinds without exposing operating-system constants or types.
- Custom-action and text-input values carry an owned, exact action identifier. Reject an empty identifier or one containing NUL; do not normalize or truncate it.
- Text-input response data owns its UTF-8 `String`, preserves its exact contents, and permits an empty value. Do not trim, normalize, or reject NUL in user text.
- Default and dismiss variants have no action identifier or text-input payload. Text-input variants carry both action identifier and text.
- Make move/borrow behavior explicit in constructors and accessors. Keep values cloneable only as ordinary owned Rust values; do not add shared global state.
- Use `no_std` and `alloc` only where required. Add no dependency, executor, callback, backend, delivery method, permission method, category/action configuration, push API, or native type.
- Keep D3 schedule/cancel semantics unchanged. B12 remains responsible for delivery, callback threading, lifecycle, and native response translation.

## Explicit non-goals

- No response delegate, event stream, polling API, callback, or operation/backend trait.
- No iOS `UNNotificationResponse`, action/category registration, response delivery, remote push/APNs/PushKit, or notification center change.
- No C/C++/Python API and no changes to the shared support manifest.
- No tests may be added or run for this bounded change.

## Validation and handoff

- Run `cargo check -p framework-notifications --no-default-features`.
- Run strict Clippy for `framework-notifications` with warnings denied; do not execute tests.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, and `git diff --check`.
- Report changed files, commit SHA, public values and validation semantics, exact checks, scope adjustments, and limits.
