#[cfg(target_os = "ios")]
use alloc::boxed::Box;
#[cfg(target_os = "ios")]
use alloc::string::String;
use core::ffi::c_void;
use core::mem::{align_of, size_of};
use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{ABI_VERSION_MAJOR, FrameworkStatus, FrameworkStr, catch_unwind_status};

#[cfg(target_os = "ios")]
use framework_sharing::{ShareError, ShareItem, ShareOutcome, ShareRequest};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIView, UIViewController};

/// Fixed-width share-item tag
pub type FrameworkIosShareItem = u32;
/// A UTF-8 text item
pub const FRAMEWORK_IOS_SHARE_ITEM_TEXT: FrameworkIosShareItem = 0;
/// A URL-text item
pub const FRAMEWORK_IOS_SHARE_ITEM_URL: FrameworkIosShareItem = 1;

/// Fixed-width share-outcome tag
pub type FrameworkIosShareOutcome = u32;
/// UIKit reported completion; this does not prove recipient delivery
pub const FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED: FrameworkIosShareOutcome = 0;
/// The backend reported dismissal or cancellation without completion; no actor is identified
pub const FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED: FrameworkIosShareOutcome = 1;

/// Fixed-width share-availability tag
pub type FrameworkIosShareAvailability = u32;
/// Availability is not known
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN: FrameworkIosShareAvailability = 0;
/// Share is available
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE: FrameworkIosShareAvailability = 1;
/// Share is not supported
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED: FrameworkIosShareAvailability = 2;
/// Share requires permission
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION: FrameworkIosShareAvailability = 3;
/// Share requires an entitlement
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT: FrameworkIosShareAvailability = 4;
/// Share is not available at this time
pub const FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE: FrameworkIosShareAvailability =
    5;

/// One opaque iOS share session
pub struct FrameworkIosShareSession {
    #[cfg(target_os = "ios")]
    session: ios_sharing::IosShareSession,
}

/// One tagged text or URL-text item
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkIosShareItemV1 {
    /// Text or URL-text tag
    pub kind: FrameworkIosShareItem,
    /// Must be zero in V1
    pub reserved: u32,
    /// Borrowed UTF-8 payload
    pub text: FrameworkStr,
}

/// One borrowed, ordered share request
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkIosShareRequestV1 {
    /// Exact size of this record
    pub struct_size: u32,
    /// ABI major from `framework_abi_version`
    pub abi_version: u32,
    /// Borrowed item array
    pub items: *const FrameworkIosShareItemV1,
    /// Item count
    pub item_count: u64,
}

/// One source-view rectangle for iPad popover context
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameworkIosShareAnchorV1 {
    /// Exact size of this record
    pub struct_size: u32,
    /// ABI major from `framework_abi_version`
    pub abi_version: u32,
    /// Source-view x coordinate
    pub x: f64,
    /// Source-view y coordinate
    pub y: f64,
    /// Anchor width
    pub width: f64,
    /// Anchor height
    pub height: f64,
}

/// One-shot C callback for a UIKit terminal result; accepted start may not invoke it
pub type FrameworkIosShareCompletion = Option<
    unsafe extern "C" fn(
        context: *mut c_void,
        status: FrameworkStatus,
        outcome: FrameworkIosShareOutcome,
        native_code: i32,
    ),
>;

fn valid_span(value: FrameworkStr) -> bool {
    match usize::try_from(value.length()) {
        Ok(length) if length <= isize::MAX as usize => {
            if length == 0 {
                value.data().is_null()
            } else {
                !value.data().is_null()
            }
        }
        _ => false,
    }
}

