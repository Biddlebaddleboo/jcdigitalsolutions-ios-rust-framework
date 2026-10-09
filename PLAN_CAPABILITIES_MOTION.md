# PLAN_CAPABILITIES_MOTION.md — Workstream D11: One-shot Accelerometer Contract

## Status

D11 portable contract and guide plus B15's iOS backend are implemented locally. Eight deterministic portable tests pass for finite-value validation, axis/timestamp preservation, one-sample fake-backend results, error mapping, first-poll start, both drop paths, and duplicate/late callback handling. `cargo fmt --all -- --check`, `cargo test -p framework-motion` (8 passed), `cargo check -p framework-motion --no-default-features`, `cargo xtask no-std-check`, `cargo xtask docs-check`, and `git diff --check` pass. Tests use an in-process fake backend; they do not exercise Core Motion, native callback threading, or physical sensors. Device/simulator compile/lint evidence passes on the recorded Xcode 26.6 / iOS SDK 26.5 host, below the required Xcode 27.x baseline. Runtime sensor behavior is not claimed.

## Objective

Plan a small portable Rust contract for one finite, device-axis accelerometer sample per request. This workstream defines no sensor stream, sample-rate policy, or native backend.

## Dependencies

- Foundation A and **framework-core** are integrated
- D1 backend and future conventions are in place
- B15 is the separate named native implementation in [PLAN_IOS_MOTION.md](PLAN_IOS_MOTION.md)

## Read first

- **PLAN.md**
- **PLAN_CAPABILITIES.md**
- **PLAN_CAPABILITIES_LOCATION.md** for scalar-value and one-shot-future conventions
- **docs/ARCHITECTURE.md**
- **docs/PORTABILITY_AND_ABI.md**
- **docs/core/ASYNC_AND_OWNERSHIP.md**

## Implementation write scope

