# PLAN_IOS_SAFETYKIT.md — B53: Crash Detection Availability Query

## Objective

Add one iOS-only Rust getter for SafetyKit's Crash Detection availability bit. Keep entitlement and authorization for event access out of this scope

## Scope

- `platform/ios/ios-safety/**`
- `docs/ios/safetykit.md`
- `PLAN_CAPABILITIES_SAFETYKIT.md`
- `docs/capabilities/capability-status.json` row `073-personal-data-system-stores-safetykit` only
- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

## API requirements

- Export only `is_crash_detection_available() -> bool`
- Guard `SACrashDetectionManager::isAvailable` with an iOS 16.0 availability check and return `false` below that floor
- Enable only the generated `SACrashDetectionManager` binding feature
- Do not instantiate a manager, request permission, inspect authorization status, retain a delegate, receive an event, or call an emergency-response API
- Do not add a portable contract, Swift source, main-thread requirement, app entitlement, or privacy usage key
- Document Apple’s class-level entitlement statement and the unspecified entitlement behavior of the getter itself; make no claim that the app has event entitlement or user authorization
- Do not describe the Boolean as a guarantee of event delivery or emergency response

## Evidence

The iPhoneOS26.5 SDK header sets the class API floor at iOS 16.0 and declares `isAvailable`. The local `objc2-safety-kit` 0.3.2 source binds it as `SACrashDetectionManager::isAvailable() -> bool`. Apple docs describe the property as a Boolean for Crash Detection availability and separately define the severe vehicular crash event entitlement

## Validation and limits

- Run device and simulator `cargo check --locked` and strict all-target Clippy for `ios-safety`
- Run host package check/Clippy, rustdoc, formatter, docs-check, zero-Swift, shell syntax, and diff checks
- Build but do not execute the device and simulator link probes; inspect exact direct imports with `otool -L`, Objective-C symbols with `nm -u`, and Swift-runtime strings
- Do not add or run tests, execute probes, request permission, create a delegate, or simulate/receive events
- These gates do not verify entitlement admission, getter behavior on hardware, app authorization, prompts, Crash Detection event delivery, or emergency action
