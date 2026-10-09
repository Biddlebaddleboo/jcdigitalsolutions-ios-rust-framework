#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow default Foundation Models readiness snapshot for iOS"]

/// Failure to read default Foundation Models availability
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundationModelsError {
    /// Foundation Models symbols are unavailable or the target is not iOS
    NativeApiUnavailable,
    /// Swift returned no model metadata or default model object
    NativeValueUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_system_language_model_default_create(model_out: *mut *mut c_void) -> u8;
    fn framework_system_language_model_is_available(
        model: *mut c_void,
        available_out: *mut u8,
    ) -> u8;
}

/// Reads whether the default on-device language model is entirely ready for requests
///
/// On iOS 26.0 and later this calls `SystemLanguageModel.default.isAvailable` through compiler-
/// derived Swift-call thunks. The result is only a point-in-time readiness value for the default
/// model. `false` does not distinguish device ineligibility, Apple Intelligence settings, a model
/// download, or another unavailable reason. The call creates no `LanguageModelSession`, prompt, or
/// inference request and shows no UI. It does not report support for every model use case, model
/// output quality, future availability, entitlement, or a successful later request
///
/// Apple currently labels this Foundation Models API as beta. The local compiler and SDK evidence
/// uses Xcode 26.6 / iOS 26.5, below the repository's Xcode 27.x baseline. Non-iOS targets return
/// `NativeApiUnavailable`
pub fn default_model_is_available() -> Result<bool, FoundationModelsError> {
    #[cfg(target_os = "ios")]
    {
        let mut raw_model: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes to this valid output pointer and checks weak-linked symbols
        let create_result =
            unsafe { framework_system_language_model_default_create(&mut raw_model) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }

        // SAFETY: the default getter returns an owned SystemLanguageModel reference; this wrapper
        // keeps it alive through the synchronous getter and releases it with swift-abi-core
        let Some(model) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_model.cast::<SwiftObject>())
        }) else {
            return Err(FoundationModelsError::NativeValueUnavailable);
        };

        let mut available = 0_u8;
        // SAFETY: the retained object is the exact class returned by `SystemLanguageModel.default`
        let read_result = unsafe {
            framework_system_language_model_is_available(
                model.as_ptr().cast::<c_void>(),
                &mut available,
            )
        };
        drop(model);
        if read_result != 0 {
            return Err(map_native_result(read_result));
        }
        match available {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(FoundationModelsError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(FoundationModelsError::NativeApiUnavailable)
    }
}

#[cfg(target_os = "ios")]
fn map_native_result(code: u8) -> FoundationModelsError {
    match code {
        1 => FoundationModelsError::NativeApiUnavailable,
        2 => FoundationModelsError::NativeValueUnavailable,
        _ => FoundationModelsError::NativeBridgeFailure,
    }
}
