use core::panic::AssertUnwindSafe;
#[cfg(test)]
use core::ptr;
use framework_abi::{
    FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus, FrameworkStr, catch_unwind_status,
};
use framework_core::Error;
#[cfg(any(target_os = "ios", test))]
use framework_core::ErrorKind;
#[cfg(test)]
use framework_core::PlatformErrorCode;
#[cfg(target_os = "ios")]
use framework_secure_storage::SecureStorage;
use framework_secure_storage::{AccessPolicy, ItemId, SecureStorageError, ServiceId};

const DEVICE_UNLOCK_REQUIRED: u32 = 1 << 0;
const DEVICE_BOUND: u32 = 1 << 1;
const POLICY_MASK: u32 = DEVICE_UNLOCK_REQUIRED | DEVICE_BOUND;

fn status_from_error(error: SecureStorageError) -> FrameworkStatus {
    FrameworkStatus::from_error(Error::new(error.kind()))
}

#[cfg(any(target_os = "ios", test))]
fn native_code_from_error(error: SecureStorageError) -> i32 {
    if error.kind() == ErrorKind::Platform {
        error.platform_code().map_or(0, |code| code.get())
    } else {
        0
    }
}

fn valid_span(data: *const u8, length: u64) -> bool {
    match usize::try_from(length) {
        Ok(length) => length <= isize::MAX as usize && (length == 0 || !data.is_null()),
        Err(_) => false,
    }
}

fn policy_from_flags(flags: u32) -> Option<AccessPolicy> {
    if flags & !POLICY_MASK != 0 {
        return None;
    }
    Some(AccessPolicy::new(
        flags & DEVICE_UNLOCK_REQUIRED != 0,
        flags & DEVICE_BOUND != 0,
    ))
}

#[cfg(any(target_os = "ios", test))]
fn read_output(
    result: Option<alloc::vec::Vec<u8>>,
) -> Result<(u8, FrameworkOwnedBuffer), FrameworkStatus> {
    match result {
        None => Ok((0, FrameworkOwnedBuffer::default())),
        Some(bytes) => FrameworkOwnedBuffer::try_from_vec(bytes)
            .map(|buffer| (1, buffer))
            .map_err(|bytes| {
                drop(bytes);
                FrameworkStatus::RESOURCE_EXHAUSTED
            }),
    }
}

unsafe fn initialize_native_code(output: *mut i32) {
    if !output.is_null() {
        // SAFETY: the caller promises a writable optional native-code output when non-null.
        unsafe { output.write(0) };
    }
}

#[cfg(any(target_os = "ios", test))]
unsafe fn write_native_code(output: *mut i32, error: SecureStorageError) {
    if !output.is_null() {
        // SAFETY: the caller promises a writable optional native-code output when non-null.
        unsafe { output.write(native_code_from_error(error)) };
    }
}

