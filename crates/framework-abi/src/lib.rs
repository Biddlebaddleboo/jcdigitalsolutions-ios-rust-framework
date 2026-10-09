#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Fixed-width C ABI values, pointer-length views, and explicit owned-buffer destruction."]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;
use framework_core::{Error, ErrorKind, OperationId};

/// The current major ABI version.
pub const ABI_VERSION_MAJOR: u32 = 1;
/// The current minor ABI version.
pub const ABI_VERSION_MINOR: u32 = 1;

/// A fixed-width status value for C callers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct FrameworkStatus(u32);

impl FrameworkStatus {
    /// Successful completion.
    pub const OK: Self = Self(0);
    /// An argument or value is invalid.
    pub const INVALID_ARGUMENT: Self = Self(1);
    /// The target does not support the requested operation.
    pub const UNSUPPORTED: Self = Self(2);
    /// The operation is unavailable in the current context.
    pub const UNAVAILABLE: Self = Self(3);
    /// User authorization was denied or is insufficient.
    pub const PERMISSION_DENIED: Self = Self(4);
    /// The operation was cancelled.
    pub const CANCELLED: Self = Self(5);
    /// The operation exceeded a time limit.
    pub const TIMEOUT: Self = Self(6);
    /// The requested resource does not exist.
    pub const NOT_FOUND: Self = Self(7);
    /// The requested resource identity already exists.
    pub const ALREADY_EXISTS: Self = Self(8);
    /// A bounded resource could not be obtained.
    pub const RESOURCE_EXHAUSTED: Self = Self(9);
    /// A platform reported an otherwise unclassified error.
    pub const PLATFORM_ERROR: Self = Self(10);
    /// An internal framework operation failed.
    pub const INTERNAL_ERROR: Self = Self(11);
    /// A panic caught by the optional `std` C ABI boundary helper.
    pub const PANIC: Self = Self(12);

    /// Creates a status from its fixed-width numeric code.
    pub const fn from_code(code: u32) -> Self {
        Self(code)
    }

    /// Returns the fixed-width numeric code.
    pub const fn code(self) -> u32 {
        self.0
    }

    /// Maps a portable error category to its stable C status.
    pub const fn from_error(error: Error) -> Self {
        match error.kind() {
            ErrorKind::InvalidInput => Self::INVALID_ARGUMENT,
            ErrorKind::Unsupported => Self::UNSUPPORTED,
            ErrorKind::Unavailable => Self::UNAVAILABLE,
            ErrorKind::PermissionDenied => Self::PERMISSION_DENIED,
            ErrorKind::Cancelled => Self::CANCELLED,
            ErrorKind::Timeout => Self::TIMEOUT,
            ErrorKind::NotFound => Self::NOT_FOUND,
            ErrorKind::AlreadyExists => Self::ALREADY_EXISTS,
            ErrorKind::ResourceExhausted => Self::RESOURCE_EXHAUSTED,
            ErrorKind::Platform => Self::PLATFORM_ERROR,
            ErrorKind::Unknown | ErrorKind::Internal => Self::INTERNAL_ERROR,
            _ => Self::INTERNAL_ERROR,
        }
    }
}

/// A borrowed byte span represented by a pointer and an explicit byte length.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FrameworkSlice {
    data: *const u8,
    length: u64,
}

impl FrameworkSlice {
    /// Creates a borrowed view of bytes. The source slice must outlive foreign use of this value.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            data: bytes.as_ptr(),
            length: u64::try_from(bytes.len()).ok()?,
        })
    }

    /// Returns the data pointer without dereferencing it.
    pub const fn data(self) -> *const u8 {
        self.data
    }

    /// Returns the fixed-width byte length.
    pub const fn length(self) -> u64 {
        self.length
    }

    /// Creates a borrowed Rust slice from a foreign pointer-length pair.
    ///
    /// # Safety
    /// `data` must be null only when `length` is zero; otherwise it must be aligned, readable for
    /// `length` bytes, and valid for the returned lifetime. The memory must not mutate for that
    /// lifetime except through Rust's interior-mutability rules.
    pub unsafe fn as_bytes<'a>(self) -> Option<&'a [u8]> {
        let length = usize::try_from(self.length).ok()?;
        if length == 0 {
            return Some(&[]);
        }
        if self.data.is_null() {
            return None;
        }
        // SAFETY: the caller upholds the documented pointer, length, alignment, and lifetime rules.
        Some(unsafe { core::slice::from_raw_parts(self.data, length) })
    }
}

