#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Static C ABI entry points over the framework's Rust-native foundation types."]

pub use framework_abi::{
    FrameworkCompletionCallback, FrameworkErrorHandle, FrameworkOperationHandle,
    FrameworkOptionsV1, FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, FrameworkStr,
    framework_owned_buffer_destroy,
};

#[cfg(any(feature = "secure-storage", feature = "notification-responses"))]
extern crate alloc;

#[cfg(feature = "notification-responses")]
mod notification_responses;

#[cfg(feature = "secure-storage")]
mod secure_storage;

#[cfg(feature = "notification-responses")]
pub use notification_responses::{
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION,
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT, FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS,
    FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT, FrameworkNotificationResponse,
    FrameworkNotificationResponseKind, FrameworkNotificationResponseViewV1,
    framework_notification_response_create, framework_notification_response_destroy,
    framework_notification_response_get_view,
};

#[cfg(feature = "secure-storage")]
pub use secure_storage::{
    framework_ios_secure_storage_read, framework_ios_secure_storage_remove,
    framework_ios_secure_storage_store,
};

use framework_abi::{ABI_VERSION_MAJOR, ABI_VERSION_MINOR};

/// Returns the ABI version as major in the high 32 bits and minor in the low 32 bits.
#[unsafe(no_mangle)]
pub extern "C" fn framework_abi_version() -> u64 {
    ((ABI_VERSION_MAJOR as u64) << 32) | ABI_VERSION_MINOR as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};

    fn round_up(value: usize, alignment: usize) -> usize {
        value.div_ceil(alignment) * alignment
    }

    #[test]
    fn version_uses_fixed_width_major_minor_encoding() {
        let version = framework_abi_version();
        assert_eq!((version >> 32) as u32, ABI_VERSION_MAJOR);
        assert_eq!(version as u32, ABI_VERSION_MINOR);
    }

    #[test]
    fn exported_header_layout_matches_framework_abi_types() {
        assert_eq!(
            (size_of::<FrameworkStatus>(), align_of::<FrameworkStatus>()),
            (4, 4)
        );
        assert_eq!(size_of::<FrameworkOperationHandle>(), 8);
        assert_eq!(size_of::<FrameworkErrorHandle>(), 8);
        type CFunction = unsafe extern "C" fn(
            *mut core::ffi::c_void,
            FrameworkOperationHandle,
            FrameworkStatus,
            FrameworkSlice,
        );
        assert_eq!(
            size_of::<FrameworkCompletionCallback>(),
            size_of::<Option<CFunction>>()
        );
        assert_eq!(
            align_of::<FrameworkCompletionCallback>(),
            align_of::<Option<CFunction>>()
        );
        assert_eq!(
            (
                size_of::<FrameworkOptionsV1>(),
                align_of::<FrameworkOptionsV1>()
            ),
            (16, 4)
        );
        assert_eq!(offset_of!(FrameworkOptionsV1, struct_size), 0);
        assert_eq!(offset_of!(FrameworkOptionsV1, abi_version), 4);
        assert_eq!(offset_of!(FrameworkOptionsV1, flags), 8);
        assert_eq!(offset_of!(FrameworkOptionsV1, reserved), 12);
        let pointer = size_of::<*const u8>();
        let u64_align = align_of::<u64>();
        let aggregate_align = u64_align.max(align_of::<*const u8>());
        let length_offset = round_up(pointer, u64_align);
        assert_eq!(
            size_of::<FrameworkSlice>(),
            round_up(length_offset + 8, aggregate_align)
        );
        assert_eq!(align_of::<FrameworkSlice>(), aggregate_align);
        assert_eq!(size_of::<FrameworkStr>(), size_of::<FrameworkSlice>());
        assert_eq!(align_of::<FrameworkStr>(), align_of::<FrameworkSlice>());
        assert_eq!(
            size_of::<FrameworkOwnedBuffer>(),
            round_up(length_offset + 16, aggregate_align)
        );
        assert_eq!(align_of::<FrameworkOwnedBuffer>(), aggregate_align);
    }
}