unsafe fn identifiers<'a>(
    service: FrameworkStr,
    item: FrameworkStr,
) -> Result<(ServiceId<'a>, ItemId<'a>), FrameworkStatus> {
    if !valid_span(service.data(), service.length()) || !valid_span(item.data(), item.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: the exported functions document the pointer, length, and lifetime preconditions.
    let service = unsafe { service.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)?;
    // SAFETY: the exported functions document the pointer, length, and lifetime preconditions.
    let item = unsafe { item.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)?;
    let service = ServiceId::new(service).map_err(status_from_error)?;
    let item = ItemId::new(item).map_err(status_from_error)?;
    Ok((service, item))
}

unsafe fn secret_bytes<'a>(secret: FrameworkSlice) -> Result<&'a [u8], FrameworkStatus> {
    if !valid_span(secret.data(), secret.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: the exported functions document the pointer, length, and lifetime preconditions.
    unsafe { secret.as_bytes() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

unsafe fn read(
    service: FrameworkStr,
    item: FrameworkStr,
    out_found: *mut u8,
    out_secret: *mut FrameworkOwnedBuffer,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    // SAFETY: the caller promises a writable optional native-code output when non-null.
    unsafe { initialize_native_code(out_native_os_status) };
    // SAFETY: each non-null required output is writable per the caller contract.
    unsafe {
        if !out_found.is_null() {
            out_found.write(0);
        }
        if !out_secret.is_null() {
            out_secret.write(FrameworkOwnedBuffer::default());
        }
    }
    if out_found.is_null() || out_secret.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: the exported function documents valid input pointer and lifetime requirements.
    let (service, item) = match unsafe { identifiers(service, item) } {
        Ok(values) => values,
        Err(status) => return status,
    };

    #[cfg(target_os = "ios")]
    {
        let mut storage = SecureStorage::new(ios_secure_storage::IosSecureStorage::new());
        match storage.read(service, item) {
            Ok(result) => match read_output(result) {
                Ok((found, buffer)) => {
                    // SAFETY: outputs were validated and initialized above.
                    unsafe {
                        out_secret.write(buffer);
                        out_found.write(found);
                    }
                    FrameworkStatus::OK
                }
                Err(status) => status,
            },
            Err(error) => {
                // SAFETY: the caller promises a writable optional native-code output.
                unsafe { write_native_code(out_native_os_status, error) };
                status_from_error(error)
            }
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (service, item);
        FrameworkStatus::UNSUPPORTED
    }
}

/// Reads one matching Keychain item; the unfiltered query searches all app groups.
/// The operation is synchronous and may block.
///
/// # Safety
/// Non-empty input spans must point to valid, readable memory and remain immutable for the full
/// call; the caller must prevent unsynchronized mutation. Required output slots may be null; each
/// non-null output slot, including the optional native-code slot, must point to valid, properly
/// aligned writable memory for the full call. Output memory ranges must be pairwise disjoint and
/// must not overlap non-empty input spans; this function does not check overlap. A missing required
/// output returns `INVALID_ARGUMENT` after all other non-null output slots are initialized. A
/// non-null `out_secret` must not contain a live framework-owned allocation; it is initialized to
/// empty on entry. The function catches Rust panics, but it cannot validate arbitrary invalid C
/// addresses or pointer provenance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_secure_storage_read(
    service: FrameworkStr,
    item: FrameworkStr,
    out_found: *mut u8,
    out_secret: *mut FrameworkOwnedBuffer,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: the C caller must uphold this function's documented pointer contract.
        unsafe { read(service, item, out_found, out_secret, out_native_os_status) }
    }))
}

unsafe fn store(
    service: FrameworkStr,
    item: FrameworkStr,
    secret: FrameworkSlice,
    required_policy_flags: u32,
    out_effective_policy_flags: *mut u32,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    // SAFETY: the caller promises a writable optional native-code output when non-null.
    unsafe { initialize_native_code(out_native_os_status) };
    if out_effective_policy_flags.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: the required output is writable and distinct from the optional native-code output.
    unsafe { out_effective_policy_flags.write(0) };
    // SAFETY: the exported function documents valid input pointer and lifetime requirements.
    let (service, item) = match unsafe { identifiers(service, item) } {
        Ok(values) => values,
        Err(status) => return status,
    };
    // SAFETY: the exported function documents valid input pointer and lifetime requirements.
    let secret = match unsafe { secret_bytes(secret) } {
        Ok(secret) => secret,
        Err(status) => return status,
    };
    let Some(required_policy) = policy_from_flags(required_policy_flags) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    #[cfg(target_os = "ios")]
    {
        let mut storage = SecureStorage::new(ios_secure_storage::IosSecureStorage::new());
        match storage.store(service, item, secret, required_policy) {
            Ok(outcome) => {
                // SAFETY: the required output was validated and initialized above.
                unsafe {
                    out_effective_policy_flags.write(policy_flags(outcome.effective_policy()))
                };
                FrameworkStatus::OK
            }
            Err(error) => {
                // SAFETY: the caller promises a writable optional native-code output.
                unsafe { write_native_code(out_native_os_status, error) };
                status_from_error(error)
            }
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (service, item, secret, required_policy);
        FrameworkStatus::UNSUPPORTED
    }
}

/// Stores opaque bytes using a default-group add or an unfiltered update across app groups, then
/// reports the effective policy. The operation is synchronous and may block.
///
/// # Safety
/// Non-empty input spans must point to valid, readable memory and remain immutable for the full
/// call; the caller must prevent unsynchronized mutation. The required output slot may be null; if
/// it is null, the function returns `INVALID_ARGUMENT` after zeroing a non-null native-code output.
/// Each non-null output slot must point to valid, properly aligned writable memory for the full
/// call. Output memory ranges must be disjoint from one another and must not overlap non-empty
/// input spans; this function does not check overlap. The required and optional outputs
/// are initialized before input validation. The function catches Rust panics, but it cannot
/// validate arbitrary invalid C addresses or pointer provenance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_secure_storage_store(
    service: FrameworkStr,
    item: FrameworkStr,
    secret: FrameworkSlice,
    required_policy_flags: u32,
    out_effective_policy_flags: *mut u32,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: the C caller must uphold this function's documented pointer contract.
        unsafe {
            store(
                service,
                item,
                secret,
                required_policy_flags,
                out_effective_policy_flags,
                out_native_os_status,
            )
        }
    }))
}

