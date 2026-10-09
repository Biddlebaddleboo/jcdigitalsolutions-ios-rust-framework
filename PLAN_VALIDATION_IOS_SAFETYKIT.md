# PLAN_VALIDATION_IOS_SAFETYKIT.md — G47: SafetyKit Crash Detection Availability

## Objective

Gate D48/B53's single `SACrashDetectionManager.isAvailable` query without claiming event access,
authorization, or emergency response.

## Required checks

- Run `sh platform/ios/ios-safety/scripts/check.sh` on macOS.
- The script checks formatting, host package check/strict Clippy/rustdoc, device and Simulator check/
  strict Clippy, device rustdoc, and the exact SafetyKit/Foundation/libSystem/libobjc import set.
- Run docs freshness, zero-Swift-source, and whitespace checks.
- Build and inspect device/Simulator probes only; never execute them or simulate an event.
- Do not add or run tests, request permission, sign an app, or validate entitlement admission.

## Evidence boundary

Compile, lint, and import results do not establish the getter's live hardware result or whether the
getter alone requires the class-level entitlement. Event authorization/delivery and emergency
response remain unverified. The inspected Xcode 26.6 / SDK 26.5 host is below the planned Xcode
27.x baseline.
