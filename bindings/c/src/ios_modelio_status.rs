use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkStatus, FrameworkStr, catch_unwind_status};

const OUTPUT_SIZE: usize = 1;

fn input_output_overlap(input: *const u8, input_len: usize, output: *mut u8) -> Option<bool> {
    let input_start = input as usize;
    let output_start = output as usize;
    let input_end = input_start.checked_add(input_len)?;
    let output_end = output_start.checked_add(OUTPUT_SIZE)?;
    Some(input_len != 0 && input_start < output_end && output_start < input_end)
}

/// Ask whether ModelIO can read asset files with the supplied extension.
///
/// This forwards valid UTF-8 to `MDLAsset.canImportFileExtension` without normalization. It does
/// not create an asset, access a URL or file data, or establish that a specific asset is valid.
///
/// # Safety
/// `extension` must reference readable UTF-8 bytes that remain immutable for this call when its
/// length is nonzero. `out_supported` must address valid, properly aligned writable memory for one
/// byte for the full synchronous call and must not overlap a nonempty input range. The caller must
/// prevent unsynchronized access to either region. This API checks range arithmetic and input/output
/// overlap, but cannot prove that memory is valid, aligned, writable, or live. Neither pointer is
/// retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_modelio_can_import_file_extension(
    extension: FrameworkStr,
    out_supported: *mut u8,
) -> FrameworkStatus {
    if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Ok(input_len) = usize::try_from(extension.length()) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if input_len > isize::MAX as usize {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if input_len != 0 && extension.data().is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if !matches!(
        input_output_overlap(extension.data(), input_len, out_supported),
        Some(false)
    ) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises one valid, aligned writable byte for this call and the checked
    // ranges do not overlap.
    unsafe { ptr::write_bytes(out_supported, 0, OUTPUT_SIZE) };

    // SAFETY: The caller promises valid UTF-8 readable for `input_len` bytes; null is accepted
    // only for the zero-length case, which `as_str` handles.
    let Some(extension) = (unsafe { extension.as_str() }) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let supported = ::ios_modelio_status::can_import_file_extension(extension);
            // SAFETY: The caller supplied one valid, aligned writable byte above; no pointer is
            // retained.
            unsafe { out_supported.write(u8::from(supported)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = extension;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