unsafe fn remove(
    service: FrameworkStr,
    item: FrameworkStr,
    out_removed: *mut u8,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    // SAFETY: the caller promises a writable optional native-code output when non-null.
    unsafe { initialize_native_code(out_native_os_status) };
    if out_removed.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: the required output is writable and distinct from the optional native-code output.
    unsafe { out_removed.write(0) };
    // SAFETY: the exported function documents valid input pointer and lifetime requirements.
    let (service, item) = match unsafe { identifiers(service, item) } {
        Ok(values) => values,
        Err(status) => return status,
    };

    #[cfg(target_os = "ios")]
    {
        let mut storage = SecureStorage::new(ios_secure_storage::IosSecureStorage::new());
        match storage.remove(service, item) {
            Ok(removed) => {
                // SAFETY: the required output was validated and initialized above.
                unsafe { out_removed.write(u8::from(removed)) };
                FrameworkStatus::OK
            }
            Err(error) => {
                // SAFETY: the caller promises a writable optional native-code output.
                unsafe { write_native_code(out_native_os_status, error) };
                status_from_error(error)
            }
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (service, item);
        FrameworkStatus::UNSUPPORTED
    }
}

/// Removes all matching items across app groups and reports whether any existed.
/// The operation is synchronous and may block.
///
/// # Safety
/// Input spans must point to valid, readable memory and remain immutable for the full call; the
/// caller must prevent unsynchronized mutation. The required output slot may be null; if it is
/// null, the function returns `INVALID_ARGUMENT` after zeroing a non-null native-code output. Each
/// non-null output slot must point to valid, properly aligned writable memory for the full call.
/// Output memory ranges must be disjoint from one another and must not overlap non-empty input
/// spans; this function does not check overlap. The required and optional outputs are
/// initialized before input validation. The function catches Rust panics, but it cannot validate
/// arbitrary invalid C addresses or pointer provenance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_secure_storage_remove(
    service: FrameworkStr,
    item: FrameworkStr,
    out_removed: *mut u8,
    out_native_os_status: *mut i32,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: the C caller must uphold this function's documented pointer contract.
        unsafe { remove(service, item, out_removed, out_native_os_status) }
    }))
}

