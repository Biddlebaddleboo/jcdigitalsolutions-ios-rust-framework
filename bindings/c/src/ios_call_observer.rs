use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use ios_call_observer::active_call_snapshot;

/// Fixed-width bitmask for aggregate CallKit call state.
pub type FrameworkIosCallObserverStateFlags = u32;

/// At least one returned call is outgoing.
pub const FRAMEWORK_IOS_CALL_OBSERVER_STATE_OUTGOING: FrameworkIosCallObserverStateFlags = 1 << 0;
/// At least one returned call is connected.
pub const FRAMEWORK_IOS_CALL_OBSERVER_STATE_CONNECTED: FrameworkIosCallObserverStateFlags = 1 << 1;
/// At least one returned call is on hold.
pub const FRAMEWORK_IOS_CALL_OBSERVER_STATE_ON_HOLD: FrameworkIosCallObserverStateFlags = 1 << 2;
/// At least one returned call has ended.
pub const FRAMEWORK_IOS_CALL_OBSERVER_STATE_ENDED: FrameworkIosCallObserverStateFlags = 1 << 3;

/// Reads one synchronous snapshot of CallKit's returned call list.
///
/// The count is the number of calls in `CXCallObserver.calls`. The flags are copied unchanged
/// from D76 and are independent ORed facts across that list; they do not correlate with one call.
/// This call may block while CallKit retrieves its initial state. Keep it off UI-critical work.
///
/// # Safety
/// Each non-null output must point to aligned writable storage. Both outputs are required and
/// must be distinct and must not overlap. Inspect either output only when `FRAMEWORK_STATUS_OK`
/// is returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_call_observer_active_call_snapshot(
    out_call_count: *mut u64,
    out_state_flags: *mut FrameworkIosCallObserverStateFlags,
) -> FrameworkStatus {
    // SAFETY: The caller promises each non-null output is writable.
    unsafe {
        if !out_call_count.is_null() {
            out_call_count.write(0);
        }
        if !out_state_flags.is_null() {
            out_state_flags.write(0);
        }
    }
    if out_call_count.is_null() || out_state_flags.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let snapshot = active_call_snapshot();
            // SAFETY: Required outputs are writable and disjoint by the caller contract.
            unsafe {
                out_call_count.write(snapshot.active_call_count());
                out_state_flags.write(snapshot.state_flags());
            }
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
