use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Signed 64-bit native Speech authorization status.
pub type FrameworkIosSpeechAuthorizationStatus = i64;

/// Apple reports that no Speech authorization choice exists.
pub const FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED:
    FrameworkIosSpeechAuthorizationStatus = 0;
/// Apple reports that the app lacks Speech authorization.
pub const FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED: FrameworkIosSpeechAuthorizationStatus =
    1;
/// Apple reports a system restriction on Speech authorization.
pub const FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED:
    FrameworkIosSpeechAuthorizationStatus = 2;
/// Apple reports that the app has Speech authorization.
pub const FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED:
    FrameworkIosSpeechAuthorizationStatus = 3;

/// Reads the point-in-time iOS Speech authorization status.
///
/// Known native values are 0 (`NOT_DETERMINED`), 1 (`DENIED`), 2 (`RESTRICTED`), and 3
/// (`AUTHORIZED`). Any other signed native value is written unchanged. This reports saved
/// authorization only; it does not request permission, create a recognizer, accept audio, or start
/// recognition. It does not report service availability or guarantee recognition success.
///
/// On iOS below 10.0 this returns `FRAMEWORK_STATUS_UNAVAILABLE` and leaves the output zero. The
/// Apple header gives no main-thread requirement for the class method; this API adds no broader
/// thread-safety guarantee.
///
/// # Safety
/// A non-null `out_raw_status` must point to valid, writable, properly aligned `int64_t` storage
/// for this synchronous call. The caller must prevent unsynchronized concurrent access. The API
/// checks only nullness and does not retain the pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_speech_status_authorization_status(
    out_raw_status: *mut FrameworkIosSpeechAuthorizationStatus,
) -> FrameworkStatus {
    if out_raw_status.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises that a non-null output pointer is writable and aligned.
    unsafe { out_raw_status.write(0) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let status = ios_speech_status::authorization_status();
            let raw_status = match status {
                ios_speech_status::SpeechAuthorizationStatus::NotDetermined => {
                    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED
                }
                ios_speech_status::SpeechAuthorizationStatus::Denied => {
                    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED
                }
                ios_speech_status::SpeechAuthorizationStatus::Restricted => {
                    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED
                }
                ios_speech_status::SpeechAuthorizationStatus::Authorized => {
                    FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED
                }
                ios_speech_status::SpeechAuthorizationStatus::Unavailable => {
                    return FrameworkStatus::UNAVAILABLE;
                }
                ios_speech_status::SpeechAuthorizationStatus::Unknown(value) => value as i64,
            };
            // SAFETY: The caller supplied writable output storage above.
            unsafe { out_raw_status.write(raw_status) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
