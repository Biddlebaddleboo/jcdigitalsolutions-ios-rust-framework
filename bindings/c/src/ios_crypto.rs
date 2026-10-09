use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkSlice, FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use ::ios_crypto::Sha256Error;

const DIGEST_SIZE: usize = 32;

fn input_output_overlap(input: *const u8, input_len: usize, output: *mut u8) -> Option<bool> {
    let input_start = input as usize;
    let output_start = output as usize;
    let input_end = input_start.checked_add(input_len)?;
    let output_end = output_start.checked_add(DIGEST_SIZE)?;
    Some(input_len != 0 && input_start < output_end && output_start < input_end)
}

/// Compute one iOS CommonCrypto SHA-256 digest into caller-owned storage.
///
/// This exposes only `CC_SHA256` through the optional `ios-crypto` feature. It adds no portable
/// crypto contract, replacement/default claim, digest-parity result, or performance claim.
///
/// # Safety
/// On iOS, a nonzero input within `UINT32_MAX` and Rust's slice limit must describe readable bytes
/// that remain immutable for the full synchronous call. `out_digest` must be non-null and
/// name 32 writable bytes for the full call. The output range must not overlap nonempty input.
/// The app must keep input immutable and prevent unsynchronized output access for the full call.
/// The wrapper checks pointer presence, byte-length bounds, checked address ranges, and
/// input/output overlap, but cannot prove that memory is valid, readable, writable, or live. Neither
/// pointer is retained after return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_crypto_sha256(
    input: FrameworkSlice,
    out_digest: *mut u8,
) -> FrameworkStatus {
    if out_digest.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Ok(input_len) = usize::try_from(input.length()) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if input_len > isize::MAX as usize {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if input_len != 0 && input.data().is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if !matches!(
        input_output_overlap(input.data(), input_len, out_digest),
        Some(false)
    ) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises 32 writable bytes and the checked ranges do not overlap.
    unsafe { ptr::write_bytes(out_digest, 0, DIGEST_SIZE) };

    if input.length() > u64::from(u32::MAX) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            // SAFETY: The caller guarantees readable, immutable input for the stated length;
            // null is accepted only for the zero-length case, which `as_bytes` handles.
            let Some(bytes) = (unsafe { input.as_bytes() }) else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            match ::ios_crypto::sha256(bytes) {
                Ok(digest) => {
                    // SAFETY: The caller supplied 32 writable bytes; the local digest is a
                    // distinct fixed-size array and the output was checked disjoint from input.
                    unsafe { ptr::copy_nonoverlapping(digest.as_ptr(), out_digest, DIGEST_SIZE) };
                    FrameworkStatus::OK
                }
                Err(Sha256Error::InputTooLarge) => FrameworkStatus::INVALID_ARGUMENT,
                Err(Sha256Error::NativeFailure) => FrameworkStatus::PLATFORM_ERROR,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = input;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
