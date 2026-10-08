use alloc::boxed::Box;
use alloc::string::String;
use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkStatus, FrameworkStr, catch_unwind_status};
use framework_notifications::NotificationId;
use framework_notifications::response::{
    NotificationActionId, NotificationResponse, NotificationResponseKind,
};

/// Fixed-width response-kind tag for the notification-response C ABI
pub type FrameworkNotificationResponseKind = u32;

/// Standard/default notification activation tag
pub const FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT: FrameworkNotificationResponseKind = 0;
/// Notification dismissal tag
pub const FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS: FrameworkNotificationResponseKind = 1;
/// Custom action tag
pub const FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION: FrameworkNotificationResponseKind = 2;
/// Text-input action tag
pub const FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT: FrameworkNotificationResponseKind = 3;

/// Opaque notification-response value with unique ownership
pub struct FrameworkNotificationResponse {
    value: NotificationResponse,
}

/// V1 C layout for response fields
#[repr(C)]
pub struct FrameworkNotificationResponseViewV1 {
    /// Response-kind tag
    pub kind: FrameworkNotificationResponseKind,
    /// Reserved field; always zero
    pub reserved: u32,
    /// Exact notification ID span; present for every kind
    pub notification_id: FrameworkStr,
    /// Exact action ID span; present only for custom and text-input actions
    pub action_id: FrameworkStr,
    /// User text span; present only for text-input actions
    pub user_text: FrameworkStr,
}

fn valid_span(value: FrameworkStr) -> bool {
    match usize::try_from(value.length()) {
        Ok(length) => {
            length <= isize::MAX as usize
                && if length == 0 {
                    value.data().is_null()
                } else {
                    !value.data().is_null()
                }
        }
        Err(_) => false,
    }
}

fn unused(value: FrameworkStr) -> bool {
    value.data().is_null() && value.length() == 0
}

unsafe fn input_text<'a>(value: FrameworkStr) -> Result<&'a str, FrameworkStatus> {
    if !valid_span(value) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: C code must keep these UTF-8 bytes valid and fixed until create returns
    unsafe { value.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

unsafe fn owned_text(value: FrameworkStr) -> Result<String, FrameworkStatus> {
    // SAFETY: the exported create function sets the input span lifetime and valid-byte contract
    let value = unsafe { input_text(value) }?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| FrameworkStatus::RESOURCE_EXHAUSTED)?;
    owned.push_str(value);
    Ok(owned)
}

unsafe fn create(
    notification_id: FrameworkStr,
    kind: FrameworkNotificationResponseKind,
    action_id: FrameworkStr,
    user_text: FrameworkStr,
    out_response: *mut *mut FrameworkNotificationResponse,
) -> FrameworkStatus {
    if out_response.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: C code must supply an output slot with write access, distinct from inputs and handles
    unsafe { out_response.write(ptr::null_mut()) };

    if !valid_span(notification_id) || !valid_span(action_id) || !valid_span(user_text) {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    let response_kind = match kind {
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT if unused(action_id) && unused(user_text) => {
            NotificationResponseKind::Default
        }
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS if unused(action_id) && unused(user_text) => {
            NotificationResponseKind::Dismiss
        }
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION if unused(user_text) => {
            // SAFETY: C code must keep these UTF-8 bytes valid and fixed until create returns
            let action = match unsafe { owned_text(action_id) } {
                Ok(value) => value,
                Err(status) => return status,
            };
            let action = match NotificationActionId::new(action) {
                Ok(value) => value,
                Err(_) => return FrameworkStatus::INVALID_ARGUMENT,
            };
            NotificationResponseKind::CustomAction(action)
        }
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT => {
            // SAFETY: C code must keep these UTF-8 bytes valid and fixed until create returns
            let action = match unsafe { owned_text(action_id) } {
                Ok(value) => value,
                Err(status) => return status,
            };
            let action = match NotificationActionId::new(action) {
                Ok(value) => value,
                Err(_) => return FrameworkStatus::INVALID_ARGUMENT,
            };
            // SAFETY: C code must keep these UTF-8 bytes valid and fixed until create returns
            let text = match unsafe { owned_text(user_text) } {
                Ok(value) => value,
                Err(status) => return status,
            };
            NotificationResponseKind::TextInput {
                action_id: action,
                text,
            }
        }
        _ => return FrameworkStatus::INVALID_ARGUMENT,
    };

    // SAFETY: C code must keep these UTF-8 bytes valid and fixed until create returns
    let notification_id = match unsafe { owned_text(notification_id) } {
        Ok(value) => value,
        Err(status) => return status,
    };
    let notification_id = match NotificationId::new(notification_id) {
        Ok(value) => value,
        Err(_) => return FrameworkStatus::INVALID_ARGUMENT,
    };
    let response = NotificationResponse::new(notification_id, response_kind);
    let response = Box::into_raw(Box::new(FrameworkNotificationResponse { value: response }));
    // SAFETY: C code must supply an output slot with write access, distinct from inputs and handles
    unsafe { out_response.write(response) };
    FrameworkStatus::OK
}

/// Create an owned response value from UTF-8 spans
///
/// # Safety
/// Each non-empty span must point to valid UTF-8 bytes; C code must not alter them before return
/// Zero-length spans must set data to null. `out_response` must name aligned output storage with
/// write access, distinct from all input spans and live handle slots. Its current value must not
/// name a live handle
/// The new handle must not be copied or used once destroy runs
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_notification_response_create(
    notification_id: FrameworkStr,
    kind: FrameworkNotificationResponseKind,
    action_id: FrameworkStr,
    user_text: FrameworkStr,
    out_response: *mut *mut FrameworkNotificationResponse,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: C code must uphold this function's documented address contract
        unsafe { create(notification_id, kind, action_id, user_text, out_response) }
    }))
}