#[cfg(any(target_os = "ios", test))]
fn policy_flags(policy: AccessPolicy) -> u32 {
    (if policy.device_unlock_required() {
        DEVICE_UNLOCK_REQUIRED
    } else {
        0
    }) | (if policy.device_bound() {
        DEVICE_BOUND
    } else {
        0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_bits_are_fixed_and_unknown_bits_are_rejected() {
        for flags in 0..=POLICY_MASK {
            let policy = policy_from_flags(flags).unwrap();
            assert_eq!(policy_flags(policy), flags);
        }
        assert_eq!(policy_from_flags(1 << 2), None);
        assert_eq!(policy_from_flags(u32::MAX), None);
    }

    #[test]
    fn error_status_and_native_code_are_separate() {
        let invalid = SecureStorageError::InvalidItemId;
        assert_eq!(
            status_from_error(invalid),
            FrameworkStatus::INVALID_ARGUMENT
        );
        assert_eq!(native_code_from_error(invalid), 0);
        let unsupported = SecureStorageError::UnsupportedPolicy;
        assert_eq!(status_from_error(unsupported), FrameworkStatus::UNSUPPORTED);
        assert_eq!(native_code_from_error(unsupported), 0);
        let coded_non_platform = SecureStorageError::Backend(
            Error::new(ErrorKind::Internal)
                .with_platform_code(PlatformErrorCode::new(-50).unwrap()),
        );
        assert_eq!(
            status_from_error(coded_non_platform),
            FrameworkStatus::INTERNAL_ERROR
        );
        assert_eq!(native_code_from_error(coded_non_platform), 0);
        let mut output = -99;
        // SAFETY: `output` is a writable native-code slot for this unit test.
        unsafe { write_native_code(&mut output, coded_non_platform) };
        assert_eq!(output, 0);
        let platform = SecureStorageError::Backend(
            Error::new(ErrorKind::Platform)
                .with_platform_code(PlatformErrorCode::new(-25300).unwrap()),
        );
        assert_eq!(status_from_error(platform), FrameworkStatus::PLATFORM_ERROR);
        assert_eq!(native_code_from_error(platform), -25300);
        // SAFETY: `output` is a writable native-code slot for this unit test.
        unsafe { write_native_code(&mut output, platform) };
        assert_eq!(output, -25300);
    }

    #[test]
    fn every_known_error_kind_maps_to_its_stable_status() {
        let cases = [
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
        ];

        for (kind, expected) in cases {
            assert_eq!(
                status_from_error(SecureStorageError::Backend(Error::new(kind))),
                expected
            );
        }
    }

    #[test]
    fn absent_and_empty_secret_results_remain_distinguishable() {
        let (found, absent) = read_output(None).unwrap();
        assert_eq!(found, 0);
        assert_eq!(absent.length(), 0);
        assert!(absent.data().is_null());
        let (found, empty) = read_output(Some(alloc::vec::Vec::new())).unwrap();
        assert_eq!(found, 1);
        assert_eq!(empty.length(), 0);
        assert!(empty.data().is_null());
    }

    fn raw_str(data: *const u8, length: u64) -> FrameworkStr {
        // SAFETY: FrameworkStr is repr(C) with the C fields `(const uint8_t *, uint64_t)`.
        unsafe { core::mem::transmute((data, length)) }
    }

    #[test]
    fn identifiers_reject_empty_nul_invalid_utf8_and_malformed_spans() {
        let item = FrameworkStr::from_utf8("item").unwrap();
        let empty = FrameworkStr::from_utf8("").unwrap();
        let nul = FrameworkStr::from_utf8("a\0b").unwrap();
        let invalid_utf8_bytes = [0xff];
        // SAFETY: each span is readable for its stated length; one intentionally is invalid UTF-8.
        assert_eq!(
            unsafe { identifiers(empty, item) },
            Err(FrameworkStatus::INVALID_ARGUMENT)
        );
        assert_eq!(
            unsafe { identifiers(nul, item) },
            Err(FrameworkStatus::INVALID_ARGUMENT)
        );
        assert_eq!(
            unsafe { identifiers(raw_str(invalid_utf8_bytes.as_ptr(), 1), item) },
            Err(FrameworkStatus::INVALID_ARGUMENT)
        );
        assert_eq!(
            unsafe { identifiers(raw_str(ptr::null(), 1), item) },
            Err(FrameworkStatus::INVALID_ARGUMENT)
        );
        assert_eq!(
            unsafe { identifiers(raw_str(item.data(), u64::MAX), item) },
            Err(FrameworkStatus::INVALID_ARGUMENT)
        );
    }

    #[test]
    fn panic_boundary_returns_the_stable_panic_status() {
        let status = catch_unwind_status(AssertUnwindSafe(|| panic!("boundary test")));
        assert_eq!(status, FrameworkStatus::PANIC);
    }

    #[cfg(not(target_os = "ios"))]
    #[test]
    fn host_stubs_validate_then_return_unsupported_with_initialized_outputs() {
        let service = FrameworkStr::from_utf8("service").unwrap();
        let item = FrameworkStr::from_utf8("item").unwrap();
        let secret = FrameworkSlice::from_bytes(&[]).unwrap();
        let mut found = 7;
        let mut output = FrameworkOwnedBuffer::default();
        let mut native = -99;
        // SAFETY: all spans and output slots are valid for each synchronous call.
        let status = unsafe {
            framework_ios_secure_storage_read(service, item, &mut found, &mut output, &mut native)
        };
        assert_eq!(status, FrameworkStatus::UNSUPPORTED);
        assert_eq!(found, 0);
        assert_eq!(output.length(), 0);
        assert_eq!(native, 0);

        let mut effective = 99;
        native = -99;
        // SAFETY: all spans and output slots are valid for this synchronous call.
        let status = unsafe {
            framework_ios_secure_storage_store(
                service,
                item,
                secret,
                0,
                &mut effective,
                &mut native,
            )
        };
        assert_eq!(status, FrameworkStatus::UNSUPPORTED);
        assert_eq!(effective, 0);
        assert_eq!(native, 0);

        let mut removed = 7;
        native = -99;
        // SAFETY: all spans and output slots are valid for this synchronous call.
        let status = unsafe {
            framework_ios_secure_storage_remove(service, item, &mut removed, &mut native)
        };
        assert_eq!(status, FrameworkStatus::UNSUPPORTED);
        assert_eq!(removed, 0);
        assert_eq!(native, 0);

        effective = 99;
        native = -99;
        // SAFETY: all spans and output slots are valid for this synchronous call.
        let status = unsafe {
            framework_ios_secure_storage_store(
                service,
                item,
                secret,
                1 << 2,
                &mut effective,
                &mut native,
            )
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(effective, 0);
        assert_eq!(native, 0);
    }

    #[cfg(not(target_os = "ios"))]
    #[test]
    fn host_stubs_initialize_available_outputs_before_validation_errors() {
        let service = FrameworkStr::from_utf8("service").unwrap();
        let item = FrameworkStr::from_utf8("item").unwrap();
        let empty = FrameworkStr::from_utf8("").unwrap();
        let secret = FrameworkSlice::from_bytes(&[]).unwrap();
        let mut native = -99;
        let mut output = FrameworkOwnedBuffer::default();

        // SAFETY: each non-null output points to a writable local slot.
        let status = unsafe {
            framework_ios_secure_storage_read(
                service,
                item,
                ptr::null_mut(),
                &mut output,
                &mut native,
            )
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(output.length(), 0);
        assert!(output.data().is_null());
        assert_eq!(native, 0);

        let mut found = 7;
        native = -99;
        // SAFETY: each non-null output points to a writable local slot.
        let status = unsafe {
            framework_ios_secure_storage_read(
                service,
                item,
                &mut found,
                ptr::null_mut(),
                &mut native,
            )
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(found, 0);
        assert_eq!(native, 0);

        native = -99;
        // SAFETY: the optional native-code output points to a writable local slot.
        let status = unsafe {
            framework_ios_secure_storage_store(
                service,
                item,
                secret,
                0,
                ptr::null_mut(),
                &mut native,
            )
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(native, 0);

        let mut effective = 99;
        native = -99;
        // SAFETY: each non-null output points to a writable local slot; empty IDs are rejected.
        let status = unsafe {
            framework_ios_secure_storage_store(empty, item, secret, 0, &mut effective, &mut native)
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(effective, 0);
        assert_eq!(native, 0);

        native = -99;
        // SAFETY: the optional native-code output points to a writable local slot.
        let status = unsafe {
            framework_ios_secure_storage_remove(service, item, ptr::null_mut(), &mut native)
        };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(native, 0);

        let mut removed = 7;
        native = -99;
        // SAFETY: each non-null output points to a writable local slot; empty IDs are rejected.
        let status =
            unsafe { framework_ios_secure_storage_remove(empty, item, &mut removed, &mut native) };
        assert_eq!(status, FrameworkStatus::INVALID_ARGUMENT);
        assert_eq!(removed, 0);
        assert_eq!(native, 0);
    }
}
