#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Narrow RealityFoundation photogrammetry hardware snapshots for iOS"]

/// Failure to read `PhotogrammetrySession.isSupported`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhotogrammetrySupportError {
    /// The native type metadata was unavailable.
    NativeTypeUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result.
    NativeBridgeFailure,
    /// The current target is not iOS.
    NativeApiUnavailable,
}

/// Failure to read `PhotogrammetrySession.Limits`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhotogrammetryLimitsError {
    /// The native type metadata was unavailable.
    NativeTypeUnavailable,
    /// The native bridge received an invalid pointer, value-witness table, or allocation result.
    NativeBridgeFailure,
    /// The current target is not iOS.
    NativeApiUnavailable,
}

/// Device-specific input limits reported by `PhotogrammetrySession.Limits`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhotogrammetrySessionLimits {
    /// Maximum allowed input image width or height, as reported by Swift `Int`.
    pub maximum_input_image_dimension: i64,
    /// Maximum input image or sample count, as reported by Swift `Int`.
    pub maximum_number_of_input_images: i64,
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_photogrammetry_session_is_supported(supported_out: *mut u8) -> u8;
    fn framework_photogrammetry_session_limits(
        maximum_input_image_dimension_out: *mut i64,
        maximum_number_of_input_images_out: *mut i64,
    ) -> u8;
}

/// Reads whether the current iOS device's hardware supports RealityKit Object Capture processing.
///
/// On iOS 17.0 and later this calls only `RealityFoundation.PhotogrammetrySession.isSupported`.
/// `Ok(false)` means the native API reports that the current hardware does not support Object
/// Capture. This snapshot does not create a session, access input images, start reconstruction,
/// capture camera data, present UI, or guarantee that a later reconstruction succeeds for a given
/// image set or device state. It does not establish broad RealityKit scene, entity, rendering, or
/// ARView support. Non-iOS targets return `NativeApiUnavailable`.
pub fn photogrammetry_is_supported() -> Result<bool, PhotogrammetrySupportError> {
    #[cfg(target_os = "ios")]
    {
        let mut supported = 0_u8;
        // SAFETY: the C bridge checks the output pointer and uses the exact class metadata and
        // scalar getter ABI verified by this package's compiler-oracle gate.
        let result = unsafe { framework_photogrammetry_session_is_supported(&mut supported) };
        match (result, supported) {
            (0, 0) => Ok(false),
            (0, 1) => Ok(true),
            (1, _) => Err(PhotogrammetrySupportError::NativeTypeUnavailable),
            _ => Err(PhotogrammetrySupportError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(PhotogrammetrySupportError::NativeApiUnavailable)
    }
}

/// Reads the current device-specific photogrammetry input limits
///
/// On iOS 17.0 and later this calls only `PhotogrammetrySession.limits` and its two scalar
/// properties. The returned signed 64-bit fields preserve the exact Swift `Int` values for the
/// supported 64-bit iOS targets. Apple documents the image-dimension limit as the maximum allowed
/// input width or height, and the image-count limit as the maximum usable images or samples. Input
/// beyond either limit is ignored by reconstruction and produces an `.invalidSample` message.
/// This snapshot does not create a session, read images, start reconstruction, capture camera
/// data, present UI, or guarantee reconstruction success or quality. It does not establish broad
/// RealityKit scene, entity, rendering, or ARView support. Non-iOS targets return
/// `NativeApiUnavailable`.
pub fn photogrammetry_session_limits()
-> Result<PhotogrammetrySessionLimits, PhotogrammetryLimitsError> {
    #[cfg(target_os = "ios")]
    {
        let mut maximum_input_image_dimension = 0_i64;
        let mut maximum_number_of_input_images = 0_i64;
        // SAFETY: the C bridge checks both output pointers, allocates storage from the exact
        // `Limits` metadata and value witnesses, invokes the compiler-verified getters, then
        // destroys the initialized Swift value before it frees that storage.
        let result = unsafe {
            framework_photogrammetry_session_limits(
                &mut maximum_input_image_dimension,
                &mut maximum_number_of_input_images,
            )
        };
        match result {
            0 => Ok(PhotogrammetrySessionLimits {
                maximum_input_image_dimension,
                maximum_number_of_input_images,
            }),
            1 => Err(PhotogrammetryLimitsError::NativeTypeUnavailable),
            _ => Err(PhotogrammetryLimitsError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(PhotogrammetryLimitsError::NativeApiUnavailable)
    }
}