/// A borrowed UTF-8 string represented by a pointer and an explicit byte length.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FrameworkStr {
    data: *const u8,
    length: u64,
}

impl FrameworkStr {
    /// Creates a borrowed UTF-8 view. The source string must outlive foreign use of this value.
    pub fn from_utf8(value: &str) -> Option<Self> {
        Some(Self {
            data: value.as_ptr(),
            length: u64::try_from(value.len()).ok()?,
        })
    }

    /// Returns the data pointer without dereferencing it.
    pub const fn data(self) -> *const u8 {
        self.data
    }

    /// Returns the fixed-width byte length.
    pub const fn length(self) -> u64 {
        self.length
    }

    /// Creates a borrowed UTF-8 string from a foreign pointer-length pair.
    ///
    /// # Safety
    /// The pointer and lifetime requirements are the same as [`FrameworkSlice::as_bytes`], and
    /// the bytes must contain valid UTF-8.
    pub unsafe fn as_str<'a>(self) -> Option<&'a str> {
        let slice = FrameworkSlice {
            data: self.data,
            length: self.length,
        };
        // SAFETY: the caller upholds the pointer and lifetime contract documented above.
        let bytes = unsafe { slice.as_bytes()? };
        core::str::from_utf8(bytes).ok()
    }
}

/// An owned byte buffer whose allocation must be released by the framework destructor.
///
/// The pointer, length, and capacity are a C layout contract, not a Rust `Vec` ABI. Do not copy
/// this value by value or mutate its fields from C; use `framework_owned_buffer_destroy` once.
#[repr(C)]
pub struct FrameworkOwnedBuffer {
    data: *mut u8,
    length: u64,
    capacity: u64,
}

impl FrameworkOwnedBuffer {
    /// Transfers a vector allocation into an ABI-owned buffer.
    pub fn try_from_vec(bytes: Vec<u8>) -> Result<Self, Vec<u8>> {
        let length = match u64::try_from(bytes.len()) {
            Ok(value) => value,
            Err(_) => return Err(bytes),
        };
        let capacity = match u64::try_from(bytes.capacity()) {
            Ok(value) => value,
            Err(_) => return Err(bytes),
        };
        if capacity == 0 {
            return Ok(Self {
                data: ptr::null_mut(),
                length: 0,
                capacity: 0,
            });
        }
        let data = bytes.as_ptr().cast_mut();
        core::mem::forget(bytes);
        Ok(Self {
            data,
            length,
            capacity,
        })
    }

    /// Returns a shared view of this valid framework-owned buffer.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        let length = usize::try_from(self.length).ok()?;
        if length == 0 {
            return Some(&[]);
        }
        if self.data.is_null() {
            return None;
        }
        // SAFETY: only the constructor can create a nonempty buffer in safe Rust; C mutation
        // violates the documented ownership contract and requires an unsafe FFI call by the user.
        Some(unsafe { core::slice::from_raw_parts(self.data, length) })
    }

    /// Returns the fixed-width byte length.
    pub const fn length(&self) -> u64 {
        self.length
    }

    /// Returns the fixed-width allocation capacity.
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns the mutable data pointer without dereferencing it.
    pub const fn data(&self) -> *mut u8 {
        self.data
    }

    /// Transfers this allocation back into a Rust vector.
    pub fn into_vec(mut self) -> Vec<u8> {
        let data = core::mem::replace(&mut self.data, ptr::null_mut());
        let length = core::mem::replace(&mut self.length, 0);
        let capacity = core::mem::replace(&mut self.capacity, 0);
        if capacity == 0 {
            return Vec::new();
        }
        let (Ok(length), Ok(capacity)) = (usize::try_from(length), usize::try_from(capacity))
        else {
            return Vec::new();
        };
        if length > capacity || data.is_null() {
            return Vec::new();
        }
        // SAFETY: the buffer was created from a Vec and its allocation fields are private to Rust.
        unsafe { Vec::from_raw_parts(data, length, capacity) }
    }

    fn release(&mut self) {
        let data = core::mem::replace(&mut self.data, ptr::null_mut());
        let length = core::mem::replace(&mut self.length, 0);
        let capacity = core::mem::replace(&mut self.capacity, 0);
        if capacity == 0 {
            return;
        }
        let (Ok(length), Ok(capacity)) = (usize::try_from(length), usize::try_from(capacity))
        else {
            return;
        };
        if length > capacity || data.is_null() {
            return;
        }
        // SAFETY: valid buffers originate from Vec and the unsafe C destructor contract forbids
        // callers from changing the pointer, length, or capacity before release.
        unsafe {
            drop(Vec::from_raw_parts(data, length, capacity));
        }
    }
}

