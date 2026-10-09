#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow WidgetKit all-timeline reload request for iOS"]

/// Failure to ask WidgetKit to reload configured widget timelines
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidgetKitReloadError {
    /// WidgetKit symbols are unavailable or the target is not iOS
    NativeApiUnavailable,
    /// Swift returned no WidgetCenter metadata or singleton object
    NativeValueUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_widgetkit_center_create(center_out: *mut *mut c_void) -> u8;
    fn framework_widgetkit_reload_all_timelines(center: *mut c_void) -> u8;
}

/// Requests a timeline reload for every configured widget belonging to the containing app
///
/// On iOS 14.0 and later this calls `WidgetCenter.shared.reloadAllTimelines()` through
/// compiler-derived Swift-call thunks. The native method returns no completion or result; `Ok(())`
/// means only that the synchronous request call returned. WidgetKit controls provider invocation,
/// refresh budgets, and rendering time. This function does not prove that a widget is configured,
/// that a provider ran successfully, or that a view was rendered
///
/// The call does not implement a widget extension, provider, timeline, or SwiftUI view. Non-iOS
/// targets return `NativeApiUnavailable`
pub fn request_reload_all_timelines() -> Result<(), WidgetKitReloadError> {
    #[cfg(target_os = "ios")]
    {
        let mut raw_center: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes to this valid output pointer and checks weak-linked symbols
        let create_result = unsafe { framework_widgetkit_center_create(&mut raw_center) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }

        // SAFETY: the shared getter returns an owned WidgetCenter reference; this wrapper keeps it
        // alive through the synchronous reload request and releases it with swift-abi-core
        let Some(center) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_center.cast::<SwiftObject>())
        }) else {
            return Err(WidgetKitReloadError::NativeValueUnavailable);
        };

        // SAFETY: the retained object is the exact class returned by `WidgetCenter.shared`
        let reload_result =
            unsafe { framework_widgetkit_reload_all_timelines(center.as_ptr().cast::<c_void>()) };
        drop(center);
        if reload_result != 0 {
            return Err(map_native_result(reload_result));
        }
        Ok(())
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(WidgetKitReloadError::NativeApiUnavailable)
    }
}

#[cfg(target_os = "ios")]
fn map_native_result(code: u8) -> WidgetKitReloadError {
    match code {
        1 => WidgetKitReloadError::NativeApiUnavailable,
        2 => WidgetKitReloadError::NativeValueUnavailable,
        _ => WidgetKitReloadError::NativeBridgeFailure,
    }
}
