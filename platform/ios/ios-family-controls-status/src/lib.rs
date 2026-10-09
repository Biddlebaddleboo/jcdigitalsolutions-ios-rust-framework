#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Read-only Family Controls authorization status for iOS"]

/// Raw signed Swift `AuthorizationStatus.RawValue`
///
/// This wrapper keeps unknown future raw values intact. It does not assign meaning to a raw value
/// or claim that a status grants Family Controls entitlement, activity data access, or control use
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct AuthorizationStatusRawValue(i64);

impl AuthorizationStatusRawValue {
    /// Creates a raw status value from any signed Swift `Int`
    pub const fn from_raw_value(raw_value: i64) -> Self {
        Self(raw_value)
    }

    /// Returns the exact signed Swift `Int` value
    pub const fn raw_value(self) -> i64 {
        self.0
    }
}

/// Status snapshot error
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationStatusError {
    /// The iOS 15 API or one of its weak-linked symbols is not present
    NativeApiUnavailable,
    /// Swift returned no metadata or singleton value
    NativeValueUnavailable,
    /// Swift value layout did not pass its ABI checks
    InvalidNativeLayout,
    /// Swift value storage allocation failed
    AllocationFailed,
    /// The native bridge returned an unrecognized failure code
    NativeBridgeFailure,
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_family_controls_status_snapshot(raw_value: *mut i64) -> u8;
}

/// Reads `AuthorizationCenter.shared.authorizationStatus.rawValue`
///
/// Apple requires this property read on the main dispatch queue. The unsafe call makes that queue
/// requirement visible; it does not hop queues. It does not request or revoke authorization, show
/// UI, read activity data, or use DeviceActivity or ManagedSettings. The raw signed value keeps each
/// SDK case, such as `approvedWithDataAccess`, distinct and keeps unknown future values intact
///
/// The host app must own any Family Controls entitlement setup for its broader feature. This read
/// does not inspect or prove that entitlement, Apple's distribution approval, activity-data access,
/// or the ability to apply controls
///
/// This API is available from iOS 15.0. Non-iOS targets have no backend
#[cfg(target_os = "ios")]
///
/// # Safety
///
/// The current code must run on the main dispatch queue
pub unsafe fn authorization_status_raw_value_on_main_queue()
-> Result<AuthorizationStatusRawValue, AuthorizationStatusError> {
    let mut raw_value = 0_i64;
    // SAFETY: the bridge accepts a valid output pointer and checks all Swift metadata and storage
    let result = unsafe { framework_family_controls_status_snapshot(&mut raw_value) };
    match result {
        0 => Ok(AuthorizationStatusRawValue::from_raw_value(raw_value)),
        1 => Err(AuthorizationStatusError::NativeApiUnavailable),
        2 => Err(AuthorizationStatusError::NativeValueUnavailable),
        3 => Err(AuthorizationStatusError::InvalidNativeLayout),
        4 => Err(AuthorizationStatusError::AllocationFailed),
        _ => Err(AuthorizationStatusError::NativeBridgeFailure),
    }
}
