use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether StoreKit reports that the person can authorize purchases.
///
/// A false value may also mean the weak iOS 15.0 StoreKit symbol is absent. It does not report
/// product availability, account identity, entitlement state, or transaction success.
///
/// # Safety
/// `out_can_make_payments` must be non-null and point to valid, properly aligned writable memory
/// for one byte for the duration of this synchronous call. The caller must prevent unsynchronized
/// concurrent access to that byte. This function checks only nullness and does not retain the
/// pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_storekit2_status_can_make_payments(
    out_can_make_payments: *mut u8,
) -> FrameworkStatus {
    if out_can_make_payments.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one writable byte at this non-null pointer.
    unsafe { out_can_make_payments.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let can_make_payments = ::ios_storekit2_status::can_make_payments();
            // SAFETY: The caller supplied one writable byte above.
            unsafe { out_can_make_payments.write(u8::from(can_make_payments)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
