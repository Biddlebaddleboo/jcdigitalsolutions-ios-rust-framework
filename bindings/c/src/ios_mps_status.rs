use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether iOS MPS returns a preferred device with default options.
///
/// This reports only point-in-time preferred-device presence. It does not submit GPU work or
/// report support for any MPS operation, model, or workload.
///
/// # Safety
/// A non-null `out_available` must address valid, properly aligned writable memory for one byte
/// during this synchronous call. The client must prevent unsynchronized concurrent access.
/// It checks nullness only; it does not retain the output address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_mps_status_preferred_device_available(
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
            let available = ::ios_mps_status::preferred_mps_device_available();
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
