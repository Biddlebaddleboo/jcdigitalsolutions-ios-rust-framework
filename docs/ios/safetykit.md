# iOS Crash Detection availability

`ios-safety::is_crash_detection_available()` reads the point-in-time SafetyKit device-support bit from `SACrashDetectionManager::isAvailable`. It returns `false` below iOS 16.0

The result reports only whether SafetyKit says the current device supports Crash Detection. It does not report app authorization, entitlement admission, emergency readiness, or a guarantee of future event delivery. The call does not request permission, create a manager instance, set a delegate, or read event data

## Entitlement and privacy limit

The local SDK header states that `SACrashDetectionManager` requires an Apple entitlement. Apple identifies `com.apple.developer.severe-vehicular-crash-event` as the entitlement for Crash Detection event access. Apple docs do not state whether the static availability getter alone has the same prerequisite, and this crate does not claim an entitlement or authorization. An app that needs event access must handle entitlement admission and explicit authorization in a separate implementation

This API returns one Boolean only. It does not receive, persist, or expose `SACrashDetectionEvent` data. No permission prompt, event callback, live hardware result, or emergency-response action is part of this API

## API evidence

The inspected iPhoneOS26.5 SDK marks `SACrashDetectionManager` as available from iOS 16.0. The exact binding is `objc2-safety-kit` 0.3.2 with feature `SACrashDetectionManager`; its generated method is `unsafe fn isAvailable() -> bool`. The Rust wrapper checks the iOS 16.0 floor before that call. See Apple's [SACrashDetectionManager reference](https://developer.apple.com/documentation/safetykit/sacrashdetectionmanager) and the [Crash Detection entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.severe-vehicular-crash-event)

See [D48](../../PLAN_CAPABILITIES_SAFETYKIT.md), [B53](../../PLAN_IOS_SAFETYKIT.md), and [validation scope](../VALIDATION.md)
