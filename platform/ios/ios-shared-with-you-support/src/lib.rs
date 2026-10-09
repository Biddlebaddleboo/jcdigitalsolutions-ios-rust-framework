#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Point-in-time SharedWithYou system collaboration status for iOS"]

#[cfg(target_os = "ios")]
use objc2_shared_with_you::SWHighlightCenter;

/// Reads `SWHighlightCenter::isSystemCollaborationSupportAvailable`
///
/// This reports Apple's software-version status for full Messages collaboration support only. It
/// does not report app-specific access, a user's account state, highlight access, CloudKit state,
/// permission, UI, or collaboration success
///
/// This returns `false` below iOS 16.0
#[cfg(target_os = "ios")]
pub fn is_system_collaboration_support_available() -> bool {
    if !objc2::available!(ios = 16.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 16.0 class API floor; the getter takes no pointers
    unsafe { SWHighlightCenter::isSystemCollaborationSupportAvailable() }
}
