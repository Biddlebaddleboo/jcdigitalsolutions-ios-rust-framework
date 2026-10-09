#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow AdAttributionKit app-impression support snapshot for iOS"]

/// Failure to read AdAttributionKit app-impression support
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdAttributionError {
    /// The iOS 18.0 API symbol is unavailable or the target is not iOS
    NativeApiUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_ad_attribution_app_impression_is_supported(supported_out: *mut u8) -> u8;
}

/// Reads whether AdAttributionKit supports app impressions on the current device
///
/// On iOS 18.0 and later this returns only Apple's `AppImpression.isSupported` value. It does not
/// create an impression, validate a JWS, record a view or tap, generate an attribution token, make a
/// network request, or report ad-network registration, campaign eligibility, postback delivery,
/// consent, entitlement, or general AdAttributionKit readiness. Older iOS releases return
/// `NativeApiUnavailable` because the Swift API symbol is weak-linked. Non-iOS targets return
/// `NativeApiUnavailable`
pub fn app_impression_is_supported() -> Result<bool, AdAttributionError> {
    #[cfg(target_os = "ios")]
    {
        let mut supported = 0_u8;
        // SAFETY: the C bridge accepts this valid output pointer and checks the weak-linked API
        let result =
            unsafe { framework_ad_attribution_app_impression_is_supported(&mut supported) };
        match (result, supported) {
            (0, 0) => Ok(false),
            (0, 1) => Ok(true),
            (1, _) => Err(AdAttributionError::NativeApiUnavailable),
            _ => Err(AdAttributionError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(AdAttributionError::NativeApiUnavailable)
    }
}
