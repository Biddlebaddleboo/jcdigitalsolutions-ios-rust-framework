# PLAN_IOS_NOTIFICATION_SETTINGS.md — Workstream B88: Raw Notification Setting Snapshot

## Objective

Expose the native alert, sound, and badge setting values from one prompt-free iOS `UNNotificationSettings` callback. Preserve the signed raw values without adding platform types to D3 or changing B4's portable authorization mapping

## Scope

- Add an iOS-only `IosNotificationsBackend::notification_setting_raw_values` future
- Return `alert`, `sound`, and `badge` as signed `NSInteger` values (`isize`) in `IosNotificationSettingRawValues`
- Preserve unknown values; known `UNNotificationSetting` raw values are 0 (`NotSupported`), 1 (`Disabled`), and 2 (`Enabled`)
- Read all three values from the same `getNotificationSettingsWithCompletionHandler:` callback
- Keep the query prompt-free and leave portable D3 and B86 authorization-status behavior unchanged

## Availability and limits

Xcode 26.6 (17F113) iPhoneOS SDK 26.5 headers place `UNNotificationSettings`, `alertSetting`, `soundSetting`, `badgeSetting`, and `getNotificationSettingsWithCompletionHandler:` at an iOS 10.0 floor. The three getters are available on iOS; their tvOS/watchOS exclusions do not affect this iOS-only API

The result is a callback-time settings snapshot, not a readiness, scheduling, or delivery result. An enabled value does not guarantee an alert is presented, a sound is played, or a badge changes. The API does not query other notification settings, start a prompt, request access, schedule or deliver a notification, or mutate D3

## Validation record

B88 added the raw settings future, public result type, compile/link probe selection, and guide text. On 2026-10-09, locked device and Simulator checks, strict all-target Clippy, device and Simulator rustdoc, formatting, `xtask docs-check`, `xtask zero-swift-source`, shell syntax, scoped diff check, and the Release link/import gate passed. The link gate built and inspected both probes but did not execute them. No tests, app execution, live settings query, or prompt ran
