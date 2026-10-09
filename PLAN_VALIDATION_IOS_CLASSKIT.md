# PLAN_VALIDATION_IOS_CLASSKIT.md — G73 ClassKit deep-link marker

## Scope

G73 validates D81/B74's iOS-only read of the caller-owned
`NSUserActivity.isClassKitDeepLink` Boolean. It does not query or read assignment data, context
identifiers, `CLSDataStore`, or user identity. The getter itself has no documented entitlement;
ClassKit/Schoolwork host adoption and the environment entitlement are separate.

## Gate

`sh platform/ios/ios-system-services/check.sh` passed formatting, host/device/Simulator checks,
strict device/Simulator Clippy, iOS rustdoc, dependency and feature isolation, and Release
link/import inspection.

Device imports are ClassKit, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`, with
`_objc_getClass` and `_objc_msgSend`; `vtool` reports minimum iOS 11.3 and SDK 26.5. Simulator
imports are the same and minimum iOS is 14.0. Probe strings contain only `NSUserActivity` and
`isClassKitDeepLink` from this capability. Neither probe was executed. Local evidence used Rust
1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the plan's Xcode 27.x baseline.

## Limits

The gate does not establish a live incoming activity, Schoolwork assignment navigation, ClassKit
data access, or runtime thread behavior. The package follows the host activity lifecycle and does
not retain the borrowed object.
