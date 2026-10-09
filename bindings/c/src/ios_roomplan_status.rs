use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether the RoomPlan platform query reports device support.
///
/// This calls only B61's `RoomCaptureSession.isSupported` route. It does not create or run a
/// capture session, read camera or LiDAR frames, request permission, present UI, or start a scan.
///
/// # Safety
/// `out_supported` must be non-null and point to valid, properly aligned writable memory for one
/// byte for the duration of this synchronous call. The caller must prevent unsynchronized
/// concurrent access to that byte. This function checks only nullness and does not retain the pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_roomplan_status_is_supported(
    out_supported: *mut u8,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one writable byte at this non-null pointer.
    unsafe { out_supported.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(support) = ios_roomplan::device_support() else {
                return FrameworkStatus::UNSUPPORTED;
            };
            // SAFETY: The caller supplied one writable byte above.
            unsafe { out_supported.write(u8::from(support.is_supported())) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
