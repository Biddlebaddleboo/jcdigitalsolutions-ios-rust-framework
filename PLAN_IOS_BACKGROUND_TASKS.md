# PLAN_IOS_BACKGROUND_TASKS.md — Workstream B25: iOS App Refresh Tasks

## Objective

Add an opt-in Rust adapter for D20 app-refresh register, submit, synchronous work closure, expiry, and cancel calls via public `BGTaskScheduler`

## Dependencies

- D20 `framework-background`
- Xcode 26.6 (build 17F113), iPhoneOS and iPhoneSimulator SDK 26.5
- `objc2-background-tasks` 0.3.2 generated bindings
- The active SDK marks selected BackgroundTasks classes and methods as iOS 13.0 in `BGTask.h`, `BGTaskRequest.h`, and `BGTaskScheduler.h` under `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/BackgroundTasks.framework/Headers`

## Write scope

- `PLAN_IOS_BACKGROUND_TASKS.md`
- `platform/ios/ios-background-tasks/**`
- `docs/ios/background-tasks.md`

Root owns Cargo.lock, shared workspace manifests, the capability manifest and counts, shared indexes, aggregate plans, CI, and aggregate validation docs. Do not edit those paths

## Native scope

- Use `BGTaskScheduler.sharedScheduler`, `registerForTaskWithIdentifier:usingQueue:launchHandler:`, `BGAppRefreshTaskRequest`, `submitTaskRequest:error:`, `cancelTaskRequestWithIdentifier:`, `setExpirationHandler:`, and `setTaskCompletedWithSuccess:` only
- The API floor is iOS 13.0 in Xcode 26.6 / iOS SDK 26.5 headers
- Use only generated `objc2-background-tasks` bindings and `block2`; add no Swift or handwritten Apple ABI
- Pass `nil` for `usingQueue`; Apple documents a default background queue. Require a `Send + Sync` Rust closure and make no order or main-queue promise
- Keep native `BGTask` use within its launch closure. The expiry block may run on another queue and captures only shared atomic state, not the non-`Send` task
- The closure is synchronous work, not a `Future`; no async work is awaited and the return is the only native finish point
- The native scheduler holds each registered launch block for the app process lifetime; no unregister API exists
- Register each ID once per process from host launch code before launch ends; duplicate native register can end the app
- `registerForTaskWithIdentifier` returns false if the ID is absent from `BGTaskSchedulerPermittedIdentifiers`
- `submit_app_refresh` leaves `earliestBeginDate` unset. `Ok` means accepted by the scheduler only, not a launch or time promise
- The scheduler controls launch time and may launch each accepted request zero or one time; submit a new request for another run
- Resubmit with the same ID replaces the prior pending request. Cancel removes a pending request only and does not stop a launched closure
- The expiry block only sets an atomic signal; it does not stop Rust work or call the native finish method
- After a normal closure return, the adapter makes one native finish call with the returned outcome, unless expiry wins the atomic race against the finish claim
- A Rust panic is caught at the native boundary, maps to failure, then gets one native finish call
- If expiry wins, a returned closure gets one failure finish call. If app code ignores expiry and does not return, iOS may end the app and no finish call is assured
- Preserve `BGTaskSchedulerErrorDomain` codes in `PlatformErrorCode`; map unavailable, too many pending requests, and not-permitted to portable error kinds

## Host app needs

- Add each app-owned reverse-DNS task ID to `BGTaskSchedulerPermittedIdentifiers` in the host `Info.plist`
- Add `fetch` to host `UIBackgroundModes` for `BGAppRefreshTask`
- This slice uses no entitlement. The host owns plist values, capability/signing setup, launch order, and app state
- When `BGTaskSchedulerPermittedIdentifiers` is set, iOS 13+ disables legacy `application:performFetchWithCompletionHandler:` and `setMinimumBackgroundFetchInterval:` APIs
- Run Rust launch setup on every process launch and call each register method once before launch ends

## Not in scope

- No `BGProcessingTask`, `BGContinuedProcessingTask`, `BGHealthResearchTask`, GPU resource, Health Research entitlement, or continued-work UI
- No UIKit background task, app delegate, BackgroundAssets, URLSession, B13 transfer, local notification, PushKit, APNs, C ABI, or Swift
- No task history, durable result store, persistent registry, runtime launch promise, network access, or live task run

## References

- [Apple `BGTaskScheduler`](https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler)
- [Apple `BGTask.expirationHandler`](https://developer.apple.com/documentation/backgroundtasks/bgtask/expirationhandler)
- [Apple `BGTask.setTaskCompleted(success:)`](https://developer.apple.com/documentation/backgroundtasks/bgtask/settaskcompleted%28success%3A%29)
- [Apple `BGTaskSchedulerPermittedIdentifiers`](https://developer.apple.com/documentation/bundleresources/information-property-list/bgtaskschedulerpermittedidentifiers)
- [Apple `UIBackgroundModes`](https://developer.apple.com/documentation/bundleresources/information-property-list/uibackgroundmodes)
- [`objc2-background-tasks` 0.3.2 generated API](https://docs.rs/objc2-background-tasks/0.3.2/objc2_background_tasks/)

## Handoff

Report public names, exact checks, direct imports, API floor, host plist/mode needs, deviations, and unresolved assumptions. Compile and link evidence does not prove host launch setup, schedule acceptance, runtime launch, task time, expiry, or native execution
