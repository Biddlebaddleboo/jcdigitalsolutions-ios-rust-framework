#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow DockKit system-tracking setting snapshot for iOS devices"]

/// Failure to read DockKit system-tracking state
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DockKitError {
    /// DockKit symbols are unavailable or the target has no DockKit backend
    NativeApiUnavailable,
    /// Swift returned no manager metadata or singleton object
    NativeValueUnavailable,
    /// The native bridge received an invalid pointer or returned an unknown result
    NativeBridgeFailure,
}

#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
use core::ffi::c_void;
#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
unsafe extern "C" {
    fn framework_dockkit_manager_create(manager_out: *mut *mut c_void) -> u8;
    fn framework_dockkit_is_system_tracking_enabled(
        manager: *mut c_void,
        enabled_out: *mut u8,
    ) -> u8;
}

/// Reads whether DockKit system tracking is enabled
///
/// On iOS 17.0 and later this calls `DockAccessoryManager.shared.isSystemTrackingEnabled` through
/// compiler-derived Swift-call thunks. The value reports the system-tracking setting only; it does
/// not show that an accessory is connected, tracking is active, a camera is available, or an
/// accessory operation will succeed. The call does not set the system-tracking value, read camera
/// frames, or observe `accessoryStateChanges`
///
/// The installed iOS Simulator SDK does not include DockKit, so Simulator and non-iOS targets
/// return `NativeApiUnavailable`. The API does not establish a queue or main-thread contract
pub fn system_tracking_enabled() -> Result<bool, DockKitError> {
    #[cfg(all(target_os = "ios", not(target_abi = "sim")))]
    {
        let mut raw_manager: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes to this valid output pointer and checks weak-linked symbols
        let create_result = unsafe { framework_dockkit_manager_create(&mut raw_manager) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }

        // SAFETY: the shared getter returns an owned DockAccessoryManager reference; this wrapper
        // keeps it alive through the synchronous property read and releases it with swift-abi-core
        let Some(manager) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_manager.cast::<SwiftObject>())
        }) else {
            return Err(DockKitError::NativeValueUnavailable);
        };

        let mut enabled = 0_u8;
        // SAFETY: the retained object is the exact class returned by `DockAccessoryManager.shared`
        let read_result = unsafe {
            framework_dockkit_is_system_tracking_enabled(
                manager.as_ptr().cast::<c_void>(),
                &mut enabled,
            )
        };
        drop(manager);
        if read_result != 0 {
            return Err(map_native_result(read_result));
        }
        match enabled {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DockKitError::NativeBridgeFailure),
        }
    }

    #[cfg(any(not(target_os = "ios"), target_abi = "sim"))]
    {
        Err(DockKitError::NativeApiUnavailable)
    }
}

#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
fn map_native_result(code: u8) -> DockKitError {
    match code {
        1 => DockKitError::NativeApiUnavailable,
        2 => DockKitError::NativeValueUnavailable,
        _ => DockKitError::NativeBridgeFailure,
    }
}
