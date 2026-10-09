# PLAN_IOS_NOTIFICATION_SETTINGS_SURFACES.md — Workstream B94: CarPlay and Announcement Settings

## Audit outcome

B94 adds raw CarPlay and Siri-announcement setting snapshots from one prompt-free `UNNotificationSettings` callback. It audits but does not expose `showPreviewsSetting`

## Binding and availability audit

Xcode 26.6 (17F113) iPhoneOS SDK 26.5 marks `UNNotificationSettings` and `carPlaySetting` at iOS 10.0. `announcementSetting` is available at iOS 13.0. objc2-user-notifications 0.3.2 exposes both as typed methods returning `UNNotificationSetting(pub NSInteger)`. CarPlay needs no availability guard because it is at the backend's iOS 10.0 floor. The iOS 13+ announcement getter is checked with `respondsToSelector:` before call; `None` means the selector is unavailable, while `Some(raw)` preserves every signed value including unknown values and `Some(0)` (`NotSupported`)

`showPreviewsSetting` is available from iOS 11.0 and is typed as `UNShowPreviewsSetting(pub NSInteger)`; its defined raw values are 0 (`Always`), 1 (`WhenAuthenticated`), and 2 (`Never`). It is not included in B94

## Semantics, privacy, and entitlement scope

Apple describes `carPlaySetting` as whether this app's notifications appear in CarPlay, and `announcementSetting` as whether Siri can announce this app's notifications. These are app-specific settings only. B94 queries neither CarPlay connectivity nor Siri hardware/availability, does not display or announce anything, and does not infer actual presentation. The SDK getter declarations add no entitlement requirement for these reads; B94 adds no separate CarPlay/Siri framework or entitlement integration

The CarPlay and Siri fields reveal per-app settings choices, not notification content, IDs, or delivery history. Callers must not treat either field as proof of a connected vehicle, available Siri, or successful presentation

`showPreviewsSetting` directly reports the user's notification-content preview privacy choice. B4 submits title/body content to UserNotifications and does not build a preview UI or alter system presentation; reading the value adds no required local-scheduling behavior, while the system already enforces the setting. B94 therefore leaves the preview preference unexposed

The callback does not prompt or mutate portable D3/B4 behavior

## Validation record

B94 adds an iOS-only raw settings result type and future, runtime selector guard, compile/link probe selection, focused guide note, and source assertion for the iOS 13 selector. On 2026-10-09, locked device and Simulator checks, strict all-target Clippy, device and Simulator rustdoc, package formatting, `xtask docs-check`, `xtask zero-swift-source`, shell syntax, scoped diff check, and the Release link/import gate passed. The workspace-wide `cargo +1.94.1 fmt --all -- --check` also reported a formatting delta in `platform/ios/ios-accessibility/src/platform.rs`, outside this B94 scope; B94's package-only formatter check passed. The link gate built and inspected both probes but did not execute them. No tests, app execution, live settings query, or prompt ran
