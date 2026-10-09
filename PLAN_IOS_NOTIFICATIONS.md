# PLAN_IOS_NOTIFICATIONS.md — Workstream B4: Local Notification Backend

## Status

B4 status: package-local Release link/import gate passed on 2026-10-09 for device and Simulator after Cargo.lock refresh; the probe executables were not run

B4's UserNotifications backend matches the plan; cancellation reports whether the ID appeared in the asynchronous request snapshot, not an atomic removal result. The guide records this best-effort query/remove behavior and the race with direct native-center changes. The guide records Xcode 26.6 with iPhoneOS SDK 26.5

B4 now also exposes `IosNotificationsBackend::pending_request_count`, an iOS-only future for the
app's `u64` local-request count at native callback time. It claims no authorization, delivery, or
readiness; the count may change after the native snapshot and has no ID or content

B86 extends B4 with `IosNotificationsBackend::authorization_status_raw_value`, an iOS-only
prompt-free settings future that preserves the signed native status value (including provisional,
ephemeral, and unknown future values). It is a status snapshot only, does not establish enabled
interactions or delivery/readiness, and leaves portable D3 normalization unchanged

B86 passed locked device and Simulator checks, strict all-target Clippy, rustdoc, and the Release
link/import gate on 2026-10-09. Xcode 26.6's iPhoneOS SDK 26.5 headers place the settings object,
`authorizationStatus`, and `getNotificationSettingsWithCompletionHandler:` at iOS 10.0; provisional
and ephemeral raw enum cases were added at iOS 12.0 and iOS 14.0. The probe binaries were built and
inspected but not executed; no tests, prompt, or live settings query ran

Row 031's canonical support reason now includes the iOS-only local-request count snapshot; the
portable D3 contract stays unchanged

B88 adds one prompt-free callback snapshot of raw alert, sound, and badge setting values, with unknown signed values preserved. It does not guarantee presentation, sound, badge updates, scheduling, or delivery; see `PLAN_IOS_NOTIFICATION_SETTINGS.md`

B91 adds raw Notification Center and Lock Screen settings plus optional critical-alert, time-sensitive, and scheduled-delivery values from the same prompt-free settings API. Newer selectors are checked before call; `None` means selector absent and `Some(0)` means `NotSupported`. This does not verify the critical-alert entitlement or guarantee notification behavior; see `PLAN_IOS_NOTIFICATION_SETTINGS_EXTENDED.md`

B94 adds raw CarPlay and optional Siri-announcement setting values from one prompt-free callback. It does not prove CarPlay connection, Siri availability, or actual presentation. The iOS 11+ preview-privacy setting was audited but is not exposed because B4 builds no preview UI and the system already enforces the choice; see `PLAN_IOS_NOTIFICATION_SETTINGS_SURFACES.md`

The pending-count follow-up passed root device and Simulator checks, strict all-target Clippy,
rustdoc, and the Release link/import gate. The compile-only example selects the new method, but no
tests, pending-request callback, or linked probe executable ran

This audit pass used Xcode 26.6 (17F113) and iPhoneOS SDK 26.5. These locked device and simulator checks pass

- `cargo +1.94.1 check --locked -p ios-notifications --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-notifications --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-notifications --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --no-deps -p ios-notifications --target aarch64-apple-ios --document-private-items`
- `cargo +1.94.1 doc --locked --no-deps -p ios-notifications --target aarch64-apple-ios-sim --document-private-items`
- `cargo +1.94.1 xtask docs-check`; `cargo +1.94.1 xtask zero-swift-source`; `cargo +1.94.1 fmt --all -- --check`; `git diff --check`

Release cdylib probes for device and simulator were built from a temporary crate under `target/ios-notifications-link-probe`
- `cargo +1.94.1 build --manifest-path target/ios-notifications-link-probe/Cargo.toml --release --target aarch64-apple-ios`
- `cargo +1.94.1 build --manifest-path target/ios-notifications-link-probe/Cargo.toml --release --target aarch64-apple-ios-sim`

- `otool -L target/ios-notifications-link-probe/target/aarch64-apple-ios/release/libios_notifications_link_probe.dylib`
- `otool -L target/ios-notifications-link-probe/target/aarch64-apple-ios-sim/release/libios_notifications_link_probe.dylib`
- `nm -u target/ios-notifications-link-probe/target/aarch64-apple-ios/release/libios_notifications_link_probe.dylib`
- `nm -u target/ios-notifications-link-probe/target/aarch64-apple-ios-sim/release/libios_notifications_link_probe.dylib`

The prior manual probe record reports only UserNotifications, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib` after the cdylib self path. `nm -u` reported 75 undefined symbols per target and no Swift, Python, UIKit, Core Location, APNs, or PushKit match. Those results remain historical evidence; the new package gate is a separate check, and neither probe set ran its binaries

A direct `cargo +1.94.1 rustc --locked -p ios-notifications --target aarch64-apple-ios --release -- --crate-type cdylib` attempt did not form a usable probe because its dependencies were not available in rlib form; the temporary crate dependency produced a valid link probe. The first `sh platform/ios/ios-notifications/check-link-imports.sh` attempt on 2026-10-09 exited 101 before either target build with `error: cannot update the lock file /Users/john/Projects/jcdigitalsolutions-ios-rust-framework/Cargo.lock because --locked was passed to prevent this`. After root refreshed Cargo.lock, the same command passed for `aarch64-apple-ios` and `aarch64-apple-ios-sim`. Both exact import lists were `Foundation`, `UserNotifications`, `libSystem.B.dylib`, and `libobjc.A.dylib`; both `nm -u` denylist scans passed. The linked binaries were inspected but not executed. G5 CI retains the target check and Clippy gates and now runs this import gate on macOS; no passing workflow run is recorded

No tests, live permission prompts, or notification delivery checks were run in this audit. The package gate provides link/import evidence only; its probe binaries were not executed and no runtime behavior is claimed

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

- Use `UNUserNotificationCenter` for authorization query/request, one-shot schedule, pending-request cancellation, and a local pending-request count snapshot.
- Map native authorization states to `framework_core::AuthorizationState`; document any information loss for provisional or ephemeral states.
- Preserve D3 semantics for caller-supplied IDs, same-ID replacement, immediate or absolute Unix-millisecond triggers, future drop/detach, and boolean cancel results. The iOS cancellation result is based on whether the ID appears in the asynchronous pending-request snapshot; UserNotifications exposes no atomic remove result, so query/remove is best-effort and concurrent native changes can race.
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