impl Default for FrameworkOwnedBuffer {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            length: 0,
            capacity: 0,
        }
    }
}

impl Drop for FrameworkOwnedBuffer {
    fn drop(&mut self) {
        self.release();
    }
}

/// Destroys an owned buffer and resets its fields to the empty state.
///
/// # Safety
/// A non-null argument must point to a live `FrameworkOwnedBuffer` created by this framework.
/// The fields must be unchanged, and no copied descriptor may also be destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_owned_buffer_destroy(buffer: *mut FrameworkOwnedBuffer) {
    if let Some(buffer) = unsafe { buffer.as_mut() } {
        buffer.release();
    }
}

/// A fixed-width C handle for an operation; zero means no operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct FrameworkOperationHandle(u64);

impl FrameworkOperationHandle {
    /// Creates a nonzero handle from its C representation.
    pub const fn new(raw: u64) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    /// Creates a C handle from a portable operation ID.
    pub const fn from_operation_id(id: OperationId) -> Self {
        Self(id.get())
    }

    /// Converts a nonzero C handle to a portable operation ID.
    pub const fn operation_id(self) -> OperationId {
        match OperationId::new(self.0) {
            Some(value) => value,
            None => panic!("FrameworkOperationHandle invariant violated"),
        }
    }

    /// Returns the fixed-width C representation.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A fixed-width C handle for optional extended error detail; zero means no detail.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct FrameworkErrorHandle(u64);

impl FrameworkErrorHandle {
    /// Creates a handle from its C representation; zero is reserved.
    pub const fn new(raw: u64) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    /// Returns the fixed-width C representation.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// The C callback signature for one operation completion.
pub type FrameworkCompletionCallback = Option<
    unsafe extern "C" fn(
        context: *mut c_void,
        operation: FrameworkOperationHandle,
        status: FrameworkStatus,
        result: FrameworkSlice,
    ),
>;

/// A one-shot callback adapter with explicit context and operation identity.
pub struct FrameworkCompletion {
    callback: FrameworkCompletionCallback,
    context: *mut c_void,
    operation: FrameworkOperationHandle,
}

impl FrameworkCompletion {
    /// Creates a one-shot completion adapter.
    ///
    /// # Safety
    /// If `callback` is present, `context` must remain valid for callback use until `invoke` or
    /// the adapter is dropped. The callback must obey the C ABI and must not unwind across it.
    pub const unsafe fn new(
        callback: FrameworkCompletionCallback,
        context: *mut c_void,
        operation: FrameworkOperationHandle,
    ) -> Self {
        Self {
            callback,
            context,
            operation,
        }
    }

    /// Invokes the callback once, if present. The result view is valid only for the call duration.
    pub fn invoke(self, status: FrameworkStatus, result: FrameworkSlice) {
        if let Some(callback) = self.callback {
            // SAFETY: construction is unsafe and requires a live context and a non-unwinding C callback.
            unsafe {
                callback(self.context, self.operation, status, result);
            }
        }
    }
}

/// The V1 extensible options header for future C ABI functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct FrameworkOptionsV1 {
    /// The byte size of the caller-provided struct.
    pub struct_size: u32,
    /// The ABI major version used by the caller.
    pub abi_version: u32,
    /// Version-specific option flags; unknown bits must be ignored.
    pub flags: u32,
    /// Reserved for future use and required to be zero in V1.
    pub reserved: u32,
}

impl FrameworkOptionsV1 {
    /// Creates a zero-reserved V1 options header with the supplied flags.
    pub const fn new(flags: u32) -> Self {
        Self {
            struct_size: size_of::<Self>() as u32,
            abi_version: ABI_VERSION_MAJOR,
            flags,
            reserved: 0,
        }
    }
}

/// Runs a callback under `std` panic containment when the optional `std` feature is enabled.
#[cfg(feature = "std")]
pub fn catch_unwind_status<F>(callback: F) -> FrameworkStatus
where
    F: FnOnce() -> FrameworkStatus + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(callback) {
        Ok(status) => status,
        Err(_) => FrameworkStatus::PANIC,
    }
}

