use core::mem::size_of;
use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkOwnedBuffer, FrameworkStatus, FrameworkStr, catch_unwind_status};
#[cfg(target_os = "ios")]
use ios_extension_support::ExtensionMetadataError;

/// Fixed-width F30 result code for a caller-selected extension metadata read.
pub type FrameworkIosExtensionMetadataError = u32;

/// The metadata read succeeded and the identifier buffer contains UTF-8.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE: FrameworkIosExtensionMetadataError = 0;
/// The path is not absolute, contains NUL, or does not name a nonempty .appex directory.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH:
    FrameworkIosExtensionMetadataError = 1;
/// Foundation could not create a bundle for the supplied URL.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE:
    FrameworkIosExtensionMetadataError = 2;
/// Foundation did not provide the bundle information dictionary.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE:
    FrameworkIosExtensionMetadataError = 3;
/// The information dictionary has no NSExtension value.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY:
    FrameworkIosExtensionMetadataError = 4;
/// The NSExtension value is not a dictionary.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE:
    FrameworkIosExtensionMetadataError = 5;
/// The NSExtension dictionary has no NSExtensionPointIdentifier value.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER:
    FrameworkIosExtensionMetadataError = 6;
/// The point identifier value is not a string.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE:
    FrameworkIosExtensionMetadataError = 7;
/// The point identifier string is empty.
pub const FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER:
    FrameworkIosExtensionMetadataError = 8;

fn input_length(path: FrameworkStr) -> Option<usize> {
    let length = usize::try_from(path.length()).ok()?;
    if length > isize::MAX as usize || (length != 0 && path.data().is_null()) {
        return None;
    }
    Some(length)
}

fn ranges_overlap(
    first: *const u8,
    first_length: usize,
    second: *const u8,
    second_length: usize,
) -> Option<bool> {
    if first_length == 0 || second_length == 0 {
        return Some(false);
    }
    let first_start = first as usize;
    let first_end = first_start.checked_add(first_length)?;
    let second_start = second as usize;
    let second_end = second_start.checked_add(second_length)?;
    Some(first_start < second_end && second_start < first_end)
}

#[cfg(target_os = "ios")]
fn metadata_error_code(error: ExtensionMetadataError) -> FrameworkIosExtensionMetadataError {
    match error {
        ExtensionMetadataError::InvalidBundlePath => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH
        }
        ExtensionMetadataError::BundleUnavailable => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE
        }
        ExtensionMetadataError::InfoDictionaryUnavailable => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE
        }
        ExtensionMetadataError::MissingExtensionDictionary => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY
        }
        ExtensionMetadataError::InvalidExtensionDictionaryType => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE
        }
        ExtensionMetadataError::MissingExtensionPointIdentifier => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER
        }
        ExtensionMetadataError::InvalidExtensionPointIdentifierType => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE
        }
        ExtensionMetadataError::EmptyExtensionPointIdentifier => {
            FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER
        }
    }
}

/// Reads only the extension-point identifier from one caller-supplied iOS .appex path.
///
/// FRAMEWORK_STATUS_OK means the read completed. The error output is NONE with an owned UTF-8
/// identifier buffer on success, or one of the eight B77 metadata error codes with an empty
/// buffer. The API does not load extension code or establish installation, approval, entitlement,
/// launch, or host compatibility.
///
/// # Safety
/// A nonempty bundle_path must point to valid, immutable UTF-8 bytes readable for the call.
/// A null data pointer is valid only with zero length. Both output pointers must be non-null,
/// properly aligned, writable, and disjoint from each other and the input bytes. The owned-buffer
/// output must not hold a live framework allocation on entry. The caller must prevent unsynchronized
/// concurrent access to all input and output memory. A returned buffer must be destroyed once,
/// unchanged, through framework_owned_buffer_destroy. No pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_extension_support_read_extension_point_identifier(
    bundle_path: FrameworkStr,
    out_error: *mut FrameworkIosExtensionMetadataError,
    out_identifier: *mut FrameworkOwnedBuffer,
) -> FrameworkStatus {
    if out_error.is_null() || out_identifier.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Some(path_length) = input_length(bundle_path) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    let error_overlaps_input = ranges_overlap(
        bundle_path.data(),
        path_length,
        out_error.cast(),
        size_of::<FrameworkIosExtensionMetadataError>(),
    );
    let identifier_overlaps_input = ranges_overlap(
        bundle_path.data(),
        path_length,
        out_identifier.cast(),
        size_of::<FrameworkOwnedBuffer>(),
    );
    let outputs_overlap = ranges_overlap(
        out_error.cast(),
        size_of::<FrameworkIosExtensionMetadataError>(),
        out_identifier.cast(),
        size_of::<FrameworkOwnedBuffer>(),
    );
    if error_overlaps_input != Some(false)
        || identifier_overlaps_input != Some(false)
        || outputs_overlap != Some(false)
    {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    // SAFETY: The caller promises valid, aligned, disjoint writable output slots and an empty
    // owned-buffer descriptor on entry.
    unsafe {
        out_error.write(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE);
        out_identifier.write(FrameworkOwnedBuffer::default());
    }
    // SAFETY: The caller promises the input span is readable and immutable for this call; length
    // and nullness have been checked, and null with zero length is a valid empty string.
    let Some(bundle_path) = (unsafe { bundle_path.as_str() }) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            match ios_extension_support::read_extension_point_identifier(bundle_path) {
                Ok(identifier) => {
                    let bytes = identifier.into_bytes();
                    match FrameworkOwnedBuffer::try_from_vec(bytes) {
                        Ok(identifier) => {
                            // SAFETY: The caller supplied disjoint writable output slots above.
                            unsafe { out_identifier.write(identifier) };
                            FrameworkStatus::OK
                        }
                        Err(bytes) => {
                            drop(bytes);
                            FrameworkStatus::RESOURCE_EXHAUSTED
                        }
                    }
                }
                Err(error) => {
                    let error = metadata_error_code(error);
                    // SAFETY: The caller supplied a writable error slot above.
                    unsafe { out_error.write(error) };
                    FrameworkStatus::OK
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = bundle_path;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
