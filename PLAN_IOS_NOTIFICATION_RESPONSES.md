# PLAN_IOS_NOTIFICATION_RESPONSES.md — Workstream B12: Local Notification Responses

## Goal

Map native iOS local-notification response data into D9 `NotificationResponse` values with an opt-in Rust bridge and no Swift source

## Needs

Requires D9 `PLAN_CAPABILITIES_NOTIFICATION_RESPONSES.md`, the D3 notification ID contract, and B4 `ios-notifications`. B12 uses a separate crate so it does not alter B4 paths or schedule rules

## Write scope

- `platform/ios/ios-notification-responses/**`
- `docs/ios/notification-responses.md`
- `PLAN_IOS_NOTIFICATION_RESPONSES.md`
- one B12 link in `PLAN_IOS_NATIVE.md`
- workspace lock metadata only if Cargo needs it

Do not edit `platform/ios/ios-notifications/**`, D3/D9 contracts, the shared capability manifest, C or Swift ABIs, Swift source, or unlisted iOS backends

## Required work

- Use `UNUserNotificationCenterDelegate.userNotificationCenter(_:didReceive:withCompletionHandler:)` only; omit `willPresent` and app-settings methods
- Add an explicit `IosNotificationResponses::install` API; if the shared center has a delegate, return a stable error and do not replace it
- Keep a strong `Retained` handle for the Rust delegate while the center holds only a weak reference
- Call install before app launch completes and serialize it with all reads or writes of the shared center delegate. Apple marks this property weak and non-atomic, and offers no compare-and-set API, so unsynchronized outside writes can race the check and set
- Store the app closure in delegate-owned `Arc` state and require `Send + Sync + 'static`; Apple docs do not name a callback queue
- Call the closure on the native method thread, with no main-thread promise
- Map request IDs and action IDs exactly into D9 validation; map Apple's default and dismiss IDs to `Default` and `Dismiss`, and map custom IDs and `UNTextInputNotificationResponse.userText` to D9's custom-action and text-input values
- Suppress invalid native strings without edits to their content and still call native completion
- Ignore remote-push response values via `UNPushNotificationTrigger`; add no APNs calls, push API, or action/category setup
- Call native completion once on every method path, also for bad-value suppression and caught Rust panic
- Catch Rust panics at the Objective-C boundary
- Retain `self` and closure state across sync reentrancy. Drop of the public handle releases its strong delegate ref and must not clear a delegate set by app code after this install

## Local API audit

Xcode 26.6 with iPhoneOS 26.5 SDK headers marks `UNUserNotificationCenter`, `UNUserNotificationCenterDelegate`, and `didReceiveNotificationResponse` as iOS 10.0 APIs; Apple docs require delegate set before app launch completes and native completion after response work

Apple does not state a callback queue in the relevant pages. Do not claim main-queue or background-queue use

Local objc2 0.3.2 bindings mark `delegate` weak and gate the response method on `UNNotificationResponse` plus `block2`. The crate uses only `block2`, `UNNotification`, `UNNotificationRequest`, `UNNotificationResponse`, `UNNotificationTrigger`, and `UNUserNotificationCenter`; `UNNotificationTrigger` permits the `UNPushNotificationTrigger` check

## Not in scope

- No permission API, schedule/cancel change, action/category registration, foreground presentation method, app-settings method, remote push/APNs/PushKit implementation, notification service extension, Swift source, live prompt/response, or test add/run
- No replacement or chain of a present app delegate
- No promise of delivery, response order across app states, callback queue, UI-thread context, or response life beyond the sync method call

## Checks and handoff

- Add or run no tests
- Run iOS device and simulator `cargo check`, strict Clippy with `-D warnings`, `cargo fmt --all -- --check`, `cargo xtask docs-check`, zero-Swift source check, and `git diff --check`
- Report changed paths, local commit SHA, exact checks, public API, queue/lifetime limits, Apple sources, deviations, and open assumptions; do not push
