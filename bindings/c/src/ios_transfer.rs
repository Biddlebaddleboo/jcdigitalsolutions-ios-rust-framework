use alloc::boxed::Box;
#[cfg(target_os = "ios")]
use alloc::vec::Vec;
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use core::mem::align_of;
use core::mem::{size_of, zeroed};
use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkSlice, FrameworkStatus, FrameworkStr, catch_unwind_status};

/// Fixed-width transfer-availability tag.
pub type FrameworkTransferAvailability = u32;
/// Availability is unknown to this ABI version.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN: FrameworkTransferAvailability = 0;
/// Transfers are available.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_AVAILABLE: FrameworkTransferAvailability = 1;
/// Transfers are unsupported.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED: FrameworkTransferAvailability = 2;
/// User permission is required.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_PERMISSION: FrameworkTransferAvailability = 3;
/// A platform entitlement is required.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_ENTITLEMENT: FrameworkTransferAvailability = 4;
/// Transfers are temporarily unavailable.
pub const FRAMEWORK_TRANSFER_AVAILABILITY_TEMPORARILY_UNAVAILABLE: FrameworkTransferAvailability =
    5;

/// Fixed-width application-directory tag.
pub type FrameworkTransferDirectory = u32;
/// The app Documents directory.
pub const FRAMEWORK_TRANSFER_DIRECTORY_DOCUMENTS: FrameworkTransferDirectory = 0;
/// The app Caches directory.
pub const FRAMEWORK_TRANSFER_DIRECTORY_CACHES: FrameworkTransferDirectory = 1;
/// The app Temporary directory.
pub const FRAMEWORK_TRANSFER_DIRECTORY_TEMPORARY: FrameworkTransferDirectory = 2;
/// The app Application Support directory.
pub const FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT: FrameworkTransferDirectory = 3;

/// Fixed-width transfer-state tag.
pub type FrameworkTransferState = u32;
/// The task is queued.
pub const FRAMEWORK_TRANSFER_STATE_QUEUED: FrameworkTransferState = 0;
/// The task is active.
pub const FRAMEWORK_TRANSFER_STATE_ACTIVE: FrameworkTransferState = 1;
/// The destination file is committed.
pub const FRAMEWORK_TRANSFER_STATE_SUCCEEDED: FrameworkTransferState = 2;
/// The task failed.
pub const FRAMEWORK_TRANSFER_STATE_FAILED: FrameworkTransferState = 3;
/// The task was cancelled.
pub const FRAMEWORK_TRANSFER_STATE_CANCELLED: FrameworkTransferState = 4;

/// Fixed-width portable error-kind tag.
pub type FrameworkTransferErrorKind = u32;
/// The cause is unknown to this ABI version.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN: FrameworkTransferErrorKind = 0;
/// An input value is invalid.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_INVALID_INPUT: FrameworkTransferErrorKind = 1;
/// The target does not support the operation.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_UNSUPPORTED: FrameworkTransferErrorKind = 2;
/// The operation is unavailable.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_UNAVAILABLE: FrameworkTransferErrorKind = 3;
/// Permission was denied.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_PERMISSION_DENIED: FrameworkTransferErrorKind = 4;
/// The operation was cancelled.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_CANCELLED: FrameworkTransferErrorKind = 5;
/// The operation timed out.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_TIMEOUT: FrameworkTransferErrorKind = 6;
/// The task or resource was not found.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_NOT_FOUND: FrameworkTransferErrorKind = 7;
/// The task ID already exists.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_ALREADY_EXISTS: FrameworkTransferErrorKind = 8;
/// A bounded resource could not be obtained.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_RESOURCE_EXHAUSTED: FrameworkTransferErrorKind = 9;
/// A platform error has no more specific category.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_PLATFORM: FrameworkTransferErrorKind = 10;
/// An internal operation failed.
pub const FRAMEWORK_TRANSFER_ERROR_KIND_INTERNAL: FrameworkTransferErrorKind = 11;

/// A 128-bit app-assigned transfer ID split into high and low words.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkTransferIdV1 {
    /// Bits 127 through 64.
    pub high: u64,
    /// Bits 63 through 0.
    pub low: u64,
}

/// One borrowed request header.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkTransferHeaderV1 {
    /// ASCII HTTP token name.
    pub name: FrameworkStr,
    /// Opaque request field-value bytes.
    pub value: FrameworkSlice,
}

