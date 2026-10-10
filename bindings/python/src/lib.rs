#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![doc = "Optional PyO3 extension for framework-owned bytes."]

use framework_abi::{
    FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, framework_owned_buffer_copy,
};
use pyo3::exceptions::{PyMemoryError, PyOverflowError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyBytesMethods, PyModule};

/// A Python object that owns bytes through the framework's allocator and destructor contract.
#[pyclass(frozen, module = "framework_python", name = "OwnedBytes")]
pub struct OwnedBytes {
    buffer: FrameworkOwnedBuffer,
}

// SAFETY: this type only stores an immutable FrameworkOwnedBuffer created by the framework's
// Vec<u8>-backed copy function. Vec<u8> allocations may be dropped on any thread; methods only
// borrow the bytes, and FrameworkOwnedBuffer's destructor releases that same allocation.
unsafe impl Send for OwnedBytes {}

// SAFETY: all methods access the owned allocation immutably. PyO3's frozen-class borrow rules
// prevent destruction while a method holds a shared borrow.
unsafe impl Sync for OwnedBytes {}

impl OwnedBytes {
    fn copy_from(bytes: &[u8]) -> PyResult<FrameworkOwnedBuffer> {
        let slice = FrameworkSlice::from_bytes(bytes)
            .ok_or_else(|| PyOverflowError::new_err("byte length exceeds the framework ABI"))?;
        let mut buffer = FrameworkOwnedBuffer::default();
        // SAFETY: the stack output is aligned, writable, empty, and disjoint from the input. The
        // ABI copies synchronously and retains neither the descriptor nor the input pointer.
        let status = unsafe { framework_owned_buffer_copy(slice, &mut buffer) };
        if status == FrameworkStatus::OK {
            return Ok(buffer);
        }
        if status == FrameworkStatus::RESOURCE_EXHAUSTED {
            return Err(PyMemoryError::new_err(
                "framework could not allocate owned bytes",
            ));
        }
        if status == FrameworkStatus::INVALID_ARGUMENT {
            return Err(PyOverflowError::new_err(
                "byte length exceeds the framework slice limit",
            ));
        }
        Err(PyRuntimeError::new_err(format!(
            "framework_owned_buffer_copy failed with status {}",
            status.code()
        )))
    }

    fn bytes(&self) -> PyResult<&[u8]> {
        self.buffer
            .as_bytes()
            .ok_or_else(|| PyRuntimeError::new_err("framework-owned byte buffer is invalid"))
    }
}

#[pymethods]
impl OwnedBytes {
    /// Copies a Python `bytes` value into framework-owned storage.
    #[new]
    fn new(value: &Bound<'_, PyBytes>) -> PyResult<Self> {
        Ok(Self {
            buffer: Self::copy_from(value.as_bytes())?,
        })
    }

    /// Returns the byte count as a Python integer.
    #[getter]
    fn length(&self) -> u64 {
        self.buffer.length()
    }

    /// Returns the byte count using Python's `len()` protocol.
    fn __len__(&self) -> PyResult<usize> {
        usize::try_from(self.buffer.length())
            .map_err(|_| PyOverflowError::new_err("byte length exceeds Python's size limit"))
    }

    /// Copies the value into a Python `bytes` object.
    fn __bytes__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.bytes()?;
        PyBytes::new_with(py, bytes.len(), |output| {
            output.copy_from_slice(bytes);
            Ok(())
        })
    }
}

/// Initializes the optional `framework_python` extension module.
#[pymodule]
fn framework_python(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<OwnedBytes>()?;
    Ok(())
}
