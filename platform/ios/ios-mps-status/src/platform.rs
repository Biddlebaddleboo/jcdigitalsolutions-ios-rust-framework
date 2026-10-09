use objc2_metal_performance_shaders::{MPSDeviceOptions, MPSGetPreferredDevice};

/// Return whether Metal Performance Shaders provides a preferred device with default options.
///
/// This point-in-time query maps a non-null result from
/// [`MPSGetPreferredDevice`](https://developer.apple.com/documentation/metalperformanceshaders/mpsgetpreferreddevice%28_%3A%29)
/// to `true`. The retained native device is dropped before return and is never exposed. A `true`
/// result does not guarantee that any particular MPS operation, model, or workload can run.
///
/// The call requires iOS 12.2 or later. Apple documents no permission, Info.plist key, or
/// entitlement requirement for this query. The API's execution cost and thread-affinity
/// requirements are not specified here; no performance or thread-safety claim is made.
pub fn preferred_mps_device_available() -> bool {
    // SAFETY: the generated binding accepts this valid default option value and returns an
    // owned retained device or `None`; no native pointer or borrowed value crosses this call.
    unsafe { MPSGetPreferredDevice(MPSDeviceOptions::Default) }.is_some()
}
