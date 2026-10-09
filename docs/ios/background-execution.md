# iOS Background Execution

`ios-background-execution` provides one opt-in Rust adapter for UIKit's `UIApplication` background-task lease API. It supports iOS application targets, not app extensions.

## Begin, poll, and end

Begin on the main thread and retain the lease while doing bounded work:

```rust,ignore
let execution = BackgroundExecution::new(IosBackgroundExecution::new());
let Some(main_thread) = MainThread::current() else { return; };
let Ok(lease) = execution.begin(main_thread) else { return; };
let expiry = lease.expiry_signal();
while app_has_bounded_work() && !expiry.is_expired() {
    run_one_bounded_unit();
}
lease.end();
```

The function names in the loop are app-owned placeholders. `MainThread::current()` must succeed for UIKit access. The lease retains its `MainThreadMarker`, so `IosBackgroundLease` is `!Send`/`!Sync`; explicit end and its drop fallback remain on the thread that began it. `IosBackgroundExpiry` is cloneable and may be polled by work on another thread.

UIKit calls the expiration handler synchronously on the main thread shortly before the remaining background time reaches zero. The adapter marks the expiry signal and ends the UIKit identifier once. If expiry races the return from begin, it ends the identifier as soon as it is available. App work must still observe the signal and stop promptly; setting the signal does not interrupt Rust work. `end` consumes the handle, and `Drop` is only a fallback if app code omits explicit end.

## Availability and limits

- The selected `UIApplication` methods and `UIBackgroundTaskInvalid` constant are `API_AVAILABLE(ios(4.0))` in the local iOS SDK 26.5 header. The adapter uses the unnamed `beginBackgroundTaskWithExpirationHandler:` method, not the iOS 7.0 task-name variant.
- `UIApplication.sharedApplication` is marked `NS_EXTENSION_UNAVAILABLE_IOS`; this adapter is for app targets only.
- UIKit may return an invalid task identifier when a background task cannot begin; the adapter maps it to `ErrorKind::Unavailable`.
- This API has no task identifier owned by app code, scheduler, registration, plist task ID, recurrence, history, or future launch.
- No duration, guaranteed extra runtime, guaranteed work completion, or continued execution after expiry is promised. The OS may terminate an app that fails to end the task before time expires.
- This adapter does not use BGTaskScheduler or URLSession and does not add a background mode, entitlement, or app delegate implementation.
- Local compile/link evidence uses Xcode 26.6 build 17F113 with iPhoneOS/iPhoneSimulator SDK 26.5; the repository's current plan baseline is Xcode 27.x. The header API floor is iOS 4.0, while rustc supports device deployment from iOS 10.0; the probe records iOS 10.0 for device and iOS 14.0 for Simulator.

## Implementation evidence

The active SDK declarations are in `UIApplication.h`: `UIBackgroundTaskInvalid` at line 69, app-extension unavailability for `sharedApplication` at line 87, and the selected begin/end selectors at lines 141–143. The backend uses generated `objc2-ui-kit` 0.3.2 bindings and `block2`; it adds no handwritten Apple ABI or Swift source.

Apple references: [`beginBackgroundTask(expirationHandler:)`](https://developer.apple.com/documentation/uikit/uiapplication/beginbackgroundtask%28expirationhandler%3A%29), [`endBackgroundTask(_:)`](https://developer.apple.com/documentation/uikit/uiapplication/endbackgroundtask%28_%3A%29), and [Extending your app's background execution time](https://developer.apple.com/documentation/uikit/extending-your-app-s-background-execution-time).

Build and link checks do not prove a live UIKit expiry callback or physical-device behavior.
