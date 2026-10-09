# iOS app refresh tasks

`ios-background-tasks` adapts D20 to public `BGTaskScheduler` and `BGAppRefreshTask` APIs. The API floor is iOS 13.0 in Xcode 26.6 / iOS SDK 26.5 `BGTask.h`, `BGTaskRequest.h`, and `BGTaskScheduler.h`

```rust,ignore
use framework_background::{
    AppRefreshBackend, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
};
use ios_background_tasks::IosBackgroundTasks;

fn register_and_submit() -> framework_core::Result<()> {
    let backend = IosBackgroundTasks::new();
    let task_id = AppRefreshTaskId::new("com.example.app.refresh")?;
    let request = AppRefreshRequest::new(task_id.clone());
    backend.register_app_refresh(&task_id, |task| {
        if task.is_expired() {
            return AppRefreshOutcome::Failed;
        }
        AppRefreshOutcome::Succeeded
    })?;
    backend.submit_app_refresh(&request)
}
```

Call register once for each task ID on every app process launch, before launch ends. The native scheduler holds each launch block until process end; no unregister API exists. A duplicate native register for one ID can end the app

The callback runs on Apple's default background queue because the adapter passes `nil` for `usingQueue`. It is a synchronous work closure, not a `Future`; no async work is awaited. It must be `Send + Sync`, must poll `task.is_expired()` during long work, and must return promptly after expiry. No callback order or main-queue promise exists

The native task stays within its launch closure. The expiry block may run on another queue and only sets atomic state; it does not access the native task or stop Rust work. After a normal closure return, the adapter makes one `setTaskCompletedWithSuccess:` call with the returned outcome, unless expiry wins the atomic race against the finish claim. A Rust panic is caught, maps to failure, then gets one finish call. If app code ignores expiry and does not return, iOS may end the app and no finish call is assured

Each accepted request may launch zero or one time, at a time set by iOS. `Ok` from submit means the scheduler accepted the request only. It does not mean the task will run. A new submit with the same ID replaces its pending request; cancel affects pending work only and does not stop a launched closure

## Host app metadata

Use a reverse-DNS task ID and list the exact same string in the host app's `BGTaskSchedulerPermittedIdentifiers` array. Add `fetch` to the host app's `UIBackgroundModes` array for `BGAppRefreshTask`. Keep all existing mode values

```xml
<key>BGTaskSchedulerPermittedIdentifiers</key>
<array>
    <string>com.example.app.refresh</string>
</array>
<key>UIBackgroundModes</key>
<array>
    <string>fetch</string>
</array>
```

The app owns these plist values, launch setup, task IDs, and callback state. This crate uses no entitlement and does not create an app delegate. When `BGTaskSchedulerPermittedIdentifiers` is set, iOS 13+ disables legacy `application:performFetchWithCompletionHandler:` and `setMinimumBackgroundFetchInterval:` APIs

## Link and runtime limits

The adapter uses generated `objc2-background-tasks` 0.3.2 bindings with only `BGTask`, `BGTaskRequest`, `BGTaskScheduler`, `block2`, and `dispatch2` features. It adds no Swift or handwritten Apple ABI

`sh platform/ios/ios-background-tasks/check-link-imports.sh` builds device and Simulator probes and inspects Mach-O imports and symbols; it never runs either probe. Target check and link evidence do not prove app launch setup, request acceptance, OS launch, task time, callback delivery, expiry, runtime behavior, or device behavior. This host toolchain is Xcode 26.6 / SDK 26.5, below the planned Xcode 27.x baseline

See [Apple `BGTaskScheduler`](https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler), [Apple `BGTask.expirationHandler`](https://developer.apple.com/documentation/backgroundtasks/bgtask/expirationhandler), [Apple `BGTask.setTaskCompleted(success:)`](https://developer.apple.com/documentation/backgroundtasks/bgtask/settaskcompleted%28success%3A%29), [Apple `BGTaskSchedulerPermittedIdentifiers`](https://developer.apple.com/documentation/bundleresources/information-property-list/bgtaskschedulerpermittedidentifiers), and [Apple `UIBackgroundModes`](https://developer.apple.com/documentation/bundleresources/information-property-list/uibackgroundmodes)
