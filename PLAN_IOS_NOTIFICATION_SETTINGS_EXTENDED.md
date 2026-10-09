# PLAN_IOS_NOTIFICATION_SETTINGS_EXTENDED.md — Workstream B91: Additional Raw Notification Settings

## Objective

Audit whether a bounded iOS-only snapshot of additional public `UNNotificationSettings` values adds useful information beyond B88, and expose the values without changing D3, B4 authorization normalization, or B88

## Scope

- Add `IosNotificationsBackend::notification_settings_extended_raw_values`
- Return raw signed `NSInteger` values for `notificationCenterSetting` and `lockScreenSetting`
- Return separate `Option<isize>` values for `criticalAlertSetting`, `timeSensitiveSetting`, and `scheduledDeliverySetting`
- Use `respondsToSelector:` for every getter newer than the package iOS 10.0 floor before calling it
- Preserve unknown values inside `Some(raw)`; use `None` only when the runtime settings object lacks that selector
- Read all fields from one prompt-free `getNotificationSettingsWithCompletionHandler:` callback

## Binding and availability audit

Xcode 26.6 (17F113) iPhoneOS SDK 26.5 headers mark `UNNotificationSettings`, `notificationCenterSetting`, `lockScreenSetting`, and the settings-query selector as available from iOS 10.0. `criticalAlertSetting` is available from iOS 12.0. `timeSensitiveSetting` and `scheduledDeliverySetting` are available from iOS 15.0. objc2-user-notifications 0.3.2 exposes each getter as a typed method returning `UNNotificationSetting(pub NSInteger)`; the tuple wrapper preserves unrecognized signed values

The binding methods do not encode these per-property deployment floors. B91 therefore checks the native settings object with `respondsToSelector:` for each iOS 12+ or iOS 15+ getter before calling it. `None` means the selector is absent, which is distinct from `Some(0)` (`UNNotificationSettingNotSupported`)

## Semantics, entitlement, and privacy

The notification-center and lock-screen values describe app-specific presentation settings, not whether a particular notification appears. Apple notes that disabling lock-screen notifications does not rule out on-screen presentation while unlocked. Time-sensitive and scheduled-delivery values report settings only; they do not promise an interruption level, delivery time, or actual delivery

Apple documents that critical alerts require a special entitlement to play critical sounds. B91 reads only the current `criticalAlertSetting`; it does not inspect provisioning, request critical authorization, construct critical-sound content, or claim that B4 can use critical alerts. The callback returns the calling app's notification settings and no notification IDs, content, or delivery history; it does not prompt

## Validation record

B91 adds an iOS-only result type, future, selector-guarded callback, guide text, compile/link probe selection, and source assertions for the newer selector guards. On 2026-10-09, locked device and Simulator checks, strict all-target Clippy, device and Simulator rustdoc, formatting, `xtask docs-check`, `xtask zero-swift-source`, shell syntax, scoped diff check, and the Release link/import gate passed. The link gate built and inspected both probes but did not execute them. No tests, app execution, live settings query, or prompt ran
