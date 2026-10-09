#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow ActivityKit Live Activity start-eligibility snapshot for iOS."]

/// Failure to read the current app's ActivityKit start-eligibility value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityAuthorizationError {
    /// The required ActivityKit API symbols are unavailable on this runtime.
    NativeApiUnavailable,
    /// ActivityKit returned no authorization-info object or class metadata.
    NativeValueUnavailable,
    /// The native bridge reported an invalid pointer or an unrecognized result.
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_activity_authorization_info_create(info_out: *mut *mut c_void) -> u8;
    fn framework_activity_authorization_info_are_activities_enabled(
        info: *mut c_void,
        enabled_out: *mut u8,
    ) -> u8;
}

/// Reads whether ActivityKit currently allows this app to start a Live Activity.
///
/// On iOS this calls only `ActivityAuthorizationInfo.areActivitiesEnabled` through a
/// compiler-derived Swift-call thunk. The synchronous snapshot does not create, update, end, or
/// observe a Live Activity and does not show UI. `Ok(true)` means only that ActivityKit reports
/// this app can start one now; a later start may still fail, for example when the device reaches
/// its active and scheduled activity limit. Offering Live Activities also requires the host's
/// `NSSupportsLiveActivities` property-list key and a WidgetKit/SwiftUI presentation, neither of
/// which this crate configures or implements. The API floor is iOS 16.1.
///
/// Non-iOS targets return `NativeApiUnavailable`.
pub fn activities_enabled_for_current_app() -> Result<bool, ActivityAuthorizationError> {
    #[cfg(target_os = "ios")]
    {
        let mut raw_info: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes only to this valid out pointer and checks weak-linked API
        // symbols before invoking the compiler-derived metadata accessor and initializer.
        let create_result = unsafe { framework_activity_authorization_info_create(&mut raw_info) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }

        // SAFETY: success from the C constructor transfers the initializer's owned Swift
        // reference to this wrapper, which releases it through the audited swift-abi-core path.
        let Some(info) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_info.cast::<SwiftObject>())
        }) else {
            return Err(ActivityAuthorizationError::NativeValueUnavailable);
        };

        let mut enabled = 0_u8;
        // SAFETY: `info` remains alive for the synchronous getter call, its type is the exact
        // ActivityAuthorizationInfo class returned by the constructor, and the out pointer is valid.
        let read_result = unsafe {
            framework_activity_authorization_info_are_activities_enabled(
                info.as_ptr().cast::<c_void>(),
                &mut enabled,
            )
        };
        drop(info);
        if read_result != 0 {
            return Err(map_native_result(read_result));
        }
        match enabled {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ActivityAuthorizationError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(ActivityAuthorizationError::NativeApiUnavailable)
    }
}

#[cfg(target_os = "ios")]
fn map_native_result(code: u8) -> ActivityAuthorizationError {
    match code {
        1 => ActivityAuthorizationError::NativeApiUnavailable,
        2 => ActivityAuthorizationError::NativeValueUnavailable,
        _ => ActivityAuthorizationError::NativeBridgeFailure,
    }
}
