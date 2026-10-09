use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether the iOS runtime lists a text-recognition request revision as supported.
///
/// This checks only membership in `VNRecognizeTextRequest.supportedRevisions`; it does not run
/// recognition or report model readiness.
///
/// # Safety
/// `out_supported` must address valid, properly aligned writable `uint8_t` memory for the full
/// synchronous call. The caller must prevent unsynchronized access to the output. The API does not
/// retain the output address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_vision_text_recognition_revision_is_supported(
    revision: u32,
    out_supported: *mut u8,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises valid, aligned writable output memory for this call.
    unsafe { out_supported.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            match ::ios_vision::text_recognition_revision_support(revision) {
                Some(support) => {
                    // SAFETY: The caller supplied valid, aligned writable output storage above.
                    unsafe { out_supported.write(u8::from(support.is_supported())) };
                    FrameworkStatus::OK
                }
                None => FrameworkStatus::UNAVAILABLE,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = revision;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
