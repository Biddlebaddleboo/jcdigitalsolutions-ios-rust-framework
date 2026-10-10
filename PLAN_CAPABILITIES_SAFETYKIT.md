# PLAN_CAPABILITIES_SAFETYKIT.md — D48: Crash Detection Availability Bit

## Objective

Expose only the point-in-time SafetyKit Crash Detection device-support bit through a direct iOS Rust API. No portable contract is useful for this hardware- and OS-specific predicate

## Public boundary

- `ios-safety::is_crash_detection_available() -> bool` calls only `SACrashDetectionManager.isAvailable`
- Return `false` below the iOS 16.0 class API floor
- Do not request authorization, construct `SACrashDetectionManager`, set a delegate, receive `SACrashDetectionEvent`, or access `SAEmergencyResponseManager`
- Do not claim app authorization, entitlement access, emergency readiness, or live event delivery
- Treat the result as a point-in-time device capability bit, not a guarantee of event access

## API and access caveat

The local iPhoneOS26.5 SDK declares `SACrashDetectionManager` with `API_AVAILABLE(ios(16.0), macos(13.0), watchos(10.1))`, `API_UNAVAILABLE(tvos)`, and class getter `isAvailable`. The generated `objc2-safety-kit` 0.3.2 binding exposes `unsafe fn isAvailable() -> bool` behind `SACrashDetectionManager`

The local header says `SACrashDetectionManager` requires an Apple entitlement. Apple docs say Crash Detection event access requires `com.apple.developer.severe-vehicular-crash-event`; the docs do not state whether the static availability getter alone has this prerequisite. This slice makes no claim that the app has the entitlement or event authorization. An app that needs event access must obtain the entitlement and request user authorization through a separate, future scope

The method reads one Boolean from SafetyKit. It does not prompt for permission or read, persist, or expose crash event data. The getter's behavior without the Crash Detection entitlement is not verified here

## Dependencies and validation

- Use only the `objc2-safety-kit` 0.3.2 `SACrashDetectionManager` feature
- The direct import gate expects `SafetyKit`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; reject all other imports and Swift runtime symbols
- Device and simulator compile, strict Clippy, rustdoc, and link/import checks prove source and link shape only; no test or probe execution, entitlement, authorization, prompt, event, or hardware result claim

## D48 closeout — 2026-10-10

The `ios-safety::is_crash_detection_available() -> bool` API meets this slice: it checks the iOS 16.0 availability floor and makes one `SACrashDetectionManager::isAvailable` call. No source edit was needed. `.github/workflows/ci.yml` runs `sh platform/ios/ios-safety/scripts/check.sh` on macOS

`sh platform/ios/ios-safety/scripts/check.sh` passed on Rust 1.94.1, Xcode 26.6 (build 17F113), and iPhoneOS SDK 26.5. Gates: source guards, fmt, host/device/Simulator checks, strict Clippy, rustdoc, release probe artifact build plus import/symbol/string audit, docs-check, zero-Swift, and diff checks. The link audit found the expected `SafetyKit`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` imports and no Swift runtime symbols. Probe artifacts were built and inspected, not run

This evidence shows source, compile, and link shape only. The getter-specific entitlement prerequisite remains unknown. No claim is made about app authorization, entitlement access, emergency readiness, event delivery, or a live hardware result; no authorization, prompt, event, or emergency-response path is present

## Files

- `platform/ios/ios-safety/**`
- `docs/ios/safetykit.md`
- `docs/VALIDATION.md`
- `PLAN_IOS_SAFETYKIT.md`
- `PLAN_CAPABILITIES_SAFETYKIT.md`
- `.github/workflows/ci.yml`
- `docs/capabilities/capability-status.json` row `073-personal-data-system-stores-safetykit` only
