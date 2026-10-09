use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

/// Writes whether iOS reports hardware decode support for one FourCC codec.
///
/// The codec is interpreted as a big-endian FourCC whose bytes are in display order. The query
/// reports only the current system predicate; it does not reserve decoder resources or guarantee
/// that a decoder session can later be created.
///
/// # Safety
/// When non-null, `out_supported` must address one valid, properly aligned writable byte for the
/// full synchronous call. The caller must prevent unsynchronized concurrent access. The API checks
/// only nullness and does not retain the output pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_videotoolbox_hardware_decode_supported(
    codec_fourcc: u32,
    out_supported: *mut u8,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one valid, properly aligned writable byte at this non-null pointer.
    unsafe { out_supported.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let codec = framework_media::VideoCodecType::from_fourcc(codec_fourcc.to_be_bytes());
            match ::ios_media::hardware_decode_support(codec) {
                Some(support) => {
                    // SAFETY: The caller supplied one valid, properly aligned writable byte above.
                    unsafe { out_supported.write(u8::from(support.is_supported())) };
                    FrameworkStatus::OK
                }
                None => FrameworkStatus::UNAVAILABLE,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = codec_fourcc;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
