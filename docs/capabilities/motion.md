# Motion

**framework-motion** defines a portable **no_std** contract for one raw accelerometer sample per request. It contains framework-owned scalar values and a statically selected backend contract; it does not create a sensor manager, global registry, executor, or native backend. The separate [iOS Core Motion backend](../ios/motion.md) implements this contract with an app-supplied `CMMotionManager`; target compile/lint checks do not prove runtime sensor behavior.

## Rust API

~~~rust
use framework_motion::{AccelerationSample, Motion, MotionBackend, MotionError};

async fn read_one_sample<B: MotionBackend>(
    motion: &mut Motion<B>,
) -> Result<AccelerationSample, MotionError> {
    motion.current_acceleration().await
}
~~~

**Motion<B>** owns the caller-supplied backend value. **MotionBackend** is selected statically and returns a concrete associated **core::future::Future**; there is no boxed trait object, executor, **Send** requirement, registry, hidden initialization, or global sensor manager. **Motion::availability** is a non-prompting backend-availability signal; it does not guarantee that an exclusive native update channel is free or that a request will succeed. The API has one operation, **Motion::current_acceleration()**, with no request options, stream, subscription, sampling interval, frequency control, batching, or rate policy.

## Values and coordinate semantics

**Acceleration** stores finite x/y/z values in meters per second squared. Construction rejects NaN and positive or negative infinity, but applies no device-specific magnitude limit. The components are the backend's device-fixed sensor axes in x/y/z order. Backends must document their native axis map and preserve the component values without screen-orientation or world-frame rotation.

The contract models raw accelerometer output, including gravity. It does not model gravity-removed user acceleration, angular velocity, magnetometer values, fused attitude, or rotation. A sample and its values are Rust-owned scalars returned by value, with no allocation or borrowed native object.

Apple's Core Motion raw accelerometer API reports three device-axis components in G units. For that backend, x/y/z map directly to the corresponding Core Motion device-axis components; convert each component to SI by multiplying by the standard-gravity factor **9.80665 m/s² per G**. This factor is a framework conversion choice based on the [NIST Guide to the SI, Appendix B.8](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b8). Apple's raw-events guide describes raw accelerometer values, and **CMAcceleration** defines its components in G. Apple's processed **CMDeviceMotion.userAcceleration** is a different, gravity-removed value; D11 does not use it. See Apple's [raw accelerometer guide](https://developer.apple.com/documentation/coremotion/getting-raw-accelerometer-events), [CMAcceleration](https://developer.apple.com/documentation/coremotion/cmacceleration), [Core Motion overview](https://developer.apple.com/documentation/coremotion), and [userAcceleration](https://developer.apple.com/documentation/coremotion/cmdevicemotion/useracceleration).

## Measurement timestamp

**MotionTimestamp** is an unsigned monotonic nanosecond count for measurement time. It is not Unix time and is not callback or return time. Values are comparable only within the same backend clock domain and device boot/session; the value itself carries no clock-domain identifier. A backend must convert its native measurement timestamp and reject non-finite or out-of-range source values rather than wrap or saturate them.

For Core Motion, **CMLogItem.timestamp** is seconds since device boot. Convert it to nanoseconds by multiplying by 1,000,000,000 and rounding to the nearest integer nanosecond after finite and range checks. The unit does not promise nanosecond sensor resolution. See Apple's [CMLogItem timestamp](https://developer.apple.com/documentation/coremotion/cmlogitem/timestamp).

The contract makes no freshness, sampling-latency, or delivery-latency guarantee. A returned sample's timestamp is the measurement time; consumers must not treat callback or return time as a replacement.

## Operation and cancellation semantics

A request starts on first poll. Dropping it before first poll starts no backend work. Dropping a pending future first detaches or invalidates its callback context, then requests native cancellation when available. If cancellation is unavailable or races with drop, late callbacks may observe detached state but must not access freed future memory. Callback state, or an equivalent stable token, must remain safe until no native callback can access it and must be released exactly once. A completion/drop race has one winner: completion yields one result only if the future remains live; if drop wins, suppress the result. Ignore duplicate and late callbacks.

Errors preserve the **framework_core::ErrorKind** category and optional native code through **MotionError**. No permission prompt or authorization-request API is exposed. Platform usage-description, privacy, and native escape behavior belong to a future platform-backend guide.

## Core Motion backend seam and limits

Core Motion reports accelerometer availability through **CMMotionManager.isAccelerometerAvailable**. Its raw API starts updates and provides callbacks or a latest-sample property, then stops updates; it does not document a distinct one-shot request method. The iOS backend uses that update path internally, returns only the first callback result, and stops on the next future poll or drop. Apple advises one **CMMotionManager** per app because multiple managers can affect sensor update rates; the backend takes the app's explicit manager and this portable API adds no global registry. See Apple's [CMMotionManager](https://developer.apple.com/documentation/coremotion/cmmotionmanager) and [isAccelerometerAvailable](https://developer.apple.com/documentation/coremotion/cmmotionmanager/isaccelerometeravailable).

Apple's current device-sensors overview lists accelerometer/gyroscope data via Core Motion with no usage-description key, and lists pedometer/motion-recording/fall-detection APIs with **NSMotionUsageDescription**. The generic Core Motion overview is broader, and the archived key reference describes older linked apps as requiring a key for accelerometer access. The B15 [iOS guide](../ios/motion.md) records this source/version caveat; this portable contract adds no plist key or permission behavior. See Apple's [Core Motion overview](https://developer.apple.com/documentation/coremotion), [device sensors](https://developer.apple.com/documentation/technologyoverviews/device-sensors), [NSMotionUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmotionusagedescription), and [archived Cocoa keys](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html).

This contract covers only one finite, raw, device-axis acceleration sample. It does not implement gyroscope, magnetometer, fused values, streams, background collection, filtering, coordinate transforms, or platform backend code. It makes no physical-hardware, availability, permission, freshness, latency, or native parity claim.
