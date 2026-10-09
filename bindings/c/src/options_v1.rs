use framework_abi::{ABI_VERSION_MAJOR, FrameworkOptionsV1, FrameworkStatus};

/// Validates the V1 options prefix while ignoring unknown flags and trailing fields.
///
/// A record must declare at least the 16-byte V1 prefix, use the current ABI major, and keep its
/// reserved field zero. Larger records with the same major are accepted without reading beyond
/// the V1 prefix.
///
/// # Safety
/// `options` may be null. Otherwise it must point to valid, properly aligned, fully initialized,
/// readable `FrameworkOptionsV1` storage for the full synchronous call. The caller must prevent
/// unsynchronized mutation of the V1 prefix. This function does not retain the pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_options_v1_validate(
    options: *const FrameworkOptionsV1,
) -> FrameworkStatus {
    if options.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises valid, aligned FrameworkOptionsV1 storage for this call.
    let options = unsafe { &*options };
    if options.struct_size < core::mem::size_of::<FrameworkOptionsV1>() as u32 {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    if options.abi_version != ABI_VERSION_MAJOR {
        return FrameworkStatus::UNSUPPORTED;
    }
    if options.reserved != 0 {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    FrameworkStatus::OK
}
