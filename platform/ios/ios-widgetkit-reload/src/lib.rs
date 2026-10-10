#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Narrow WidgetKit management requests for iOS"]

/// Failure while calling a synchronous WidgetKit management operation
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidgetKitError {
    /// The required WidgetKit symbol is unavailable or the target is not iOS
    NativeApiUnavailable,
    /// Swift returned no WidgetCenter metadata or singleton object
    NativeValueUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

/// Backward-compatible name for [`WidgetKitError`]
pub type WidgetKitReloadError = WidgetKitError;

#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_widgetkit_center_create(center_out: *mut *mut c_void) -> u8;
    fn framework_widgetkit_reload_all_timelines(center: *mut c_void) -> u8;
    fn framework_widgetkit_invalidate_configuration_recommendations(center: *mut c_void) -> u8;
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

/// Invalidates and refreshes preconfigured intent configurations for user-customizable widgets
///
/// On iOS 16.0 and later this calls
/// `WidgetCenter.shared.invalidateConfigurationRecommendations()` through a compiler-derived
/// weak `swiftcall` thunk. The native method is synchronous and returns no result; `Ok(())` means
/// only that the call returned. Apple documents this method as inactive on iOS, so this call does
/// not promise that recommendations change or appear. It does not run a provider or render a
/// widget. On earlier iOS versions, and on non-iOS targets, this returns
/// [`WidgetKitError::NativeApiUnavailable`]
pub fn invalidate_configuration_recommendations() -> Result<(), WidgetKitError> {
    #[cfg(target_os = "ios")]
    {
        let mut raw_center: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes to this valid output pointer and checks weak-linked base symbols
        let create_result = unsafe { framework_widgetkit_center_create(&mut raw_center) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }

        // SAFETY: the shared getter returns an owned WidgetCenter reference; this wrapper keeps it
        // alive through the synchronous call and releases it with swift-abi-core
        let Some(center) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_center.cast::<SwiftObject>())
        }) else {
            return Err(WidgetKitError::NativeValueUnavailable);
        };

        // SAFETY: the retained object is the exact class returned by `WidgetCenter.shared`
        let result = unsafe {
            framework_widgetkit_invalidate_configuration_recommendations(
                center.as_ptr().cast::<c_void>(),
            )
        };
        drop(center);
        if result != 0 {
            return Err(map_native_result(result));
        }
        Ok(())
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(WidgetKitError::NativeApiUnavailable)
    }
}

#[cfg(target_os = "ios")]
fn map_native_result(code: u8) -> WidgetKitError {
    match code {
        1 => WidgetKitError::NativeApiUnavailable,
        2 => WidgetKitError::NativeValueUnavailable,
        _ => WidgetKitError::NativeBridgeFailure,
    }
}
