# iOS Metal Performance Shaders device presence

The `ios-mps-status` crate exposes one iOS-only point-in-time query:

```rust,ignore
let available = ios_mps_status::preferred_mps_device_available();
```

The wrapper calls `MPSGetPreferredDevice(MPSDeviceOptions::Default)` and returns whether the result is non-null. It drops the retained native device before return and exposes no Metal object or identifier.

Apple marks `MPSGetPreferredDevice` and `MPSDeviceOptionsDefault` available from iOS 12.2. The host app must therefore use an iOS 12.2-or-later deployment target when calling this API. The validation link gate uses iOS 12.2 for device and iOS Simulator 14.0 for simulator; the simulator setting is not the API availability floor.

This query does not submit a command buffer or perform an MPS operation. A positive result does not establish support for a particular operation, model, shader, or workload. The API may select a different available device when requested options cannot be met; this wrapper always uses the default options. It makes no performance, initialization-cost, or thread-affinity claim.

The query requires no permission, Info.plist key, or entitlement. It reads no user data and has no callback or cancellation lifecycle. There is no portable facade; non-iOS targets expose no query function.

The package uses `objc2-metal-performance-shaders` 0.3.2 with default features disabled and only `MPSCore` enabled. This is an iOS API wrapper, not a replacement for Metal Performance Shaders and not a claim of full MPS support.

References: [Apple `MPSGetPreferredDevice`](https://developer.apple.com/documentation/metalperformanceshaders/mpsgetpreferreddevice%28_%3A%29), [Apple `MPSDeviceOptions.Default`](https://developer.apple.com/documentation/metalperformanceshaders/mpsdeviceoptions/3088915-default), [generated Rust binding](https://docs.rs/objc2-metal-performance-shaders/0.3.2/objc2_metal_performance_shaders/fn.MPSGetPreferredDevice.html).
