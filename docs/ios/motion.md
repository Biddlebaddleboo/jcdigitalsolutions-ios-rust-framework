# iOS raw accelerometer backend

`ios-motion` implements D11's one-shot `framework-motion` contract with Core Motion's raw
accelerometer callback API. It returns the first raw x/y/z sample, including gravity, then stops
updates on the next future poll or drop. Values are multiplied from G by `9.80665` into meters per second squared. The native
`CMLogItem.timestamp` is measurement time in seconds since boot; the backend converts it to
monotonic nanoseconds after finite/range checks and rounds to the nearest nanosecond. It does not
claim nanosecond sensor resolution, freshness, or latency.

## Manager ownership and lifecycle

The app must create and retain one `CMMotionManager` for its Core Motion use, then pass that same
manager and a `MainThreadMarker` to `IosMotionBackend::new`. The backend does not create a hidden
manager or a process-global registry. Apple advises one manager per app because multiple managers
can affect accelerometer/gyroscope update rates.

The iOS crate reexports `CMMotionManager` and `MainThreadMarker` so an app can construct one
manager and pass it to the backend without adding direct binding dependencies:

```rust
use ios_motion::{CMMotionManager, IosMotionBackend, MainThreadMarker};
use framework_motion::Motion;

let Some(marker) = MainThreadMarker::new() else { return };
// SAFETY: The call is made on the main thread, and the app creates only its single manager.
let manager = unsafe { CMMotionManager::new() };
let mut motion = Motion::new(IosMotionBackend::new(manager, marker));
let sample = motion.current_acceleration().await?;
```

The app must coordinate all other access to that manager's accelerometer update channel. A request
returns `ErrorKind::AlreadyExists` if accelerometer updates are already active; it does not replace
another handler. While the request is active, do not start another accelerometer update service.
The future starts work only on first poll and uses a dedicated serial `NSOperationQueue`; it does
not use the main operation queue, which Apple does not recommend for accelerometer delivery. The
first callback publishes one result and invokes the saved `Waker` on that serial callback queue.
This is not a main-thread dispatch: the executor must arrange the next future poll on the main
thread, and the backend supplies no executor or dispatcher. That poll stops updates before it
returns `Ready`; dropping a pending future detaches callback state before stop. Repeated or late
callbacks are ignored. The future and manager calls are main-thread-bound. No update interval or
sample-rate policy is set.

The callback queue can deliver extra samples until the executor polls the woken future or the
caller drops it. The callback only copies one scalar result into synchronized state and never
accesses the manager or future. Core Motion declares the handler escaping and its SDK header says
stop cancels queued operations on the supplied queue; no callback deadline is promised. Samples
after the first are discarded, and no freshness or latency bound applies.

The backend checks `isAccelerometerAvailable`; unavailable hardware/context maps to
`Availability::TemporarilyUnavailable` and a request error with `ErrorKind::Unavailable`. An
`Availability::Available` result does not mean this manager's update channel is idle; if
`isAccelerometerActive` is already true, a request returns `ErrorKind::AlreadyExists` without
taking over the active service. An `NSError` maps to `ErrorKind::Platform` with its representable
nonzero native code preserved. No authorization-request API is exposed, and runtime OS prompt
behavior is not claimed.

## App configuration

Apple's current [device-sensors overview](https://developer.apple.com/documentation/technologyoverviews/device-sensors)
lists accelerometer/gyroscope data via Core Motion with no usage-description key, and lists
pedometer/motion-recording/fall-detection APIs with `NSMotionUsageDescription`. The generic
[Core Motion overview](https://developer.apple.com/documentation/coremotion) is broader, and the
[archived key reference](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html)
describes older linked apps as requiring the key for accelerometer access. This backend does not
edit the app plist or request permission; verify the key requirement for the app's target OS and
SDK. Recheck Apple's current privacy docs when adopting a newer OS or adding other motion APIs.

Add `UIRequiredDeviceCapabilities` with `accelerometer` only if the app must not install or run on
devices without accelerometer hardware. It is not required merely to call Core Motion; the backend
reports availability instead.

This crate adds no Swift source. The recorded device/simulator compile and lint checks use Xcode
26.6 (build 17F113) with the iPhoneOS/iPhoneSimulator 26.5 SDK, below the required Xcode 27.x
baseline. They do not establish physical sensor delivery, callback timing, privacy behavior, or
performance.

## Dependencies and replacement seam

`objc2-core-motion` 0.3.2 supplies the generated public Core Motion types and callback ABI. Its
default features are disabled; `ios-motion` enables only `CMAccelerometer`, `CMLogItem`,
`CMMotionManager`, and `block2`. The backend also uses `objc2` 0.6.5 for retained Objective-C
objects and `MainThreadMarker`, `objc2-foundation` 0.3.2 for `NSError` and the serial
`NSOperationQueue`, and `block2` 0.6.2 for the escaping callback block. These dependencies are
iOS-target-only and remain behind `ios-motion`; no binding type enters `framework-motion`. The
generated bindings preserve the SDK's selector, block, ownership, and availability declarations;
handwritten Objective-C dispatch or a custom block ABI would add unsafe ABI code without removing
the native API. Replacing the binding crates is bounded to this backend and its manifest.
