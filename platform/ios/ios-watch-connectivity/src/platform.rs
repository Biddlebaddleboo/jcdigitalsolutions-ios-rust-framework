use framework_watch_connectivity::{WatchConnectivityBackend, WatchConnectivitySupport};
use objc2_watch_connectivity::WCSession;

/// Queries whether this iOS device supports Watch Connectivity session objects.
///
/// This adapter does not retrieve or activate the default session, inspect pairing or installed
/// app state, or communicate with a watch. The support result is not a connectivity guarantee.
pub struct IosWatchConnectivityBackend;

impl WatchConnectivityBackend for IosWatchConnectivityBackend {
    fn session_support() -> WatchConnectivitySupport {
        // SAFETY: `isSupported` is the documented class-level capability query. It requires no
        // session instance and does not require session activation or pairing state.
        if unsafe { WCSession::isSupported() } {
            WatchConnectivitySupport::Supported
        } else {
            WatchConnectivitySupport::Unsupported
        }
    }
}
