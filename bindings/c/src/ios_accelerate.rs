use core::mem::{align_of, size_of};
#[cfg(target_os = "ios")]
use core::panic::AssertUnwindSafe;
#[cfg(target_os = "ios")]
use core::slice;
use framework_abi::FrameworkStatus;

#[cfg(target_os = "ios")]
use framework_abi::catch_unwind_status;

#[cfg(target_os = "ios")]
use ::ios_accelerate::VectorAddError;

fn span_range(pointer: *const f32, length: usize, byte_length: usize) -> Option<(usize, usize)> {
    let start = pointer as usize;
    if length != 0 && (pointer.is_null() || start % align_of::<f32>() != 0) {
        return None;
    }
    Some((start, start.checked_add(byte_length)?))
}

fn ranges_overlap(left: (usize, usize), right: (usize, usize)) -> bool {
    left.0 != left.1 && right.0 != right.1 && left.0 < right.1 && right.0 < left.1
}

/// Add equal-length single-precision vectors through the opt-in iOS Accelerate backend.
///
/// Lengths are element counts, not byte counts. The operation overwrites each output element only
/// on success. It adds no portable math contract, numerical-parity claim, or performance claim.
///
/// # Safety
/// For a nonzero equal count on iOS, `a` and `b` must name readable aligned `f32` arrays and
/// `output` must name a writable aligned array for the full call. Output must be disjoint from
/// both inputs; the input ranges may overlap. The app must keep both inputs immutable and prevent
/// unsynchronized output access for the full call. The wrapper checks count equality, byte-length bounds,
/// non-nullness, alignment, address-range overflow, and output/input overlap, but cannot prove that
/// memory is valid, readable, writable, or live. No pointer is retained after return. Null pointers
/// are accepted when all lengths are zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_accelerate_vector_add(
    a: *const f32,
    a_length: u64,
    b: *const f32,
    b_length: u64,
    output: *mut f32,
    output_length: u64,
) -> FrameworkStatus {
    if a_length != b_length || a_length != output_length {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Ok(length) = usize::try_from(a_length) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    let Some(byte_length) = length.checked_mul(size_of::<f32>()) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if byte_length > isize::MAX as usize {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Some(a_range) = span_range(a, length, byte_length) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    let Some(b_range) = span_range(b, length, byte_length) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    let Some(output_range) = span_range(output.cast_const(), length, byte_length) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    if ranges_overlap(output_range, a_range) || ranges_overlap(output_range, b_range) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    #[cfg(target_os = "ios")]
    {
        if length == 0 {
            return FrameworkStatus::OK;
        }
        catch_unwind_status(AssertUnwindSafe(|| {
            // SAFETY: The caller promises aligned live ranges of `length` f32 elements; checked
            // byte ranges fit a Rust slice and output is disjoint from both shared inputs.
            let a = unsafe { slice::from_raw_parts(a, length) };
            // SAFETY: The caller promises aligned live input bytes; shared input ranges may alias.
            let b = unsafe { slice::from_raw_parts(b, length) };
            // SAFETY: The caller promises a live writable output range disjoint from both inputs.
            let output = unsafe { slice::from_raw_parts_mut(output, length) };
            match ::ios_accelerate::vector_add(a, b, output) {
                Ok(()) => FrameworkStatus::OK,
                Err(VectorAddError::LengthMismatch | VectorAddError::LengthTooLarge) => {
                    FrameworkStatus::INVALID_ARGUMENT
                }
            }
        }))
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (a, b, output, length);
        FrameworkStatus::UNSUPPORTED
    }
}
