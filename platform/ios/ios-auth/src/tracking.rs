use framework_auth::{AppTrackingAuthorizationBackend, AppTrackingAuthorizationStatus};
use objc2_app_tracking_transparency::{ATTrackingManager, ATTrackingManagerAuthorizationStatus};

/// A stateless backend for the calling app's App Tracking Transparency status.
///
/// `status` reads only `ATTrackingManager.trackingAuthorizationStatus`; it does not request
/// authorization or provide status for any other privacy feature.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IosAppTrackingAuthorizationBackend;

impl IosAppTrackingAuthorizationBackend {
    /// Creates the stateless backend without a native call.
    pub const fn new() -> Self {
        Self
    }
}

impl AppTrackingAuthorizationBackend for IosAppTrackingAuthorizationBackend {
    fn status(&self) -> AppTrackingAuthorizationStatus {
        read_status()
    }
}

fn read_status() -> AppTrackingAuthorizationStatus {
    // SAFETY: The host must use an iOS 14.0 or later deployment target, matching the SDK
    // availability of `ATTrackingManager`. This generated class getter takes no object, pointer,
    // callback, or caller-owned storage and returns a copyable status enum. Apple documents it as
    // a status read; unlike `requestTrackingAuthorizationWithCompletionHandler:`, it does not
    // request authorization or present a prompt.
    let native = unsafe { ATTrackingManager::trackingAuthorizationStatus() };
    map_status(native)
}

fn map_status(native: ATTrackingManagerAuthorizationStatus) -> AppTrackingAuthorizationStatus {
    if native == ATTrackingManagerAuthorizationStatus::NotDetermined {
        AppTrackingAuthorizationStatus::NotDetermined
    } else if native == ATTrackingManagerAuthorizationStatus::Restricted {
        AppTrackingAuthorizationStatus::Restricted
    } else if native == ATTrackingManagerAuthorizationStatus::Denied {
        AppTrackingAuthorizationStatus::Denied
    } else if native == ATTrackingManagerAuthorizationStatus::Authorized {
        AppTrackingAuthorizationStatus::Authorized
    } else {
        AppTrackingAuthorizationStatus::Unknown
    }
}
