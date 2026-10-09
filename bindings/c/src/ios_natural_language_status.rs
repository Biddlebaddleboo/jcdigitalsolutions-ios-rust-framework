use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Fixed-width C code for B59's English contextual-model asset status.
pub type FrameworkIosNaturalLanguageAssetStatus = u32;

/// Natural Language APIs are below their iOS 17.0 floor.
pub const FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE:
    FrameworkIosNaturalLanguageAssetStatus = 0;
/// The generated English language constant has no value.
pub const FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE:
    FrameworkIosNaturalLanguageAssetStatus = 1;
/// Apple returned no English contextual-model object.
pub const FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL:
    FrameworkIosNaturalLanguageAssetStatus = 2;
/// An English contextual model exists, but its assets are not on-device.
pub const FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE:
    FrameworkIosNaturalLanguageAssetStatus = 3;
/// An English contextual model exists and its assets are on-device.
pub const FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE:
    FrameworkIosNaturalLanguageAssetStatus = 4;

/// Reads B59's point-in-time English contextual-model asset status.
///
/// A successful call writes exactly one F29 status code: `UNAVAILABLE` below iOS 17.0,
/// `LANGUAGE_UNAVAILABLE` when the generated English language constant is absent, `NO_MODEL` when
/// Apple returns no model object, or `ASSETS_NOT_AVAILABLE` / `ASSETS_AVAILABLE` from
/// `hasAvailableAssets`. The status does not prove a future model load or vector result.
///
/// The iOS backend creates a temporary `NLContextualEmbedding` object and releases it before this
/// call returns. It does not load or unload a model, accept text, compute vectors, request assets,
/// or start a download. Calls are synchronous on the caller's thread; no main-thread rule or
/// broader thread-safety guarantee is added.
///
/// # Safety
/// A non-null `out_status` must point to valid, properly aligned, writable `uint32_t` storage for
/// the full synchronous call. The caller must prevent unsynchronized concurrent access. The API
/// checks only nullness and does not retain the output pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_natural_language_english_contextual_embedding_assets(
    out_status: *mut FrameworkIosNaturalLanguageAssetStatus,
) -> FrameworkStatus {
    if out_status.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises valid, properly aligned, writable output storage.
    unsafe { out_status.write(0) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let status = ios_natural_language_status::english_contextual_embedding_assets();
            let raw_status = match status {
                ios_natural_language_status::EnglishEmbeddingAssetStatus::Unavailable => {
                    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE
                }
                ios_natural_language_status::EnglishEmbeddingAssetStatus::LanguageUnavailable => {
                    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE
                }
                ios_natural_language_status::EnglishEmbeddingAssetStatus::NoModel => {
                    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL
                }
                ios_natural_language_status::EnglishEmbeddingAssetStatus::AssetsNotAvailable => {
                    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE
                }
                ios_natural_language_status::EnglishEmbeddingAssetStatus::AssetsAvailable => {
                    FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE
                }
            };
            // SAFETY: The caller supplied valid, properly aligned, writable output storage above.
            unsafe { out_status.write(raw_status) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
