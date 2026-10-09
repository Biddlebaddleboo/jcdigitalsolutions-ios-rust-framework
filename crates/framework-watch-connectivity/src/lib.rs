#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable support-status contract for Watch Connectivity session capability."]

/// Whether the current platform can provide a Watch Connectivity session object.
///
/// `Supported` describes platform capability only. It does not report whether a watch is paired,
/// whether a counterpart app is installed, whether a session is active, or whether communication
/// can succeed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum WatchConnectivitySupport {
    /// The platform does not provide a Watch Connectivity session object on this device.
    Unsupported,
    /// The platform provides a Watch Connectivity session object on this device.
    Supported,
}

/// A compile-time-selected backend for a Watch Connectivity platform-support query.
///
/// This contract does not create or activate a session, inspect pairing state, or communicate
/// with another device.
pub trait WatchConnectivityBackend {
    /// Queries whether the platform provides a session object on this device.
    fn session_support() -> WatchConnectivitySupport;
}
