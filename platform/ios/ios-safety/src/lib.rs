#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS SafetyKit Crash Detection availability query"]

use objc2_safety_kit::SACrashDetectionManager;

/// Read whether the current device supports Crash Detection
///
/// This is a point-in-time device capability bit, not app authorization or proof that an app has
/// the entitlement required for Crash Detection event access. It does not ask for permission,
/// create a manager instance, set a delegate, or read event data
///
/// This returns `false` below iOS 16.0. Apple docs state that Crash Detection requires the
/// `com.apple.developer.severe-vehicular-crash-event` entitlement for event access; Apple does not
/// specify whether this availability getter alone has the same entitlement prerequisite
pub fn is_crash_detection_available() -> bool {
    if !objc2::available!(ios = 16.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 16.0 class API floor; the getter has no pointer inputs
    unsafe { SACrashDetectionManager::isAvailable() }
}
