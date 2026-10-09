# ios-system-services

This package currently exposes one iOS-only ClassKit helper:
`is_classkit_deep_link(&NSUserActivity) -> bool`. It reads only the read-only
`NSUserActivity.isClassKitDeepLink` marker and returns a Rust `bool`. The caller owns the
`NSUserActivity`; the helper borrows it for the synchronous call and does not retain, store, or
transfer it.

The API floor is iOS 11.3. There is no prompt, callback, queue hop, context-path read, ClassKit
data-store access, assignment lookup, or portable facade. The returned marker identifies whether
Apple classifies that activity as a ClassKit context deep link. It is not an authorization check,
student identity, assignment state, or proof that the host can read ClassKit data.

The property itself has no documented permission, usage-description, or entitlement requirement.
An app that adopts ClassKit to share context and assignment data with Schoolwork uses the separate
`com.apple.developer.ClassKit-environment` entitlement and ClassKit capability. This package does
not access that data path.

Run the scoped no-test gate with:

```sh
sh platform/ios/ios-system-services/check.sh
```
