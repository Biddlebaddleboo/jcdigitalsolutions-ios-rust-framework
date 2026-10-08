# PLAN_IOS_NOTIFICATIONS.md — Workstream B4: Local Notification Backend

## Objective

Implement the D3 local-notification contract with public iOS UserNotifications APIs reached from Rust, without a Swift application/source layer.

## Dependencies

Requires integrated `PLAN_FOUNDATION.md`, `PLAN_CAPABILITIES_NOTIFICATIONS.md`, and the `framework-notifications` contract.

## Write scope

- `platform/ios/ios-notifications/**`
- `docs/ios/notifications.md`
- `Cargo.toml` and `Cargo.lock` only for a reviewed, capability-local workspace dependency

Do not change the portable D3 contract, shared core semantics, other iOS backends, C ABI, or Swift ABI.

## Required implementation

- Use `UNUserNotificationCenter` for authorization query/request, one-shot schedule, and pending-request cancellation.
- Map native authorization states to `framework_core::AuthorizationState`; document any information loss for provisional or ephemeral states.
- Preserve D3 semantics for caller-supplied IDs, same-ID replacement, immediate or absolute Unix-millisecond triggers, future drop/detach, and cancel results.
- Start no permission prompt or scheduling work during backend construction. `request_authorization` may prompt only when polled.
- Keep native callbacks safe and exactly-once; do not unwind across an Objective-C block boundary.
- Expose a narrow native notification-center escape hatch under the iOS backend.
- Record minimum OS availability, permission/prompt behavior, Info.plist requirements, lifecycle, errors, and dependency feature rationale.

## Non-goals

- No remote notifications/APNs/PushKit, response delegate, categories/actions, badges, attachments, repeats, or calendar-recurring schedule API.
- No live user prompt, simulator delivery, or device-delivery claim in automated checks.
- No `.swift` source, C ABI surface, global notification registry, or mandatory executor.

## Validation and handoff

- Add portable/unit coverage for native status and trigger conversion where it can run without an iOS user prompt.
- Run format, package tests, device/simulator `cargo check` and Clippy with `-D warnings`, plus a Release import audit for the backend.
- Confirm linkage is limited to the required public Apple frameworks and has no Swift runtime import.
- Run `cargo xtask docs-check` and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, runtime limits, and unresolved assumptions.
