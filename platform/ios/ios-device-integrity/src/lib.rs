#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Point-in-time, non-prompting DeviceCheck and App Attest support queries for iOS."]

use framework_device_integrity::AvailabilitySnapshot;

/// Captures the native DeviceCheck and App Attest `isSupported` values without starting either
/// service.
///
/// This function is synchronous and does not prompt, generate a token or key, attest a key,
/// generate an assertion, or make a network request. DeviceCheck is reported unsupported before
/// iOS 11.0, where its `DeviceCheck.framework` API is unavailable. App Attest is reported
/// unsupported before iOS 14.0, where its native API is unavailable. On iOS 11.0 through 13.x,
/// DeviceCheck is queried while App Attest is reported unsupported. The returned values are
/// transient capability hints only; they do not establish device integrity, app identity,
/// enrollment, entitlement configuration, or future operation success. On non-iOS targets, both
/// values are `false` because these APIs are iOS-specific. Apple notes that App Attest support may
/// vary for app extensions and that a positive support result does not make all operations valid
/// from an extension.
pub fn availability_snapshot() -> AvailabilitySnapshot {
    #[cfg(target_os = "ios")]
    {
        use objc2_device_check::{DCAppAttestService, DCDevice};

        let device_check_supported = if objc2::available!(ios = 11.0, ..) {
            // SAFETY: this calls the documented DeviceCheck class property, which returns its
            // retained current-device object; no caller-provided pointer or state is involved.
            let device = unsafe { DCDevice::currentDevice() };
            // SAFETY: the retained current-device object is valid for this documented Boolean query.
            unsafe { device.isSupported() }
        } else {
            false
        };

        let app_attest_supported = if objc2::available!(ios = 14.0, ..) {
            // SAFETY: the runtime availability check precedes this iOS 14 API; the documented
            // singleton class property returns a retained service instance.
            let service = unsafe { DCAppAttestService::sharedService() };
            // SAFETY: the retained service instance is valid for this documented Boolean query.
            unsafe { service.isSupported() }
        } else {
            false
        };

        AvailabilitySnapshot::new(device_check_supported, app_attest_supported)
    }

    #[cfg(not(target_os = "ios"))]
    {
        AvailabilitySnapshot::new(false, false)
    }
}
