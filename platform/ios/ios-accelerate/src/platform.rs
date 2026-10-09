use core::ffi::{c_long, c_ulong};

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn vDSP_vadd(
        a: *const f32,
        stride_a: c_long,
        b: *const f32,
        stride_b: c_long,
        output: *mut f32,
        stride_output: c_long,
        length: c_ulong,
    );
}

/// Error returned when vector lengths do not match or do not fit `vDSP_Length`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorAddError {
    /// The two inputs and output do not have the same length.
    LengthMismatch,
    /// The element count does not fit the platform's `vDSP_Length` type.
    LengthTooLarge,
}

/// Write the elementwise sum of `a` and `b` into `output` with unit stride.
///
/// All three slices must have equal lengths. A length mismatch returns before the
/// Accelerate call. Equal empty slices return successfully without an FFI call.
/// For non-empty slices, the function reads both inputs and overwrites every
/// output element. The Rust wrapper allocates no buffer and retains no pointer after return.
///
/// The function has no thread-affinity requirement or shared mutable state.
/// Safe Rust borrowing prevents the output slice from aliasing either input.
/// This API makes no bitwise-parity or performance claim.
///
/// # Errors
///
/// Returns [`VectorAddError::LengthMismatch`] when slice lengths differ, or
/// [`VectorAddError::LengthTooLarge`] when the element count cannot fit
/// `vDSP_Length`.
pub fn vector_add(a: &[f32], b: &[f32], output: &mut [f32]) -> Result<(), VectorAddError> {
    if a.len() != b.len() || a.len() != output.len() {
        return Err(VectorAddError::LengthMismatch);
    }
    if a.is_empty() {
        return Ok(());
    }
    let length = c_ulong::try_from(a.len()).map_err(|_| VectorAddError::LengthTooLarge)?;

    // SAFETY: equal slice lengths cover `length` elements at unit stride, the
    // borrows keep all buffers live, and safe Rust excludes output/input aliasing.
    unsafe {
        vDSP_vadd(a.as_ptr(), 1, b.as_ptr(), 1, output.as_mut_ptr(), 1, length);
    }
    Ok(())
}
