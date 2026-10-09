use core::ffi::c_void;
use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use ios_system_services::is_classkit_deep_link;
#[cfg(target_os = "ios")]
use objc2_foundation::NSUserActivity;

/// Fixed-width Boolean result for one ClassKit deep-link marker query.
pub type FrameworkIosClassKitBoolean = u8;

/// Reads only `NSUserActivity.isClassKitDeepLink` from a caller-owned activity.
///
/// The getter is available from iOS 11.3. The activity is borrowed only for this synchronous
/// call; the wrapper does not retain, store, or release it. This marker does not expose ClassKit
/// contexts, assignment data, or user identity.
///
/// # Safety
/// On iOS, `activity` must be a non-null pointer to a live `NSUserActivity` for the full call, and
/// its owner must keep the object alive in accordance with the host activity lifecycle and thread
/// rules. `out_is_deep_link` must point to aligned writable storage that does not overlap the
/// activity object. On non-iOS targets, a non-null activity pointer is checked but never
/// dereferenced. A non-null output is initialized to zero before input or availability checks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_classkit_is_deep_link(
    activity: *const c_void,
    out_is_deep_link: *mut FrameworkIosClassKitBoolean,
) -> FrameworkStatus {
    if out_is_deep_link.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises that the non-null output pointer is writable.
    unsafe { out_is_deep_link.write(0) };
    if activity.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if !objc2::available!(ios = 11.3, ..) {
                return FrameworkStatus::UNAVAILABLE;
            }
            // SAFETY: The caller promises a live, strongly held NSUserActivity object for this
            // synchronous call. The C wrapper only borrows the object and does not retain it.
            let activity = unsafe { &*activity.cast::<NSUserActivity>() };
            let is_deep_link = is_classkit_deep_link(activity);
            // SAFETY: The caller supplied aligned writable output storage above.
            unsafe { out_is_deep_link.write(u8::from(is_deep_link)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = activity;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
