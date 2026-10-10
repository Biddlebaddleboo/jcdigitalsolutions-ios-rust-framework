# PLAN_CAPABILITIES_APPLICATION_LIFECYCLE.md — B453: native iOS application lifecycle

## Objective

Complete the missing native iOS implementation for canonical capability row
`001-core-app-ui-application-lifecycle` by implementing the existing
`framework_app::ApplicationBackend` contract in a capability-scoped `ios-app-lifecycle` crate.
This is a native backend increment, not a new portable contract or a complete application/UI
runtime.

## Existing contract and dependency

`framework-app` already supplies `ApplicationState`, `LifecycleEvent`, `ApplicationBackend`, and
the generic `Application<B>` facade. Keep their API and semantics unchanged. The backend is
selected statically by the Rust caller; do not add a global service, initialization routine,
registry, event loop, executor, thread, or C ABI surface. Keep UIKit and Objective-C types inside
the iOS crate.

## Required behavior

- Add `ios-app-lifecycle`, whose `IosApplicationLifecycle::new(MainThread)` implements
  `framework_app::ApplicationBackend` and retains the supplied `ios_runtime::main_thread::MainThread`
  proof for its lifetime.
- At construction, sample `UIApplication.applicationState` once to seed the current semantic state,
  then register for `UIApplicationDidBecomeActiveNotification`,
  `UIApplicationWillResignActiveNotification`, `UIApplicationWillEnterForegroundNotification`,
  `UIApplicationDidEnterBackgroundNotification`, and `UIApplicationWillTerminateNotification` on
  the main queue. Construction emits no initial lifecycle event.
- Map `UIApplicationStateActive` to `ApplicationState::Foreground`,
  `UIApplicationStateInactive` to `ApplicationState::Inactive`, and
  `UIApplicationStateBackground` to `ApplicationState::Background`. An unrecognized native state
  makes `state()` return `ErrorKind::Platform` with the raw signed native value in
  `PlatformErrorCode`.
- Map `UIApplicationDidBecomeActiveNotification` to `StateChanged(Foreground)`;
  `UIApplicationWillResignActiveNotification` and
  `UIApplicationWillEnterForegroundNotification` to `StateChanged(Inactive)`; and
  `UIApplicationDidEnterBackgroundNotification` to `StateChanged(Background)`. Update the cached
  state for each accepted notification. Coalesce consecutive duplicate `StateChanged` values.
- `UIApplicationWillTerminateNotification` changes the cached state to
  `ApplicationState::Terminating` and queues `LifecycleEvent::TerminationRequested` only. Do not
  also queue `StateChanged(Terminating)`; never emit `LifecycleEvent::Terminated` or report
  `ApplicationState::Terminated`.
- Keep a fixed-capacity 16-event FIFO. When a new event arrives at capacity, drop the oldest queued
  event and retain the new event. The API exposes no overflow counter; document that an overflow
  can omit an earlier transition, while `state()` reports the latest state known to the backend.
  `poll_event()` removes and returns one event without waiting.
- `availability()` is `Availability::Available` after successful backend construction. No
  permission, entitlement, or usage-description key is required. This is an app-only backend;
  `UIApplication.sharedApplication` is unavailable to app extensions.
- Own every notification-observer token in the backend and remove all tokens in `Drop`. Observer
  closures may retain only backend event state, not the backend or observer-token collection, so
  teardown has no retain cycle. All construction, backend calls, callback state mutation, and drop
  occur on UIKit's main thread. Catch panics before returning through a native callback with
  `ios_runtime::ffi::catch_unwind`.

## Aggregate lifecycle and limits

This B453 backend reports `UIApplication` aggregate state and notification events. On scene-based
apps, Apple defines `UIApplication.applicationState` from the most active scene; these notifications
do not identify a scene or deliver one event per scene. B81's `ios-app-scenes` snapshot remains
unchanged as the separate per-scene point-in-time query. B454 adds a distinct
`ios_app_scenes::IosSceneLifecycleEvents` source for per-scene notifications and session identity;
it does not alter this B453 backend. Do not add a `UISceneDelegate`, per-scene identifiers,
windows, UI operations, app-delegate replacement, or method swizzling to B453, or claim that its
notifications observe every scene transition.

The backend observes only after construction and makes no retroactive event claim. `Launching` is
not emitted. iOS may suspend or terminate a process without a delivered
`UIApplicationWillTerminateNotification`; this callback is best-effort and does not guarantee
cleanup time. `Terminated` cannot be observed from the process that has ended.

The deployment floor is iOS 4.0, the first availability of `UIApplication.applicationState` and
the foreground/background notification declarations in the installed iPhoneOS SDK. The backend
does not target macOS Catalyst, watchOS, tvOS, app extensions, or other Apple targets.

## Deterministic regression tests

Factor the event-state reducer and fixed ring into platform-independent internal Rust code so host
tests can use synthetic native-state/event values without UIApplication, a simulator, delays, or
user interaction. Cover:

1. all three `UIApplicationState` mappings and unknown raw-state error preservation;
2. all five notification mappings, including `WillEnterForeground -> Inactive`;
3. initial state seeding without a synthetic initial event;
4. state update and the single `TerminationRequested` event for termination, with no
   `Terminated` event/state;
5. consecutive duplicate-state coalescing, FIFO polling, capacity 16, and drop-oldest overflow;
6. observer-token removal on drop through a deterministic registration/token test seam, with no
   callback accepted after teardown;
7. panic containment around a synthetic native callback.

Do not launch an app, synthesize real UIKit lifecycle transitions, or claim runtime evidence as part
of these regression tests.

## Acceptance and validation

The root integration registers validation profile `ios-app-lifecycle` in
`tools/validation/specs/validation-v1.json` with its bounded Python adapter. The profile must guard
the crate/API shape, notification map, fixed queue and drop behavior, token teardown, platform
floor, and explicit no-runtime/no-scene claims without duplicating generic validation orchestration.

Run the crate's focused `check.sh` and the registered profile after root integration. The intended
commands include:

```sh
cargo +1.94.1 test --locked -p ios-app-lifecycle
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --all-targets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-app-lifecycle --target aarch64-apple-ios --document-private-items
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-app-lifecycle
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-app-lifecycle
```

Record device and Simulator compile, Clippy and rustdoc results separately from test results.
Link/import inspection is not runtime proof; do not execute the app for this workstream or claim
UIKit notification delivery, scene lifecycle, or termination behavior was observed at runtime.

## Write scope

- `PLAN_CAPABILITIES_APPLICATION_LIFECYCLE.md`
- `PLAN_IOS_APPLICATION_LIFECYCLE.md`
- `platform/ios/ios-app-lifecycle/**`
- `docs/ios/app-lifecycle.md`
- The root-owned `ios-app-lifecycle` validation profile and adapter

Root owns workspace and lock integration, canonical capability rows, shared indexes, CI and
aggregate plans. Do not change `framework-app`, B81, any manifest row, root `PLAN.md`, workspace
configuration, or validation spec outside the assigned integration.
