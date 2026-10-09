# PLAN_IOS_BACKGROUND_EXECUTION.md — Workstream B28: UIKit Background Execution

## Status

B28 implements one opt-in UIKit background-task lease for iOS app targets. It is separate from D20/B25 BGTaskScheduler app-refresh work.

## Objective

Implement D23's begin, cooperative expiry, and explicit end contract through `UIApplication` only.

## Dependencies

- D23 `framework-background-execution`
- Rust 1.94.1
- Locally available Xcode 26.6 build 17F113 and iPhoneOS/iPhoneSimulator SDK 26.5
- `objc2-ui-kit` 0.3.2 generated `UIApplication` bindings and `block2` 0.6.2
- `ios-runtime::main_thread::MainThread`

## Write scope

- `PLAN_IOS_BACKGROUND_EXECUTION.md`
- `platform/ios/ios-background-execution/**`
- `docs/ios/background-execution.md`

Root owns Cargo.lock, shared workspace manifests, capability JSON and counts, shared indexes, aggregate plans, CI, and aggregate validation docs. Do not edit those paths.

## Native scope and semantics

- Call only `UIApplication.sharedApplication`, `beginBackgroundTaskWithExpirationHandler:`, and `endBackgroundTask:` from UIKit.
- Use generated public bindings and `block2`; add no handwritten Apple ABI, Swift, or `.swift` source.
- Require `ios_runtime::main_thread::MainThread` for begin. Retain the `MainThreadMarker` in `IosBackgroundLease` so the lease is `!Send`/`!Sync` and explicit end or drop stays on its creation thread.
- Expose a cloneable atomic expiry signal that app work may poll from another thread. UIKit invokes the expiration handler synchronously on the main thread; it sets the signal and ends the UIKit identifier once. If the callback races the return from begin, the adapter ends immediately after the identifier becomes available.
- `IosBackgroundLease::end` and `Drop` share a one-shot state so UIKit receives at most one end call. End promptly when work is done; Drop is only a fallback.
- A missing `MainThread` proof is not accepted. UIKit's invalid identifier maps to `ErrorKind::Unavailable`.
- The selected API floor is iOS 4.0: `UIBackgroundTaskInvalid`, `beginBackgroundTaskWithExpirationHandler:`, and `endBackgroundTask:` are `API_AVAILABLE(ios(4.0))` in the active SDK header. The unnamed begin API avoids the iOS 7.0 task-name method.
- `UIApplication.sharedApplication` is unavailable to app extensions. This package supports application targets only.
- An expiration signal does not interrupt work. The app must stop promptly after observing it. The API supplies no duration, guaranteed extra runtime, scheduler launch, continued execution after expiry, or completion guarantee.
- The local Xcode 26.6 host is below the repository plan's Xcode 27.x baseline. The SDK header API floor is iOS 4.0, while this rustc target supports device deployment from iOS 10.0; the link probe records iOS 10.0 for device and 14.0 for Simulator.

## Exclusions

- No BGTaskScheduler registration, launch, submit, cancel, or app-refresh API; no D20/B25 dependency.
- No background processing task, task name, task history, task registry, URLSession, BackgroundAssets, app delegate, host plist writer, C ABI, entitlement claim, or runtime task run.
- No claim that UIKit grants additional time or that Rust work continues after expiry.

## Primary SDK/API evidence

- Xcode header: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIApplication.h:69,87,141-143`.
- Generated binding: `objc2-ui-kit` 0.3.2 `src/generated/UIApplication.rs`; `UIBackgroundTaskIdentifier` maps to `NSUInteger`, and generated methods expose the selected begin and end selectors behind the `block2` feature.
- Apple docs: [beginBackgroundTask(expirationHandler:)](https://developer.apple.com/documentation/uikit/uiapplication/beginbackgroundtask%28expirationhandler%3A%29), [endBackgroundTask(_:)](https://developer.apple.com/documentation/uikit/uiapplication/endbackgroundtask%28_%3A%29), and [Extending your app's background execution time](https://developer.apple.com/documentation/uikit/extending-your-app-s-background-execution-time).

## Local implementation evidence

- Direct linked frameworks: `UIKit`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`
- The probe imports `_UIBackgroundTaskInvalid` and contains `sharedApplication`, `beginBackgroundTaskWithExpirationHandler:`, and `endBackgroundTask:`
- Release metadata is device `LC_VERSION_MIN_IPHONEOS version 10.0` and Simulator `LC_BUILD_VERSION minos 14.0`
- The link script found no Swift/Python runtime or BGTaskScheduler, URLSession, UserNotifications, or Network.framework imports
- No live UIKit callback, simulator launch, or device run was attempted

## Handoff

Report public names, exact dependency/manifests, checks, direct imports and deployment metadata, host/API limits, deviations, and unresolved assumptions. Compile/link evidence does not prove a live UIKit callback or device behavior.