/// A versioned GET download request supplied by C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkTransferRequestV1 {
    /// Exact byte size of this record.
    pub struct_size: u32,
    /// ABI major version from `framework_abi_version`.
    pub abi_version: u32,
    /// App-assigned nonzero transfer ID.
    pub id: FrameworkTransferIdV1,
    /// HTTP or HTTPS URL.
    pub url: FrameworkStr,
    /// Borrowed ordered header array; null only when `header_count` is zero.
    pub headers: *const FrameworkTransferHeaderV1,
    /// Number of request headers.
    pub header_count: u64,
    /// Semantic application directory tag.
    pub directory: FrameworkTransferDirectory,
    /// Reserved; must be zero.
    pub reserved: u32,
    /// Relative destination path within `directory`.
    pub relative_path: FrameworkStr,
}

/// A fully overwritten view of one durable task snapshot.
#[repr(C)]
pub struct FrameworkTransferSnapshotViewV1 {
    /// Exact byte size of this record.
    pub struct_size: u32,
    /// ABI major version from `framework_abi_version`.
    pub abi_version: u32,
    /// App-assigned task ID.
    pub id: FrameworkTransferIdV1,
    /// Transfer-state tag.
    pub state: FrameworkTransferState,
    /// Portable error-kind tag for Failed; otherwise zero.
    pub failure_kind: FrameworkTransferErrorKind,
    /// Optional backend-native error code for Failed; otherwise zero.
    pub native_code: i32,
    /// HTTP response status for Succeeded; otherwise zero.
    pub http_status: u32,
    /// Reserved; always zero.
    pub reserved: u32,
    /// Reserved; always zero.
    pub reserved2: u32,
    /// Number of observed response headers for Succeeded; otherwise zero.
    pub response_header_count: u64,
}

/// One borrowed response-header view.
#[repr(C)]
pub struct FrameworkTransferHeaderViewV1 {
    /// ASCII HTTP token name.
    pub name: FrameworkStr,
    /// Opaque response field-value bytes.
    pub value: FrameworkSlice,
}

/// C callback used after a forwarded background-session event batch completes.
pub type FrameworkIosTransferEventsCompletion = unsafe extern "C" fn(context: *mut c_void);

/// Unique handle with one B13 `IosTransferBackend`; use only on the main thread.
pub struct FrameworkIosTransferClient {
    #[cfg(target_os = "ios")]
    transfers: framework_transfer::Transfers<::ios_transfer::IosTransferBackend>,
}

/// Owned D10 snapshot whose views may outlive its client.
pub struct FrameworkIosTransferSnapshot {
    #[cfg(target_os = "ios")]
    value: framework_transfer::TransferSnapshot,
}

#[cfg(target_os = "ios")]
use ::ios_transfer::{BackgroundEventError, IosTransferBackend};
#[cfg(target_os = "ios")]
use framework_core::{Availability, Error, ErrorKind};
#[cfg(target_os = "ios")]
use framework_files::{AppDirectory, AppPath};
#[cfg(target_os = "ios")]
use framework_network::{Header, HttpUrl};
#[cfg(target_os = "ios")]
use framework_transfer::{DownloadRequest, TransferError, TransferId, TransferStatus};
#[cfg(target_os = "ios")]
use objc2::MainThreadMarker;

fn empty_str() -> FrameworkStr {
    // SAFETY: A null pointer and zero length are valid empty FrameworkStr values.
    unsafe { zeroed() }
}

fn empty_slice() -> FrameworkSlice {
    // SAFETY: A null pointer and zero length are valid empty FrameworkSlice values.
    unsafe { zeroed() }
}

fn valid_span(data: *const u8, length: u64) -> bool {
    match usize::try_from(length) {
        Ok(length) if length <= isize::MAX as usize => {
            if length == 0 {
                data.is_null()
            } else {
                !data.is_null()
            }
        }
        _ => false,
    }
}

