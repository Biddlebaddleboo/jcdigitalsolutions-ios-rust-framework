use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Signed 64-bit native MediaPlayer library authorization status.
pub type FrameworkIosMediaLibraryAuthorizationStatus = i64;

/// The user has not chosen whether to authorize media-library access.
pub const FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_NOT_DETERMINED:
    FrameworkIosMediaLibraryAuthorizationStatus = 0;
/// The app may not access media-library items.
pub const FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_DENIED:
    FrameworkIosMediaLibraryAuthorizationStatus = 1;
/// Restrictions limit which media-library content the app may access.
pub const FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_RESTRICTED:
    FrameworkIosMediaLibraryAuthorizationStatus = 2;
/// The app may access media-library items.
pub const FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_AUTHORIZED:
    FrameworkIosMediaLibraryAuthorizationStatus = 3;

/// Reads the point-in-time iOS MediaPlayer library authorization status.
///
/// Known native values are 0 (`NOT_DETERMINED`), 1 (`DENIED`), 2 (`RESTRICTED`), and 3
/// (`AUTHORIZED`). Any other signed native value is returned unchanged.
///
/// # Safety
/// A non-null `out_raw_status` must point to writable, aligned `int64_t` storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_media_library_authorization_status(
    out_raw_status: *mut FrameworkIosMediaLibraryAuthorizationStatus,
) -> FrameworkStatus {
    if out_raw_status.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises that a non-null output pointer is writable and aligned.
    unsafe { out_raw_status.write(0) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let status = ios_media_library_status::media_library_authorization_status();
            // SAFETY: The caller supplied writable output storage above.
            unsafe { out_raw_status.write(status.raw_value()) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
