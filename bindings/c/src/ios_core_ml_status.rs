use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether Core ML reports at least one available compute device
///
/// A true result is only a point-in-time nonempty-list snapshot from
/// `MLModel.availableComputeDevices`. It does not establish compatibility or success for any model
/// or operation. The iOS backend returns false below iOS 17.0
///
/// # Safety
/// `out_available` must be non-null and point to valid, properly aligned writable memory for one
/// byte for the duration of the synchronous call. The caller must prevent unsynchronized
/// concurrent access to that byte. This function checks only nullness and does not retain the
/// pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_core_ml_status_has_available_compute_device(
    out_available: *mut u8,
) -> FrameworkStatus {
    if out_available.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one writable byte at this non-null pointer.
    unsafe { out_available.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let available = ::ios_core_ml_status::has_available_compute_device();
            // SAFETY: The caller supplied one writable byte above.
            unsafe { out_available.write(u8::from(available)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