#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use core::mem::{align_of, offset_of, size_of};
    use core::sync::atomic::{AtomicUsize, Ordering};

    fn round_up(value: usize, alignment: usize) -> usize {
        value.div_ceil(alignment) * alignment
    }

    #[test]
    fn stable_value_layouts_use_fixed_width_fields() {
        assert_eq!(
            (size_of::<FrameworkStatus>(), align_of::<FrameworkStatus>()),
            (4, 4)
        );
        assert_eq!(
            (
                size_of::<FrameworkOperationHandle>(),
                align_of::<FrameworkOperationHandle>()
            ),
            (8, align_of::<u64>())
        );
        assert_eq!(
            (
                size_of::<FrameworkErrorHandle>(),
                align_of::<FrameworkErrorHandle>()
            ),
            (8, align_of::<u64>())
        );
        assert_eq!(
            (
                size_of::<FrameworkOptionsV1>(),
                align_of::<FrameworkOptionsV1>()
            ),
            (16, 4)
        );
        let pointer = size_of::<*const u8>();
        let alignment = align_of::<u64>().max(align_of::<*const u8>());
        assert_eq!(offset_of!(FrameworkSlice, data), 0);
        assert_eq!(
            offset_of!(FrameworkSlice, length),
            round_up(pointer, align_of::<u64>())
        );
        assert_eq!(
            size_of::<FrameworkSlice>(),
            round_up(pointer + 8, alignment)
        );
        assert_eq!(size_of::<FrameworkStr>(), size_of::<FrameworkSlice>());
        assert_eq!(offset_of!(FrameworkOwnedBuffer, data), 0);
        assert_eq!(
            offset_of!(FrameworkOwnedBuffer, length),
            round_up(pointer, align_of::<u64>())
        );
        assert_eq!(
            offset_of!(FrameworkOwnedBuffer, capacity),
            round_up(pointer, align_of::<u64>()) + 8
        );
        assert_eq!(
            size_of::<FrameworkOwnedBuffer>(),
            round_up(pointer + 16, alignment)
        );
    }

    #[test]
    fn handles_reserve_zero_and_options_set_the_v1_header() {
        assert_eq!(FrameworkOperationHandle::new(0), None);
        assert_eq!(FrameworkErrorHandle::new(0), None);

        let options = FrameworkOptionsV1::new(0x25);
        assert_eq!(options.struct_size, 16);
        assert_eq!(options.abi_version, ABI_VERSION_MAJOR);
        assert_eq!(options.flags, 0x25);
        assert_eq!(options.reserved, 0);
    }

    #[test]
    fn pointer_length_views_preserve_utf8_and_explicit_lengths() {
        let text = "café";
        let value = FrameworkStr::from_utf8(text).unwrap();
        assert_eq!(value.length(), 5);
        // SAFETY: the source string remains alive and unchanged for this assertion.
        assert_eq!(unsafe { value.as_str() }, Some(text));
        let bytes = [0, 1, 2, 255];
        let value = FrameworkSlice::from_bytes(&bytes).unwrap();
        // SAFETY: the source array remains alive and unchanged for this assertion.
        assert_eq!(unsafe { value.as_bytes() }, Some(&bytes[..]));
    }

    #[test]
    fn owned_buffer_round_trips_and_c_destructor_resets_the_descriptor() {
        let empty = FrameworkOwnedBuffer::try_from_vec(vec![]).unwrap();
        assert!(empty.data().is_null());
        assert_eq!(empty.length(), 0);
        assert_eq!(empty.capacity(), 0);

        let buffer = FrameworkOwnedBuffer::try_from_vec(vec![4, 5, 6]).unwrap();
        assert_eq!(buffer.as_bytes(), Some(&[4, 5, 6][..]));
        let bytes = buffer.into_vec();
        assert_eq!(bytes, vec![4, 5, 6]);

        let mut buffer = FrameworkOwnedBuffer::try_from_vec(vec![7, 8]).unwrap();
        // SAFETY: this descriptor is live, framework-created, and has not been copied or mutated.
        unsafe {
            framework_owned_buffer_destroy(&mut buffer);
        }
        assert_eq!(buffer.length(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert!(buffer.data().is_null());
        drop(buffer);

        // SAFETY: the destructor accepts a null buffer pointer.
        unsafe {
            framework_owned_buffer_destroy(core::ptr::null_mut());
        }
    }

    #[test]
    fn portable_errors_map_to_stable_c_statuses() {
        for (kind, expected) in [
            (ErrorKind::Unknown, FrameworkStatus::INTERNAL_ERROR),
            (ErrorKind::InvalidInput, FrameworkStatus::INVALID_ARGUMENT),
            (ErrorKind::Unsupported, FrameworkStatus::UNSUPPORTED),
            (ErrorKind::Unavailable, FrameworkStatus::UNAVAILABLE),
            (
                ErrorKind::PermissionDenied,
                FrameworkStatus::PERMISSION_DENIED,
            ),
            (ErrorKind::Cancelled, FrameworkStatus::CANCELLED),
            (ErrorKind::Timeout, FrameworkStatus::TIMEOUT),
            (ErrorKind::NotFound, FrameworkStatus::NOT_FOUND),
            (ErrorKind::AlreadyExists, FrameworkStatus::ALREADY_EXISTS),
            (
                ErrorKind::ResourceExhausted,
                FrameworkStatus::RESOURCE_EXHAUSTED,
            ),
            (ErrorKind::Platform, FrameworkStatus::PLATFORM_ERROR),
            (ErrorKind::Internal, FrameworkStatus::INTERNAL_ERROR),
        ] {
            assert_eq!(FrameworkStatus::from_error(Error::new(kind)), expected);
        }

        assert_eq!(FrameworkStatus::OK.code(), 0);
        assert_eq!(FrameworkStatus::INVALID_ARGUMENT.code(), 1);
        assert_eq!(FrameworkStatus::UNSUPPORTED.code(), 2);
        assert_eq!(FrameworkStatus::UNAVAILABLE.code(), 3);
        assert_eq!(FrameworkStatus::PERMISSION_DENIED.code(), 4);
        assert_eq!(FrameworkStatus::CANCELLED.code(), 5);
        assert_eq!(FrameworkStatus::TIMEOUT.code(), 6);
        assert_eq!(FrameworkStatus::NOT_FOUND.code(), 7);
        assert_eq!(FrameworkStatus::ALREADY_EXISTS.code(), 8);
        assert_eq!(FrameworkStatus::RESOURCE_EXHAUSTED.code(), 9);
        assert_eq!(FrameworkStatus::PLATFORM_ERROR.code(), 10);
        assert_eq!(FrameworkStatus::INTERNAL_ERROR.code(), 11);
        assert_eq!(FrameworkStatus::PANIC.code(), 12);
    }

    unsafe extern "C" fn count_callback(
        context: *mut c_void,
        operation: FrameworkOperationHandle,
        status: FrameworkStatus,
        result: FrameworkSlice,
    ) {
        // SAFETY: the test provides an aligned live AtomicUsize context for this one call.
        let count = unsafe { &*(context.cast::<AtomicUsize>()) };
        assert_eq!(operation.raw(), 1);
        assert_eq!(status, FrameworkStatus::OK);
        assert_eq!(result.length(), 0);
        count.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn one_shot_callback_adapter_keeps_c_shapes_explicit() {
        let count = AtomicUsize::new(0);
        let operation = FrameworkOperationHandle::new(1).unwrap();
        // SAFETY: `count` remains live through invocation and the callback does not unwind.
        let completion = unsafe {
            FrameworkCompletion::new(
                Some(count_callback),
                (&count as *const AtomicUsize).cast_mut().cast(),
                operation,
            )
        };
        completion.invoke(
            FrameworkStatus::OK,
            FrameworkSlice::from_bytes(&[]).unwrap(),
        );
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "std")]
    #[test]
    fn optional_std_boundary_maps_a_panic_without_unwinding() {
        let status = catch_unwind_status(|| panic!("test panic"));
        assert_eq!(status, FrameworkStatus::PANIC);
    }
}
