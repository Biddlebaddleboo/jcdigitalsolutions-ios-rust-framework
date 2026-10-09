#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A read-only caller-app Personal VPN status query for iOS."]

mod completion;
mod platform;

pub use framework_vpn::{PersonalVpnLoadError, PersonalVpnQueryError, PersonalVpnStatus};
pub use platform::IosPersonalVpnStatusFuture;

use ios_runtime::main_thread::MainThread;

/// Starts a one-shot read of the calling app's Personal VPN connection status.
///
/// The native preference load starts when this function runs, not when the returned future is
/// first polled. The `MainThread` proof keeps the future on the caller's main thread, where Apple
/// documents that the load completion runs. Dropping the future abandons Rust interest but does
/// not cancel the native request. This request requires the host app's `allow-vpn` entitlement.
pub fn request_personal_vpn_status(main_thread: MainThread) -> IosPersonalVpnStatusFuture {
    platform::start(main_thread)
}