- **crates/framework-motion/***
- **docs/capabilities/motion.md**
- **Cargo.lock** only for central workspace resolution by the orchestrator

Do not edit root Cargo configuration, the shared capability manifest, D1-owned crates, iOS backends or their plans, Swift ABI, bindings, or unrelated sensor families. The orchestrator owns manifest and index reconciliation. Do not add a dependency for this scalar contract.

The orchestrator owns the portable **no_std** check-list update in **tools/xtask/src/main.rs**; it is not part of D11 implementation scope.

## Required portable contract

- Add an independently usable crate named **framework-motion**, with **#![no_std]**, no third-party dependency, and no heap allocation in its values or facade.
- Expose framework-owned **Acceleration**, **MotionTimestamp**, **AccelerationSample**, and **MotionError** values. Use finite **f64** x/y/z components in meters per second squared.
- Define x/y/z in the device-fixed sensor frame exposed by the backend. Do not rotate values for screen orientation or a world frame. Document each backend's native axis map.
- Model raw accelerometer output, including gravity; do not model gravity-removed user acceleration or fused attitude.
- Reject NaN and positive or negative infinity in any acceleration component. Do not add a device-specific magnitude limit.
- Store the measurement timestamp as unsigned monotonic nanoseconds. It is not Unix time. Values are comparable only within the same backend clock domain and device boot/session; do not promise cross-device, cross-boot, or wall-clock comparison.
- Expose **MotionBackend** with **Availability** and a concrete associated **Future** for one sample. Expose a thin generic **Motion<B>** facade selected by Rust's type system. Do not add dynamic dispatch, a registry, hidden initialization, a required executor, or a **Send** bound.
- Define availability as a non-prompting backend/hardware signal, not a guarantee that an exclusive native update channel is free or that a request will succeed.
- Expose **Motion::current_acceleration()** as one request for one sample. Do not add a request option, update stream, callback stream, sampling interval, frequency control, batching, or rate policy.
- Preserve the **framework_core::ErrorKind** category and optional native code through **MotionError**.
- A request starts only on first poll; drop before first poll starts no work. Drop of a pending future first detaches or invalidates its callback context, then requests native cancellation when available. If cancellation is unavailable or races with drop, late callbacks must observe detached state without accessing freed future memory. If completion races with drop, deliver one result only when completion wins; otherwise suppress it. Ignore duplicate or late callbacks and release callback state exactly once.
- Return one owned scalar sample by value. State that no freshness or latency bound applies; use the sample's measurement timestamp, not callback or return time.
- Add no permission prompt API. Platform usage-description, privacy, manager ownership, and native escape details are in [the B15 iOS backend guide](docs/ios/motion.md).

## Core Motion facts and native seam

Apple documents raw accelerometer values on three device axes and uses G units: its raw-events guide describes 1.0 as about 9.8 m/s², while **CMAcceleration** defines each axis in G and describes G as about 9.81 m/s². The portable contract uses the exact standard-gravity factor 9.80665 m/s² to convert a native G value to SI; that factor is a framework choice based on the [NIST Guide to the SI, Appendix B.8](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b8). See Apple's [raw accelerometer guide](https://developer.apple.com/documentation/coremotion/getting-raw-accelerometer-events) and [CMAcceleration](https://developer.apple.com/documentation/coremotion/cmacceleration).

Apple distinguishes raw sensor values from processed motion values that remove gravity; **CMDeviceMotion.userAcceleration** is the user-imparted component. D11 chooses the raw accelerometer channel, including gravity. This is an inference from Apple's raw-versus-processed distinction, not a claim that all native sensors share identical bias or calibration. Cite Apple's [Core Motion overview](https://developer.apple.com/documentation/coremotion) and [userAcceleration](https://developer.apple.com/documentation/coremotion/cmdevicemotion/useracceleration).

Apple's **CMLogItem.timestamp** is seconds since device boot. D11 normalizes that source to monotonic nanoseconds by multiplying by 1,000,000,000 and rounding to the nearest integer nanosecond after finite/range checks. The unit does not promise nanosecond sensor resolution. See Apple's [CMLogItem timestamp](https://developer.apple.com/documentation/coremotion/cmlogitem/timestamp).

Apple documents accelerometer availability through **isAccelerometerAvailable**, and Core Motion's raw guide notes that unavailable hardware yields no data. The documented raw path starts accelerometer updates, supplies callbacks or a latest-sample property, and provides a stop method; it does not describe a distinct one-shot request method. B15 uses the native update path internally, returns only the first callback result, and stops on the next main-thread future poll or drop. Its dedicated serial callback queue does not message the manager; extra native callbacks before stop are discarded. Apple advises one **CMMotionManager** per app because multiple managers can affect sensor update rates; B15 takes the app's explicit manager and does not add a global registry. See Apple's [CMMotionManager](https://developer.apple.com/documentation/coremotion/cmmotionmanager) and [isAccelerometerAvailable](https://developer.apple.com/documentation/coremotion/cmmotionmanager/isaccelerometeravailable).

Apple's current device-sensors overview lists accelerometer/gyroscope data via Core Motion with no usage-description key, and lists pedometer/motion-recording/fall-detection APIs with **NSMotionUsageDescription**. The generic Core Motion overview is broader, and the archived key reference describes older linked apps as requiring a key for accelerometer access. B15's guide records this source/version caveat; this crate does not edit the app plist or request permission. See Apple's [Core Motion overview](https://developer.apple.com/documentation/coremotion), [device sensors](https://developer.apple.com/documentation/technologyoverviews/device-sensors), [NSMotionUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmotionusagedescription), and [archived Cocoa keys](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html).

## Explicit exclusions

- Gyroscope and angular velocity
- Magnetometer and magnetic field
- Fused device attitude, gravity-removed acceleration, or rotation
- Continuous streams, subscriptions, recorder/history APIs, and background collection
- Sampling interval, frequency, filtering, batching, or rate policy
- Screen/world coordinate transforms
- iOS, Android, desktop, or web backend code

## Validation and handoff

- Add deterministic tests for finite-value validation, axis component preservation, timestamp units, one-sample fake-backend results, error mapping, first-poll start, drop-before-poll, pending-drop cancellation, and exactly-once callback behavior. Do not require physical sensor hardware.
- Run **cargo fmt --all -- --check**, **cargo test -p framework-motion**, **cargo check -p framework-motion --no-default-features**, **cargo xtask no-std-check**, **cargo xtask docs-check**, and **git diff --check**.
- Inspect the public API for platform types, **std**, allocation, dynamic dispatch, hidden initialization, rate policy, and unrelated dependencies.
- Report the exact native axis map, G-to-SI conversion, timestamp clock domain, manager ownership, permission/plist facts, checks, limits, and any deviation.
- Keep this plan until the final V1 plan audit; remove it with the other PLAN files before the final implementation commit.
