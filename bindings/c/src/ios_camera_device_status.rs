use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether AVFoundation reports a current default video capture device.
///
/// This is a point-in-time device-presence query only. It does not report camera authorization,
/// capture readiness, configured session state, or future availability.
///
/// # Safety
/// `out_present` must be non-null and point to valid, properly aligned writable memory for one
/// byte for the duration of the synchronous call. The caller must prevent unsynchronized
/// concurrent access to that byte. This function checks only nullness and does not retain the
/// pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_camera_device_status_has_default_video_capture_device(
    out_present: *mut u8,
) -> FrameworkStatus {
    if out_present.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one writable byte at this non-null pointer.
    unsafe { out_present.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let present = ::ios_camera_device_status::has_default_video_capture_device();
            // SAFETY: The caller supplied one writable byte above.
            unsafe { out_present.write(u8::from(present)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
