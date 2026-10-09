# PLAN_IOS_MOTION.md — Workstream B15: One-shot Core Motion Backend

## Status

B15 is implemented locally as `ios-motion` with an iOS-only Core Motion dependency and an integration guide. Device and simulator compile/lint checks pass; no tests were added or run, and no physical sensor, runtime callback, privacy prompt, or permission behavior is claimed. The recorded target-check host is Xcode 26.6 with iPhoneOS/iPhoneSimulator SDK 26.5, below the required Xcode 27.x baseline.

## Objective

Implement the D11 `framework-motion` contract with one raw accelerometer sample per request through public Core Motion APIs. Keep the backend separate from Core Location and other sensor families.

## Dependencies

- D11 `framework-motion` is integrated.
- B15 owns `platform/ios/ios-motion/**` and `docs/ios/motion.md`.
- The root orchestrator owns workspace dependency pins, `Cargo.lock`, CI, capability manifest, and shared plan/index reconciliation.

## Read first

- `PLAN.md`
- `PLAN_CAPABILITIES_MOTION.md`
- `PLAN_IOS_NATIVE.md`
- `docs/capabilities/motion.md`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- Apple's [CMMotionManager](https://developer.apple.com/documentation/coremotion/cmmotionmanager), [raw accelerometer guide](https://developer.apple.com/documentation/coremotion/getting-raw-accelerometer-events), [Core Motion overview](https://developer.apple.com/documentation/coremotion), [motion usage-description key](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmotionusagedescription), and [device capability key](https://developer.apple.com/documentation/bundleresources/information-property-list/uirequireddevicecapabilities).
- Apple's [Device sensors overview](https://developer.apple.com/documentation/technologyoverviews/device-sensors), whose current table lists accelerometer and gyroscope data without a usage-description key.

## Write scope

- `platform/ios/ios-motion/**`
- `docs/ios/motion.md`

Do not edit the portable `framework-motion` contract, Swift ABI, other sensor families, or shared manifests/indices. Do not create a global manager registry or prompt for permission.

## Required behavior

- Expose `IosMotionBackend` as a statically selected `MotionBackend`; no default feature or host-side Core Motion dependency.
- Require an app-supplied retained `CMMotionManager`. The app owns its single manager; the backend retains and exposes that same instance. Do not create a hidden manager or global registry.
- Bind the backend and each future to the supplied `MainThreadMarker`; all direct manager calls occur on that main-thread context. Use a dedicated serial `NSOperationQueue` for callbacks, since Apple does not recommend the main operation queue for accelerometer delivery.
- Require exclusive use of the manager's accelerometer update channel while a request is active. If `isAccelerometerActive` is already true, return `ErrorKind::AlreadyExists` without taking over or stopping the existing service. The app must coordinate other users of its manager.
- Report `Availability::Available` when `isAccelerometerAvailable` is true and `Availability::TemporarilyUnavailable` otherwise. A request rechecks availability before start.
- Start `startAccelerometerUpdatesToQueue_withHandler` only on first future poll. Do not set an update interval or add rate policy. The first callback publishes one result and invokes the saved waker on the serial callback queue; the executor must arrange the next poll on the main thread. That poll stops updates before returning `Ready`. Pending drop detaches then stops. Ignore repeat callbacks through exactly-once state. Since the callback queue and executor may be delayed, do not promise a stop deadline, freshness, or latency bound.
- Copy raw device-axis `CMAcceleration` x/y/z values, multiply each G value by exactly `9.80665`, validate through `Acceleration::new`, and return the `CMLogItem.timestamp` measurement time in monotonic nanoseconds after finite/range checks and nearest-nanosecond rounding.
- If the callback has no data and no error, return `ErrorKind::Unavailable`. Map a native `NSError` to `ErrorKind::Platform` and preserve a representable nonzero native code.
- Use a stable `Arc` callback cell with a mutex-protected waker/result/detached state. The callback block captures only this cell; it never accesses the future or manager. Drop must detach first, then call `stopAccelerometerUpdates` if this future started the service. Late or duplicate callbacks cannot access future memory or wake a detached future.
- Do not add an authorization request or permission prompt API. The current Apple Device sensors table lists accelerometer/gyroscope data with no usage-description key, while the generic Core Motion overview and archived key reference are broader; the app guide must state this source/version caveat. `UIRequiredDeviceCapabilities=accelerometer` is optional unless the app requires hardware to install/run.
- Keep all Rust source; add no Swift.

## Validation and limits

- Run `cargo +1.94.1 fmt --all -- --check`, locked `cargo check` and strict Clippy for iOS device and simulator, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, `cargo xtask no-std-check`, and `git diff --check`.
- Do not add or run tests in this execution. Compile/lint checks do not establish runtime Core Motion delivery, cancellation timing, physical-device availability, privacy behavior, or performance.
- The planned Xcode 27.x baseline is not met by the local Xcode 26.6 installation.