unsafe fn input_str<'a>(value: FrameworkStr) -> Result<&'a str, FrameworkStatus> {
    if !valid_span(value.data(), value.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The exported function requires each input span to stay readable and unchanged.
    unsafe { value.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

#[cfg(target_os = "ios")]
unsafe fn input_bytes<'a>(value: FrameworkSlice) -> Result<&'a [u8], FrameworkStatus> {
    if !valid_span(value.data(), value.length()) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The exported function requires each input span to stay readable and unchanged.
    unsafe { value.as_bytes() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

#[cfg(target_os = "ios")]
fn transfer_id(value: FrameworkTransferIdV1) -> Result<TransferId, FrameworkStatus> {
    let raw = (u128::from(value.high) << 64) | u128::from(value.low);
    TransferId::new(raw).ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

#[cfg(target_os = "ios")]
fn directory(value: FrameworkTransferDirectory) -> Result<AppDirectory, FrameworkStatus> {
    match value {
        FRAMEWORK_TRANSFER_DIRECTORY_DOCUMENTS => Ok(AppDirectory::Documents),
        FRAMEWORK_TRANSFER_DIRECTORY_CACHES => Ok(AppDirectory::Caches),
        FRAMEWORK_TRANSFER_DIRECTORY_TEMPORARY => Ok(AppDirectory::Temporary),
        FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT => Ok(AppDirectory::ApplicationSupport),
        _ => Err(FrameworkStatus::INVALID_ARGUMENT),
    }
}

#[cfg(target_os = "ios")]
unsafe fn request<'a>(
    value: *const FrameworkTransferRequestV1,
) -> Result<(TransferId, HttpUrl<'a>, Vec<Header<'a>>, AppPath<'a>), FrameworkStatus> {
    if value.is_null() {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: Read only the version prefix before the size check.
    let struct_size = unsafe { ptr::addr_of!((*value).struct_size).read_unaligned() };
    // SAFETY: Read only the version prefix before the size check.
    let abi_version = unsafe { ptr::addr_of!((*value).abi_version).read_unaligned() };
    if usize::try_from(struct_size).ok() != Some(size_of::<FrameworkTransferRequestV1>())
        || abi_version != (crate::framework_abi_version() >> 32) as u32
    {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    if (value as usize) % align_of::<FrameworkTransferRequestV1>() != 0 {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The caller supplies a readable, aligned full record after the size check.
    let value = unsafe { value.read() };
    if value.reserved != 0 {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    let id = transfer_id(value.id)?;
    // SAFETY: The C caller keeps these spans valid through this call.
    let url = unsafe { input_str(value.url) }?;
    let url = HttpUrl::new(url).map_err(|_| FrameworkStatus::INVALID_ARGUMENT)?;
    let count =
        usize::try_from(value.header_count).map_err(|_| FrameworkStatus::INVALID_ARGUMENT)?;
    if count > isize::MAX as usize / size_of::<FrameworkTransferHeaderV1>()
        || (count == 0) != value.headers.is_null()
        || (count != 0 && (value.headers as usize) % align_of::<FrameworkTransferHeaderV1>() != 0)
    {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    let raw_headers = if count == 0 {
        &[][..]
    } else {
        // SAFETY: The caller supplies an aligned readable array with `header_count` entries.
        unsafe { core::slice::from_raw_parts(value.headers, count) }
    };
    let mut headers = Vec::new();
    headers
        .try_reserve_exact(count)
        .map_err(|_| FrameworkStatus::RESOURCE_EXHAUSTED)?;
    for header in raw_headers {
        // SAFETY: Each member span follows the same call-lifetime contract as the request.
        let name = unsafe { input_str(header.name) }?;
        // SAFETY: Each member span follows the same call-lifetime contract as the request.
        let bytes = unsafe { input_bytes(header.value) }?;
        headers.push(Header::new(name, bytes).map_err(|_| FrameworkStatus::INVALID_ARGUMENT)?);
    }
    // SAFETY: The C caller keeps this path span valid through this call.
    let relative_path = unsafe { input_str(value.relative_path) }?;
    let destination = AppPath::new(directory(value.directory)?, relative_path)
        .map_err(|_| FrameworkStatus::INVALID_ARGUMENT)?;
    Ok((id, url, headers, destination))
}

#[cfg(target_os = "ios")]
fn transfer_status(error: TransferError) -> FrameworkStatus {
    FrameworkStatus::from_error(Error::new(error.kind()))
}

#[cfg(target_os = "ios")]
fn transfer_native_code(error: TransferError) -> i32 {
    error.platform_code().map_or(0, |code| code.get())
}

#[cfg(target_os = "ios")]
fn availability_tag(value: Availability) -> FrameworkTransferAvailability {
    match value {
        Availability::Unknown => FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN,
        Availability::Available => FRAMEWORK_TRANSFER_AVAILABILITY_AVAILABLE,
        Availability::Unsupported => FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED,
        Availability::RequiresPermission => FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_PERMISSION,
        Availability::RequiresEntitlement => FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_ENTITLEMENT,
        Availability::TemporarilyUnavailable => {
            FRAMEWORK_TRANSFER_AVAILABILITY_TEMPORARILY_UNAVAILABLE
        }
        _ => FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn error_kind_tag(value: ErrorKind) -> FrameworkTransferErrorKind {
    match value {
        ErrorKind::Unknown => FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN,
        ErrorKind::InvalidInput => FRAMEWORK_TRANSFER_ERROR_KIND_INVALID_INPUT,
        ErrorKind::Unsupported => FRAMEWORK_TRANSFER_ERROR_KIND_UNSUPPORTED,
        ErrorKind::Unavailable => FRAMEWORK_TRANSFER_ERROR_KIND_UNAVAILABLE,
        ErrorKind::PermissionDenied => FRAMEWORK_TRANSFER_ERROR_KIND_PERMISSION_DENIED,
        ErrorKind::Cancelled => FRAMEWORK_TRANSFER_ERROR_KIND_CANCELLED,
        ErrorKind::Timeout => FRAMEWORK_TRANSFER_ERROR_KIND_TIMEOUT,
        ErrorKind::NotFound => FRAMEWORK_TRANSFER_ERROR_KIND_NOT_FOUND,
        ErrorKind::AlreadyExists => FRAMEWORK_TRANSFER_ERROR_KIND_ALREADY_EXISTS,
        ErrorKind::ResourceExhausted => FRAMEWORK_TRANSFER_ERROR_KIND_RESOURCE_EXHAUSTED,
        ErrorKind::Platform => FRAMEWORK_TRANSFER_ERROR_KIND_PLATFORM,
        ErrorKind::Internal => FRAMEWORK_TRANSFER_ERROR_KIND_INTERNAL,
        _ => FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn state_view(
    snapshot: &framework_transfer::TransferSnapshot,
    output: &mut FrameworkTransferSnapshotViewV1,
) -> FrameworkStatus {
    let id = snapshot.id().get();
    match snapshot.status() {
        TransferStatus::Queued => {
            output.id = transfer_id_view(id);
            output.state = FRAMEWORK_TRANSFER_STATE_QUEUED;
        }
        TransferStatus::Active => {
            output.id = transfer_id_view(id);
            output.state = FRAMEWORK_TRANSFER_STATE_ACTIVE;
        }
        TransferStatus::Succeeded { status, headers } => {
            let Ok(count) = u64::try_from(headers.len()) else {
                return FrameworkStatus::RESOURCE_EXHAUSTED;
            };
            output.id = transfer_id_view(id);
            output.state = FRAMEWORK_TRANSFER_STATE_SUCCEEDED;
            output.http_status = u32::from(status.get());
            output.response_header_count = count;
        }
        TransferStatus::Failed(error) => {
            output.id = transfer_id_view(id);
            output.state = FRAMEWORK_TRANSFER_STATE_FAILED;
            output.failure_kind = error_kind_tag(error.kind());
            output.native_code = error.platform_code().map_or(0, |code| code.get());
        }
        TransferStatus::Cancelled => {
            output.id = transfer_id_view(id);
            output.state = FRAMEWORK_TRANSFER_STATE_CANCELLED;
        }
        _ => return FrameworkStatus::INTERNAL_ERROR,
    }
    FrameworkStatus::OK
}

#[cfg(target_os = "ios")]
fn transfer_id_view(value: u128) -> FrameworkTransferIdV1 {
    FrameworkTransferIdV1 {
        high: (value >> 64) as u64,
        low: value as u64,
    }
}

fn snapshot_view_zeroed() -> FrameworkTransferSnapshotViewV1 {
    FrameworkTransferSnapshotViewV1 {
        struct_size: size_of::<FrameworkTransferSnapshotViewV1>() as u32,
        abi_version: (crate::framework_abi_version() >> 32) as u32,
        id: FrameworkTransferIdV1 { high: 0, low: 0 },
        state: 0,
        failure_kind: 0,
        native_code: 0,
        http_status: 0,
        reserved: 0,
        reserved2: 0,
        response_header_count: 0,
    }
}

fn header_view_zeroed() -> FrameworkTransferHeaderViewV1 {
    FrameworkTransferHeaderViewV1 {
        name: empty_str(),
        value: empty_slice(),
    }
}

/// Creates one explicit B13 client for a stable background-session identifier.
///
/// # Safety
/// `session_identifier` must point to readable UTF-8 for the call. `out_client` must be writable
/// and must not contain a live handle. Call this and all other client operations on the main
/// thread. A non-null `out_native_code` must be writable and must not alias another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_client_create(
    session_identifier: FrameworkStr,
    out_client: *mut *mut FrameworkIosTransferClient,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    if out_client.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies an empty writable handle slot.
    unsafe { out_client.write(ptr::null_mut()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: The C caller keeps the session span valid through the constructor call.
        let identifier = match unsafe { input_str(session_identifier) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThreadMarker::new() else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            match IosTransferBackend::new(identifier, marker) {
                Ok(backend) => {
                    let client = FrameworkIosTransferClient {
                        transfers: framework_transfer::Transfers::new(backend),
                    };
                    // SAFETY: The output was initialized and is a unique writable slot.
                    unsafe { out_client.write(Box::into_raw(Box::new(client))) };
                    FrameworkStatus::OK
                }
                Err(error) => {
                    // SAFETY: The optional output remains writable and was zeroed on entry.
                    unsafe { write_native_code(out_native_code, transfer_native_code(error)) };
                    transfer_status(error)
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = identifier;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Destroys one client and clears its original pointer slot.
///
/// # Safety
/// `client` must be null or the original live slot returned by create. On iOS, an off-main call is
/// a no-op and leaves the slot unchanged; call on main after every accepted event callback. Do not
/// copy the handle, alias the slot, or race any operation with destroy. A null slot or null handle
/// is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_client_destroy(
    client: *mut *mut FrameworkIosTransferClient,
) {
    if client.is_null() {
        return;
    }
    #[cfg(target_os = "ios")]
    if MainThreadMarker::new().is_none() {
        return;
    }
    // SAFETY: The caller provides the original writable handle slot.
    let value = unsafe { client.read() };
    if value.is_null() {
        return;
    }
    // SAFETY: Clear the original slot before dropping its uniquely owned allocation.
    unsafe { client.write(ptr::null_mut()) };
    // SAFETY: Create returned this Box allocation and the caller has not copied its handle.
    unsafe { drop(Box::from_raw(value)) };
}

/// Returns availability for one client.
///
/// # Safety
/// `client` must be a live handle used only on the main thread. `out_availability` must be a
/// writable output that does not alias the handle or another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_availability(
    client: *const FrameworkIosTransferClient,
    out_availability: *mut FrameworkTransferAvailability,
) -> FrameworkStatus {
    if out_availability.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable output slot.
    unsafe { out_availability.write(FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let value = unsafe { &*client };
            // SAFETY: The caller supplies an output distinct from the client handle.
            unsafe { out_availability.write(availability_tag(value.transfers.availability())) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            // SAFETY: The caller supplies a writable output slot.
            unsafe { out_availability.write(FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED) };
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Persists one GET download request.
///
/// # Safety
/// The client must be live and used only on the main thread. The request record and non-empty
/// header array must meet their C type alignment. The request and all non-empty spans must stay
/// readable and unchanged through this call. `out_native_code`, when non-null, must be writable
/// and must not alias another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_start_download(
    client: *mut FrameworkIosTransferClient,
    request_value: *const FrameworkTransferRequestV1,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() || request_value.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The request pointer and spans meet this function's documented contract.
            let (id, url, headers, destination) = match unsafe { request(request_value) } {
                Ok(value) => value,
                Err(status) => return status,
            };
            let request = DownloadRequest::new(id, url, &headers, destination);
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let client = unsafe { &mut *client };
            match client.transfers.start_download(request) {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => {
                    // SAFETY: The optional output remains writable and was zeroed on entry.
                    unsafe { write_native_code(out_native_code, transfer_native_code(error)) };
                    transfer_status(error)
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = request_value;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Queries one durable task and returns an owned snapshot when present.
///
/// # Safety
/// The client must be live and used only on the main thread. Both required outputs must be
/// writable, distinct, and not alias the client. `out_snapshot` must not contain a live handle.
/// An optional native-code output must be writable and distinct from all other outputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_status(
    client: *mut FrameworkIosTransferClient,
    id: FrameworkTransferIdV1,
    out_snapshot: *mut *mut FrameworkIosTransferSnapshot,
    out_found: *mut u8,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    if !out_snapshot.is_null() {
        // SAFETY: The caller promises that each non-null required output is writable.
        unsafe { out_snapshot.write(ptr::null_mut()) };
    }
    if !out_found.is_null() {
        // SAFETY: The caller promises that each non-null required output is writable.
        unsafe { out_found.write(0) };
    }
    if out_snapshot.is_null() || out_found.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            let id = match transfer_id(id) {
                Ok(value) => value,
                Err(status) => return status,
            };
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let client = unsafe { &mut *client };
            match client.transfers.status(id) {
                Ok(None) => FrameworkStatus::OK,
                Ok(Some(value)) => {
                    let snapshot = FrameworkIosTransferSnapshot { value };
                    // SAFETY: Output slots were validated and initialized above.
                    unsafe {
                        out_snapshot.write(Box::into_raw(Box::new(snapshot)));
                        out_found.write(1);
                    }
                    FrameworkStatus::OK
                }
                Err(error) => {
                    // SAFETY: The optional output remains writable and was zeroed on entry.
                    unsafe { write_native_code(out_native_code, transfer_native_code(error)) };
                    transfer_status(error)
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = id;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Records a durable stop request for one task.
///
/// # Safety
/// The client must be live and used only on the main thread. An optional native-code output must
/// be writable and must not alias another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_cancel(
    client: *mut FrameworkIosTransferClient,
    id: FrameworkTransferIdV1,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            let id = match transfer_id(id) {
                Ok(value) => value,
                Err(status) => return status,
            };
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let client = unsafe { &mut *client };
            match client.transfers.cancel(id) {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => {
                    // SAFETY: The optional output remains writable and was zeroed on entry.
                    unsafe { write_native_code(out_native_code, transfer_native_code(error)) };
                    transfer_status(error)
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = id;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Removes one terminal task record and leaves its destination file.
///
/// # Safety
/// The client must be live and used only on the main thread. An optional native-code output must
/// be writable and must not alias another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_forget(
    client: *mut FrameworkIosTransferClient,
    id: FrameworkTransferIdV1,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            let id = match transfer_id(id) {
                Ok(value) => value,
                Err(status) => return status,
            };
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let client = unsafe { &mut *client };
            match client.transfers.forget(id) {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => {
                    // SAFETY: The optional output remains writable and was zeroed on entry.
                    unsafe { write_native_code(out_native_code, transfer_native_code(error)) };
                    transfer_status(error)
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = id;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Overwrites a full snapshot view without a read of prior bytes.
///
/// # Safety
/// `snapshot` must be a live handle. `out_view` must be writable for one complete V1 record and
/// must not alias the handle or another mutable access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_snapshot_get_view(
    snapshot: *const FrameworkIosTransferSnapshot,
    out_view: *mut FrameworkTransferSnapshotViewV1,
) -> FrameworkStatus {
    if out_view.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable full V1 output record; no prior bytes are read.
    unsafe { out_view.write(snapshot_view_zeroed()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if snapshot.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            // SAFETY: The caller keeps a live immutable snapshot handle.
            let value = unsafe { &*snapshot };
            // SAFETY: The output was fully initialized and is distinct from the snapshot.
            let output = unsafe { &mut *out_view };
            state_view(&value.value, output)
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Returns one borrowed response-header view by index.
///
/// # Safety
/// `snapshot` must be live; `out_header` must be writable and must not alias the snapshot. The
/// returned spans remain valid only until the snapshot is destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_snapshot_get_header(
    snapshot: *const FrameworkIosTransferSnapshot,
    index: u64,
    out_header: *mut FrameworkTransferHeaderViewV1,
) -> FrameworkStatus {
    if out_header.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable full output record.
    unsafe { out_header.write(header_view_zeroed()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if snapshot.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            let Ok(index) = usize::try_from(index) else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            // SAFETY: The caller keeps a live immutable snapshot handle.
            let value = unsafe { &*snapshot };
            let headers = match value.value.status() {
                TransferStatus::Succeeded { headers, .. } => headers,
                TransferStatus::Queued
                | TransferStatus::Active
                | TransferStatus::Failed(_)
                | TransferStatus::Cancelled => return FrameworkStatus::INVALID_ARGUMENT,
                _ => return FrameworkStatus::INTERNAL_ERROR,
            };
            let Some(header) = headers.get(index) else {
                return FrameworkStatus::NOT_FOUND;
            };
            let name = if header.name().is_empty() {
                empty_str()
            } else {
                match FrameworkStr::from_utf8(header.name()) {
                    Some(value) => value,
                    None => return FrameworkStatus::RESOURCE_EXHAUSTED,
                }
            };
            let bytes = header.value();
            let value = if bytes.is_empty() {
                empty_slice()
            } else {
                match FrameworkSlice::from_bytes(bytes) {
                    Some(value) => value,
                    None => return FrameworkStatus::RESOURCE_EXHAUSTED,
                }
            };
            // SAFETY: The caller supplies a writable output distinct from the immutable snapshot.
            unsafe { out_header.write(FrameworkTransferHeaderViewV1 { name, value }) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = index;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Destroys one snapshot and clears its original pointer slot.
///
/// # Safety
/// `snapshot` must be null or the original live slot returned by status. Destroy once; do not
/// copy the handle, destroy an alias, race a view call with destroy, or use borrowed views after
/// destroy. A null slot or null handle is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_snapshot_destroy(
    snapshot: *mut *mut FrameworkIosTransferSnapshot,
) {
    if snapshot.is_null() {
        return;
    }
    // SAFETY: The caller supplies the original writable handle slot.
    let value = unsafe { snapshot.read() };
    if value.is_null() {
        return;
    }
    // SAFETY: Clear the original slot before dropping its uniquely owned allocation.
    unsafe { snapshot.write(ptr::null_mut()) };
    // SAFETY: Status returned this Box allocation and the caller has not copied its handle.
    unsafe { drop(Box::from_raw(value)) };
}

/// Forwards UIKit's background-session event to the client for that session.
///
/// # Safety
/// The client and UTF-8 session span must remain valid through the call. The completion function
/// must be non-null and must not unwind; `context` must stay valid until the callback runs exactly
/// once on the main dispatch queue. Retain the client until callback completion. A rejected call
/// does not take callback or context ownership. Run this function on the main thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_forward_background_events(
    client: *mut FrameworkIosTransferClient,
    session_identifier: FrameworkStr,
    completion: FrameworkIosTransferEventsCompletion,
    context: *mut c_void,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        // SAFETY: The C caller keeps the session identifier span valid through the call.
        let identifier = match unsafe { input_str(session_identifier) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        #[cfg(target_os = "ios")]
        {
            let Some(marker) = MainThreadMarker::new() else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            // SAFETY: The caller upholds callback/context lifetime and no-unwind requirements.
            let client = unsafe { &mut *client };
            match unsafe {
                client
                    .transfers
                    .backend_mut()
                    .handle_background_events_c(marker, identifier, completion, context)
            } {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => background_event_status(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (identifier, completion, context);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Classifies an ordinary launch after UIKit rules out a background-session event.
///
/// # Safety
/// The client must be live and used only on the main thread. Do not also forward a background
/// event for this launch. An optional native-code output must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_transfer_finish_launch_without_background_events(
    client: *mut FrameworkIosTransferClient,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that an optional native-code output is writable.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if client.is_null() {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            if MainThreadMarker::new().is_none() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps a live, exclusive client against destroy on the main thread.
            let client = unsafe { &mut *client };
            match client
                .transfers
                .backend_mut()
                .finish_launch_without_background_events()
            {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => background_event_status(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

#[cfg(target_os = "ios")]
fn background_event_status(error: BackgroundEventError) -> FrameworkStatus {
    match error {
        BackgroundEventError::SessionIdentifierMismatch
        | BackgroundEventError::LaunchAlreadyClassified => FrameworkStatus::INVALID_ARGUMENT,
        BackgroundEventError::CompletionAlreadyPending => FrameworkStatus::ALREADY_EXISTS,
        _ => FrameworkStatus::INTERNAL_ERROR,
    }
}

unsafe fn initialize_native_code(output: *mut i32) {
    if !output.is_null() {
        // SAFETY: The caller promises a writable optional native-code output.
        unsafe { output.write(0) };
    }
}

#[cfg(target_os = "ios")]
unsafe fn write_native_code(output: *mut i32, value: i32) {
    if !output.is_null() {
        // SAFETY: The caller promises a writable optional native-code output.
        unsafe { output.write(value) };
    }
}
