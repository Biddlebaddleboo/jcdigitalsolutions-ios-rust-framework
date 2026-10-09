#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow MatterAddDeviceRequest API-support snapshot for iOS"]

/// Failure to read MatterAddDeviceRequest API support
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatterSupportError {
    /// The iOS 17.0 API symbol is unavailable or the target is not iOS
    NativeApiUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_matter_add_device_request_is_supported(supported_out: *mut u8) -> u8;
}

/// Reads whether MatterSupport reports `MatterAddDeviceRequest` usage as supported
///
/// On iOS 17.0 and later this returns only Apple's `MatterAddDeviceRequest.isSupported` value
/// The query does not create a request or topology, scan for devices, start commissioning, or show
/// UI. It does not report generic Matter, Thread, Bluetooth, accessory, entitlement, ecosystem,
/// home, permission, or setup readiness. Older iOS releases return `NativeApiUnavailable` because
/// the Swift API symbol is weak-linked. Non-iOS targets return `NativeApiUnavailable`
pub fn matter_add_device_request_is_supported() -> Result<bool, MatterSupportError> {
    #[cfg(target_os = "ios")]
    {
        let mut supported = 0_u8;
        // SAFETY: the C bridge accepts this valid output pointer and checks the weak-linked API
        let result = unsafe { framework_matter_add_device_request_is_supported(&mut supported) };
        match (result, supported) {
            (0, 0) => Ok(false),
            (0, 1) => Ok(true),
            (1, _) => Err(MatterSupportError::NativeApiUnavailable),
            _ => Err(MatterSupportError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(MatterSupportError::NativeApiUnavailable)
    }
}
