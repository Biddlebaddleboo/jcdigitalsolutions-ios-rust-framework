use alloc::vec::Vec;
use core::convert::TryFrom;
use framework_data::{ByteView, OwnedBytes};
use objc2_core_foundation::{CFData, CFIndex, CFRange, CFRetained};

/// An immutable, owned Core Foundation byte value
///
/// Construction and extraction perform explicit byte copies. This type does not expose mutable
/// storage, a raw pointer, or a zero-copy conversion
pub struct IosData {
    data: CFRetained<CFData>,
}

impl IosData {
    /// Copies a portable byte view into a new immutable `CFData`
    ///
    /// The native allocation can fail, and the byte count must fit Core Foundation's `CFIndex`
    pub fn copy_from(bytes: ByteView<'_>) -> Result<Self, IosDataError> {
        let source = bytes.as_bytes();
        let length = CFIndex::try_from(source.len()).map_err(|_| IosDataError::LengthOverflow)?;
        let empty = 0_u8;
        let source_pointer = if source.is_empty() {
            &empty as *const u8
        } else {
            source.as_ptr()
        };
        // SAFETY: The borrowed slice pointer is valid for `length` bytes; the default allocator is permitted
        // `source_pointer` points to a live byte even when the input slice is empty
        let data = unsafe { CFData::new(None, source_pointer, length) }
            .ok_or(IosDataError::NativeAllocation)?;
        Ok(Self { data })
    }

    /// Copies this `CFData` value into portable owned bytes
    ///
    /// Rust storage is reserved with `try_reserve_exact` before any byte copy
    pub fn copy_to_owned(&self) -> Result<OwnedBytes, IosDataError> {
        let native_length = self.data.length();
        let length = usize::try_from(native_length).map_err(|_| IosDataError::LengthOverflow)?;
        let native_length = CFIndex::try_from(length).map_err(|_| IosDataError::LengthOverflow)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| IosDataError::RustAllocation)?;

        if length > 0 {
            bytes.resize(length, 0);
            // SAFETY: `bytes` has `length` writable bytes after the successful exact reserve
            unsafe {
                self.data
                    .bytes(CFRange::new(0, native_length), bytes.as_mut_ptr())
            };
        }

        Ok(OwnedBytes::from_vec(bytes))
    }

    /// Borrows the native `CFData` handle for the lifetime of this owner
    pub fn as_cf_data(&self) -> &CFData {
        &self.data
    }
}

/// A failure from a checked Core Foundation byte copy
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum IosDataError {
    /// The byte count does not fit `CFIndex` or `usize`
    LengthOverflow,
    /// `CFDataCreate` could not obtain its native allocation
    NativeAllocation,
    /// Rust could not reserve the output byte buffer
    RustAllocation,
}