unsafe fn input_text<'a>(value: FrameworkStr) -> Result<&'a str, FrameworkStatus> {
    if !valid_span(value) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C host keeps each canonical span readable and unchanged through this call
    unsafe { value.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

fn valid_anchor(value: FrameworkIosShareAnchorV1) -> bool {
    value.struct_size as usize == size_of::<FrameworkIosShareAnchorV1>()
        && value.abi_version == ABI_VERSION_MAJOR
        && value.x.is_finite()
        && value.y.is_finite()
        && value.width.is_finite()
        && value.height.is_finite()
        && value.width >= 0.0
        && value.height >= 0.0
}

unsafe fn checked_items<'a>(
    request: *const FrameworkIosShareRequestV1,
) -> Result<&'a [FrameworkIosShareItemV1], FrameworkStatus> {
    if request.is_null() || (request as usize) % align_of::<FrameworkIosShareRequestV1>() != 0 {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C host supplies a readable version prefix
    let struct_size = unsafe { ptr::addr_of!((*request).struct_size).read_unaligned() };
    // SAFETY: The C host supplies a readable version prefix
    let abi_version = unsafe { ptr::addr_of!((*request).abi_version).read_unaligned() };
    if struct_size as usize != size_of::<FrameworkIosShareRequestV1>()
        || abi_version != ABI_VERSION_MAJOR
    {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C host supplies an aligned, readable V1 record after the prefix check
    let request = unsafe { request.read() };
    let count =
        usize::try_from(request.item_count).map_err(|_| FrameworkStatus::INVALID_ARGUMENT)?;
    if count == 0
        || count > isize::MAX as usize / size_of::<FrameworkIosShareItemV1>()
        || request.items.is_null()
        || (request.items as usize) % align_of::<FrameworkIosShareItemV1>() != 0
    {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C host supplies an aligned readable array for the declared item count
    let items = unsafe { core::slice::from_raw_parts(request.items, count) };
    for item in items {
        if item.reserved != 0
            || !matches!(
                item.kind,
                FRAMEWORK_IOS_SHARE_ITEM_TEXT | FRAMEWORK_IOS_SHARE_ITEM_URL
            )
        {
            return Err(FrameworkStatus::INVALID_ARGUMENT);
        }
        // SAFETY: Item payloads use the same call-lifetime rule as the request record
        unsafe { input_text(item.text) }?;
    }
    Ok(items)
}

#[cfg(target_os = "ios")]
unsafe fn owned_item(item: &FrameworkIosShareItemV1) -> Result<ShareItem, FrameworkStatus> {
    // SAFETY: `checked_items` validated this borrowed span for the same call
    let text = unsafe { input_text(item.text) }?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(text.len())
        .map_err(|_| FrameworkStatus::RESOURCE_EXHAUSTED)?;
    owned.push_str(text);
    match item.kind {
        FRAMEWORK_IOS_SHARE_ITEM_TEXT => Ok(ShareItem::text(owned)),
        FRAMEWORK_IOS_SHARE_ITEM_URL => Ok(ShareItem::url(owned)),
        _ => Err(FrameworkStatus::INVALID_ARGUMENT),
    }
}

#[cfg(target_os = "ios")]
unsafe fn owned_request(
    request: *const FrameworkIosShareRequestV1,
) -> Result<ShareRequest, FrameworkStatus> {
    // SAFETY: The C host keeps the record, array, and spans live through this synchronous copy
    let items = unsafe { checked_items(request) }?;
    let mut items = items.iter();
    let first = items.next().ok_or(FrameworkStatus::INVALID_ARGUMENT)?;
    // SAFETY: `checked_items` validated each item span
    let mut request = ShareRequest::new(unsafe { owned_item(first) }?);
    for item in items {
        // SAFETY: `checked_items` validated each item span
        request.push(unsafe { owned_item(item) }?);
    }
    Ok(request)
}

#[cfg(target_os = "ios")]
fn error_status(value: ShareError) -> FrameworkStatus {
    match value {
        ShareError::Backend(error) => FrameworkStatus::from_error(error),
        _ => FrameworkStatus::INTERNAL_ERROR,
    }
}

#[cfg(target_os = "ios")]
fn error_native_code(value: ShareError) -> i32 {
    value.platform_code().map_or(0, |code| code.get())
}

#[cfg(target_os = "ios")]
fn availability_tag(value: framework_core::Availability) -> FrameworkIosShareAvailability {
    match value {
        framework_core::Availability::Unknown => FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN,
        framework_core::Availability::Available => FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE,
        framework_core::Availability::Unsupported => FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED,
        framework_core::Availability::RequiresPermission => {
            FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION
        }
        framework_core::Availability::RequiresEntitlement => {
            FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT
        }
        framework_core::Availability::TemporarilyUnavailable => {
            FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE
        }
        _ => FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn callback_values(
    result: Result<ShareOutcome, ShareError>,
) -> (FrameworkStatus, FrameworkIosShareOutcome, i32) {
    match result {
        Ok(ShareOutcome::Completed) => (
            FrameworkStatus::OK,
            FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED,
            0,
        ),
        Ok(ShareOutcome::Dismissed) => (
            FrameworkStatus::OK,
            FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED,
            0,
        ),
        Ok(_) => (FrameworkStatus::INTERNAL_ERROR, 0, 0),
        Err(error) => (error_status(error), 0, error_native_code(error)),
    }
}

/// Create one session with retained UIKit context
///
/// # Safety
/// `presenter` and `source_view` must be live Objective-C objects of the stated UIKit types through
/// this call. `out_session` must name aligned writable storage that does not alias an input or live
/// handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_share_session_create(
    presenter: *mut c_void,
    source_view: *mut c_void,
    anchor: FrameworkIosShareAnchorV1,
    out_session: *mut *mut FrameworkIosShareSession,
) -> FrameworkStatus {
    if out_session.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The C host supplies one writable output slot
    unsafe { out_session.write(ptr::null_mut()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        if presenter.is_null() || source_view.is_null() || !valid_anchor(anchor) {
            return FrameworkStatus::INVALID_ARGUMENT;
        }
        #[cfg(target_os = "ios")]
        {
            let Some(main_thread) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let rectangle = CGRect::new(
                CGPoint::new(anchor.x, anchor.y),
                CGSize::new(anchor.width, anchor.height),
            );
            // SAFETY: The C host promises live objects of these UIKit types through this call
            let Some(presenter) =
                (unsafe { Retained::<UIViewController>::retain(presenter.cast()) })
            else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            // SAFETY: The C host promises live objects of these UIKit types through this call
            let Some(source_view) = (unsafe { Retained::<UIView>::retain(source_view.cast()) })
            else {
                return FrameworkStatus::INVALID_ARGUMENT;
            };
            let session =
                ios_sharing::IosShareSession::new(main_thread, presenter, source_view, rectangle);
            // SAFETY: The output slot was initialized and is writable for this one handle
            unsafe {
                out_session.write(Box::into_raw(Box::new(FrameworkIosShareSession {
                    session,
                })))
            };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Return share availability without a prompt
///
/// # Safety
/// `session` must be null or a live handle used only on the main thread. `out_availability` must
/// name aligned writable storage that does not alias the handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_share_session_availability(
    session: *const FrameworkIosShareSession,
    out_availability: *mut FrameworkIosShareAvailability,
) -> FrameworkStatus {
    if out_availability.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The C host supplies one writable output slot
    unsafe { out_availability.write(FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if session.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The C host promises one live handle on the main thread
            let session = unsafe { &*session };
            // SAFETY: The output slot is distinct from the live handle by contract
            unsafe { out_availability.write(availability_tag(session.session.availability())) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            if session.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The output slot was validated and initialized above
            unsafe { out_availability.write(FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED) };
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Start one share request
///
/// # Safety
/// `session` must be a live unique handle on the main thread. `request` and all spans must stay
/// readable and unchanged through this call. A non-null callback must not unwind or reenter F7
/// If UIKit reports a terminal result, the callback runs once after this function returns. UIKit
/// may report no result, so accepted start does not guarantee a callback. Keep callback and context
/// host-live until callback return, successful cancel, or destroy
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_share_start(
    session: *mut FrameworkIosShareSession,
    request: *const FrameworkIosShareRequestV1,
    completion: FrameworkIosShareCompletion,
    context: *mut c_void,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        let Some(completion) = completion else {
            return FrameworkStatus::INVALID_ARGUMENT;
        };
        #[cfg(not(target_os = "ios"))]
        let _ = (completion, context);
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if session.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The C host keeps all request spans valid through this synchronous copy
            let request = match unsafe { owned_request(request) } {
                Ok(value) => value,
                Err(status) => return status,
            };
            // SAFETY: The C host promises one live unique handle on the main thread
            let session = unsafe { &mut *session };
            let callback = move |result| {
                let (status, outcome, native_code) = callback_values(result);
                // SAFETY: The C host keeps context live through callback or successful cancel/destroy
                unsafe { completion(context, status, outcome, native_code) };
            };
            match session.session.start(request, callback) {
                Ok(()) => FrameworkStatus::OK,
                Err(start_error) => {
                    let status = error_status(start_error.error);
                    let _ = start_error.callback;
                    status
                }
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            if session.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The C host supplies a readable request and item array for this call
            if let Err(status) = unsafe { checked_items(request) } {
                return status;
            }
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Detach one active share result callback
///
/// # Safety
/// `session` must be a live unique handle used only on the main thread
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_share_cancel(
    session: *mut FrameworkIosShareSession,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if session.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The C host promises one live unique handle on the main thread
            let session = unsafe { &mut *session };
            if session.session.cancel() {
                FrameworkStatus::OK
            } else {
                FrameworkStatus::NOT_FOUND
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            if session.is_null() {
                FrameworkStatus::INVALID_ARGUMENT
            } else {
                FrameworkStatus::UNSUPPORTED
            }
        }
    }))
}

/// Destroy one session and clear its original handle slot
///
/// # Safety
/// `session` must be null or the original aligned writable slot from create. On iOS, call only on the
/// main thread and do not copy the handle, race a call, or destroy an alias
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_share_session_destroy(
    session: *mut *mut FrameworkIosShareSession,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        if MainThread::current().is_none() {
            return FrameworkStatus::UNAVAILABLE;
        }
        if session.is_null() {
            return FrameworkStatus::OK;
        }
        // SAFETY: The C host supplies the original writable handle slot
        let value = unsafe { session.read() };
        if value.is_null() {
            return FrameworkStatus::OK;
        }
        #[cfg(target_os = "ios")]
        {
            // SAFETY: Clear the unique original slot before dropping its allocation
            unsafe { session.write(ptr::null_mut()) };
            // SAFETY: Create returned this unique box and no alias may be destroyed
            unsafe { drop(Box::from_raw(value)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
