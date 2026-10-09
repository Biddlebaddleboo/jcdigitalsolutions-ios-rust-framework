use core::ffi::c_void;

#[link(name = "System")]
unsafe extern "C" {
    #[link_name = "CC_SHA256"]
    fn cc_sha256(data: *const c_void, len: u32, digest: *mut u8) -> *mut u8;
}

/// An error returned by [`sha256`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sha256Error {
    /// The input length exceeds CommonCrypto's 32-bit `CC_LONG` parameter.
    InputTooLarge,
    /// CommonCrypto did not return the caller-provided digest buffer.
    NativeFailure,
}

/// Compute one SHA-256 digest with Apple's `CC_SHA256` API.
///
/// The input is borrowed for the duration of the call. The result owns a fixed 32-byte digest.
/// Input longer than `u32::MAX` bytes is rejected before the native call. Empty input is passed
/// with length zero and a valid pointer. The Rust wrapper creates no heap-backed input/output
/// buffer; this does not make a claim about CommonCrypto's internal allocation behavior. The
/// wrapper adds no main-thread check or cross-call state, and makes no added thread-safety claim.
///
/// This is an iOS-only Apple-backed operation, not a portable crypto contract or a replacement
/// implementation. It makes no parity, certification, or performance claim.
///
/// # Errors
///
/// Returns [`Sha256Error::InputTooLarge`] if the input length cannot fit `CC_LONG`, or
/// [`Sha256Error::NativeFailure`] if the native function does not return the supplied output
/// buffer.
pub fn sha256(input: &[u8]) -> Result<[u8; 32], Sha256Error> {
    let len = u32::try_from(input.len()).map_err(|_| Sha256Error::InputTooLarge)?;
    let mut digest = [0_u8; 32];
    let digest_ptr = digest.as_mut_ptr();
    let empty_input = 0_u8;
    let input_ptr: *const u8 = if input.is_empty() {
        &empty_input
    } else {
        input.as_ptr()
    };

    // SAFETY: `input` is borrowed for `len` bytes, and `digest_ptr` points to a writable
    // 32-byte array. For empty input, `input_ptr` points to a live local byte while `len` is
    // zero. `CC_SHA256` reads the input, writes its fixed-size digest, and retains neither
    // pointer. The lengths and output size match the public CommonCrypto signature.
    let returned = unsafe { cc_sha256(input_ptr.cast(), len, digest_ptr) };
    if returned != digest_ptr {
        return Err(Sha256Error::NativeFailure);
    }

    Ok(digest)
}
