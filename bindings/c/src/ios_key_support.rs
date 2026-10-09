use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkSlice, FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use ::framework_key_support::P256PublicKey;

const P256_PUBLIC_KEY_X963_LEN: usize = 65;

fn input_output_overlap(input: *const u8, input_len: usize, output: *mut u8) -> Option<bool> {
    let input_start = input as usize;
    let output_start = output as usize;
    let input_end = input_start.checked_add(input_len)?;
    let output_end = output_start.checked_add(1)?;
    Some(input_start < output_end && output_start < input_end)
}

/// Writes whether iOS Security reports this P-256 public key suitable for ECDSA/SHA-256 message
/// verification.
///
/// This imports the caller's public key only for the query. It does not verify a signature,
/// create or use private key material, persist a key, or guarantee a later verification call.
///
/// # Safety
/// `x963_public_key` must describe exactly 65 readable bytes that remain immutable for this call.
/// A non-null `out_supported` must address valid, properly aligned writable memory for one byte
/// during this synchronous call and must not overlap the input. The client must prevent
/// unsynchronized concurrent access. The API checks pointer-range arithmetic and overlap but
/// cannot validate the memory; it does not retain the output address.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_key_support_p256_ecdsa_sha256_message_supported(
    x963_public_key: FrameworkSlice,
    out_supported: *mut u8,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Ok(input_len) = usize::try_from(x963_public_key.length()) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if input_len != P256_PUBLIC_KEY_X963_LEN || x963_public_key.data().is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if !matches!(
        input_output_overlap(x963_public_key.data(), input_len, out_supported),
        Some(false)
    ) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises one writable byte and the checked ranges do not overlap.
    unsafe { ptr::write(out_supported, 0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            // SAFETY: The caller promises 65 readable bytes; u8 has alignment one.
            let key_bytes = unsafe {
                &*x963_public_key
                    .data()
                    .cast::<[u8; P256_PUBLIC_KEY_X963_LEN]>()
            };
            let Ok(public_key) = P256PublicKey::from_x963_uncompressed(key_bytes) else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            match ::ios_key_support::IosP256VerificationSupport::new().query(public_key) {
                Ok(support) => {
                    // SAFETY: The caller supplied one writable byte above; the input is disjoint.
                    unsafe { out_supported.write(u8::from(support.is_supported())) };
                    FrameworkStatus::OK
                }
                Err(error) => FrameworkStatus::from_error(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = x963_public_key;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
