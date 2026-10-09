use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Fixed-width Boolean result for the Tap to Pay device-model predicate.
pub type FrameworkIosProximityReaderBoolean = u8;

/// Writes whether the current iPhone model supports Tap to Pay on iPhone.
///
/// This reports Apple's `PaymentCardReader.isSupported` device-model predicate only. It does not
/// report operating-system, entitlement, merchant, region, provider, or payment readiness.
///
/// # Safety
/// `out_supported` must address valid, properly aligned writable `uint8_t` memory for the full
/// synchronous call. The caller must prevent unsynchronized access to the output. The API does not
/// retain the output address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_proximity_reader_tap_to_pay_device_model_supported(
    out_supported: *mut FrameworkIosProximityReaderBoolean,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises valid, aligned writable output memory for this call.
    unsafe { out_supported.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            match ::ios_proximity_reader::tap_to_pay_device_model_supported() {
                Some(supported) => {
                    // SAFETY: The caller supplied valid, aligned writable output storage above.
                    unsafe { out_supported.write(u8::from(supported)) };
                    FrameworkStatus::OK
                }
                None => FrameworkStatus::UNAVAILABLE,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
