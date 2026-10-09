# PLAN_CAPABILITIES_CLASSKIT.md — D81 ClassKit Deep-Link Marker

## Scope

Row `080-cloud-accounts-communication-classkit` gets one iOS-only, non-data query:
`is_classkit_deep_link(&NSUserActivity) -> bool`. It reads the caller-owned activity's
`isClassKitDeepLink` marker and returns only that Boolean. It is not a portable ClassKit contract
and does not represent the full ClassKit capability.

Out of scope: `contextIdentifierPath`, `CLSDataStore`, `CLSContext`, `CLSActivity`, context or
assignment lookup, progress reporting, class/student identity, entitlement inspection, context
provider extensions, app/Schoolwork lifecycle integration, activity creation, and Swift wrappers.

## SDK and binding evidence

The installed Xcode 26.6 build `17F113` iPhoneOS SDK 26.5 declares
`NSUserActivity.isClassKitDeepLink` as a read-only `BOOL` property in
`ClassKit.framework/Headers/NSUserActivity+CLSDeepLinks.h`, available from iOS 11.3. The header
declares no prompt, permission, usage-description key, or entitlement on this getter.

The generated `objc2-class-kit` 0.3.2 binding provides the exact typed category method as
`unsafe fn isClassKitDeepLink(&self) -> bool` on the sealed
`NSUserActivityCLSDeepLinks` trait, enabled by `NSUserActivity_CLSDeepLinks`. Its feature closure
enables only the Foundation `NSArray`, `NSString`, and `NSUserActivity` types needed by the
category. The implementation wraps the read-only getter for a borrowed, live `NSUserActivity`,
documents the iOS 11.3 floor, and returns no native object.

## Entitlement and service boundary

Apple describes `isClassKitDeepLink` as a predicate for an incoming user activity from an
assignment in Schoolwork. This slice only reads that marker; it does not use the ClassKit data
store or read assignment content. Apple documents the separate
`com.apple.developer.ClassKit-environment` entitlement for an education app that shares data with
Schoolwork. Therefore the getter itself has no documented entitlement requirement, but a positive
Schoolwork assignment deep-link scenario belongs to a host that separately adopts ClassKit and
its capability. Do not claim this query makes an app ClassKit-enabled or proves assignment access.

The helper is synchronous, borrows the incoming `NSUserActivity`, performs no queue hop, and does
not retain the activity. Apple does not document a separate thread-safety guarantee for this
category property; callers must use the activity according to the host app's activity lifecycle.

## Scoped validation

`platform/ios/ios-system-services/check.sh` covers format, host/device/Simulator compile checks,
device/Simulator strict Clippy, iOS rustdoc, target dependency isolation, exact feature isolation,
and release link-import inspection. The `classkit_deep_link_probe` example is built for device at
iOS 11.3 and Simulator at iOS 14.0, inspected with `otool`, `nm`, `strings`, and `vtool`, and never
executed. No tests, runtime activity probes, ClassKit store calls, or assignment reads are part of
this scope.

## Apple and binding references

- [NSUserActivity.isClassKitDeepLink](https://developer.apple.com/documentation/foundation/nsuseractivity/isclasskitdeeplink?language=objc)
- [Linking directly to assignments](https://developer.apple.com/documentation/classkit/linking-directly-to-assignments)
- [ClassKit Environment Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.classkit-environment)
- [`objc2-class-kit` 0.3.2 binding](https://docs.rs/objc2-class-kit/0.3.2/objc2_class_kit/trait.NSUserActivityCLSDeepLinks.html)
- [Generated binding source](https://docs.rs/objc2-class-kit/0.3.2/src/objc2_class_kit/generated/NSUserActivity_CLSDeepLinks.rs.html)