fn empty_view() -> FrameworkNotificationResponseViewV1 {
    // SAFETY: null data and zero lengths are valid FrameworkStr values; all other fields are integers
    unsafe { core::mem::zeroed() }
}

fn empty_span() -> FrameworkStr {
    // SAFETY: null data and zero length form the required empty FrameworkStr value
    unsafe { core::mem::zeroed() }
}

fn view_text(value: &str) -> Result<FrameworkStr, FrameworkStatus> {
    if value.is_empty() {
        return Ok(empty_span());
    }
    FrameworkStr::from_utf8(value).ok_or(FrameworkStatus::RESOURCE_EXHAUSTED)
}

unsafe fn get_view(
    response: *const FrameworkNotificationResponse,
    out_view: *mut FrameworkNotificationResponseViewV1,
) -> FrameworkStatus {
    if out_view.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: C code must supply an output slot with write access, distinct from the live handle
    unsafe { out_view.write(empty_view()) };
    if response.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: C code must supply a live handle from create that is not past destroy
    let response = unsafe { &*response };
    let notification_id = match view_text(response.value.notification_id().as_str()) {
        Ok(value) => value,
        Err(status) => return status,
    };
    let mut view = empty_view();
    view.notification_id = notification_id;
    match response.value.kind() {
        NotificationResponseKind::Default => {
            view.kind = FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT;
        }
        NotificationResponseKind::Dismiss => {
            view.kind = FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS;
        }
        NotificationResponseKind::CustomAction(action_id) => {
            view.kind = FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION;
            view.action_id = match view_text(action_id.as_str()) {
                Ok(value) => value,
                Err(status) => return status,
            };
        }
        NotificationResponseKind::TextInput { action_id, text } => {
            view.kind = FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT;
            view.action_id = match view_text(action_id.as_str()) {
                Ok(value) => value,
                Err(status) => return status,
            };
            view.user_text = match view_text(text) {
                Ok(value) => value,
                Err(status) => return status,
            };
        }
        _ => return FrameworkStatus::INTERNAL_ERROR,
    }
    // SAFETY: the output slot has write access and starts at zero
    unsafe { out_view.write(view) };
    FrameworkStatus::OK
}

/// Return borrowed fields from a live response value
///
/// # Safety
/// `response` must be null or a live handle from create. A non-null output must name aligned storage
/// with write access, distinct from the handle. Keep the handle live for the call and do not race
/// get_view with destroy. View spans borrow from the handle and expire at destroy
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_notification_response_get_view(
    response: *const FrameworkNotificationResponse,
    out_view: *mut FrameworkNotificationResponseViewV1,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: C code must uphold this function's documented address contract
        unsafe { get_view(response, out_view) }
    }))
}

/// Destroy the original response handle and clear its original slot
///
/// # Safety
/// A non-null slot must name aligned storage with write access. A non-null handle must be the live
/// original handle from create; do not race with get_view, destroy an alias, or use a view once this call returns
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_notification_response_destroy(
    response: *mut *mut FrameworkNotificationResponse,
) {
    if response.is_null() {
        return;
    }
    // SAFETY: C code must supply a slot with write access and null or a live original handle
    let handle = unsafe { response.read() };
    if handle.is_null() {
        return;
    }
    // SAFETY: the slot has write access and holds null before the value drop
    unsafe { response.write(ptr::null_mut()) };
    // SAFETY: create yields this unique Box address; no alias may pass to Box::from_raw
    unsafe { drop(Box::from_raw(handle)) };
}
