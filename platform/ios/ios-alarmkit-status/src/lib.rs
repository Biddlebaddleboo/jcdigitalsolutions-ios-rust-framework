#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow AlarmKit authorization-state snapshot for iOS"]

/// The current authorization state reported by AlarmKit
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlarmAuthorizationState {
    /// The app has not yet requested alarm authorization
    NotDetermined,
    /// Alarm authorization is denied
    Denied,
    /// Alarm authorization is granted
    Authorized,
    /// AlarmKit returned a case not known to this crate version
    Unknown,
}

/// Failure to read the current AlarmKit authorization state
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlarmKitError {
    /// AlarmKit symbols are unavailable or the target is not iOS
    NativeApiUnavailable,
    /// AlarmKit returned no manager or authorization-state metadata
    NativeValueUnavailable,
    /// The compiler-derived Swift ABI bridge returned an invalid value
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use swift_abi_core::{SwiftObject, SwiftRetained};

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_alarm_manager_create(manager_out: *mut *mut c_void) -> u8;
    fn framework_alarm_manager_authorization_state(manager: *mut c_void, state_out: *mut u8) -> u8;
}

/// Reads the current authorization state without requesting authorization
///
/// On iOS 26.0 and later this reads `AlarmManager.shared.authorizationState` through a
/// compiler-derived Swift-call thunk and the enum's value-witness table. It does not call
/// `requestAuthorization()`, prompt the person, schedule or alter an alarm, list alarm content,
/// or observe authorization updates. The value is a point-in-time state; it does not promise a
/// later schedule operation will succeed. Non-iOS targets return `NativeApiUnavailable`
pub fn authorization_state() -> Result<AlarmAuthorizationState, AlarmKitError> {
    #[cfg(target_os = "ios")]
    {
        let mut raw_manager: *mut c_void = core::ptr::null_mut();
        // SAFETY: the C bridge writes to this valid output pointer and returns the owned result of
        // AlarmManager.shared only after checking the weak-imported metadata and getter symbols.
        let create_result = unsafe { framework_alarm_manager_create(&mut raw_manager) };
        if create_result != 0 {
            return Err(map_native_result(create_result));
        }
        // SAFETY: success transfers one owned AlarmManager class reference to this wrapper.
        let Some(manager) = (unsafe {
            SwiftRetained::<SwiftObject>::from_owned_ptr(raw_manager.cast::<SwiftObject>())
        }) else {
            return Err(AlarmKitError::NativeValueUnavailable);
        };

        let mut raw_state = u8::MAX;
        // SAFETY: `manager` keeps the exact AlarmManager instance alive for the synchronous getter;
        // the C bridge allocates metadata-sized/aligned opaque storage, reads the enum tag through
        // its value witnesses, destroys the initialized value, and writes this valid output byte.
        let read_result = unsafe {
            framework_alarm_manager_authorization_state(
                manager.as_ptr().cast::<c_void>(),
                &mut raw_state,
            )
        };
        drop(manager);
        if read_result != 0 {
            return Err(map_native_result(read_result));
        }
        match raw_state {
            0 => Ok(AlarmAuthorizationState::NotDetermined),
            1 => Ok(AlarmAuthorizationState::Denied),
            2 => Ok(AlarmAuthorizationState::Authorized),
            3 => Ok(AlarmAuthorizationState::Unknown),
            _ => Err(AlarmKitError::NativeBridgeFailure),
        }
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err(AlarmKitError::NativeApiUnavailable)
    }
}

#[cfg(target_os = "ios")]
fn map_native_result(code: u8) -> AlarmKitError {
    match code {
        1 => AlarmKitError::NativeApiUnavailable,
        2 => AlarmKitError::NativeValueUnavailable,
        _ => AlarmKitError::NativeBridgeFailure,
    }
}
