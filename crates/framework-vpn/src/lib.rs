#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Personal VPN status, configuration-flag, and preference-load error values."]

extern crate alloc;

use alloc::string::String;

/// One status value reported by the calling app's Personal VPN connection.
///
/// `Invalid` is the native `NEVPNStatusInvalid` value, not a claim about device-wide VPN state.
/// An unknown raw value is preserved so a newer system status is not confused with a known state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PersonalVpnStatus {
    /// The calling app's Personal VPN profile is invalid or not configured.
    Invalid,
    /// The profile's VPN connection is disconnected.
    Disconnected,
    /// The profile's VPN connection is connecting.
    Connecting,
    /// The profile's VPN connection is connected.
    Connected,
    /// The profile's VPN connection is reasserting after loss of underlying connectivity.
    Reasserting,
    /// The profile's VPN connection is disconnecting.
    Disconnecting,
    /// The system returned a status value unknown to this crate.
    Unknown(i64),
}

impl PersonalVpnStatus {
    /// Maps an `NEVPNStatus` raw value without discarding unrecognized values.
    pub const fn from_native_raw(raw: i64) -> Self {
        match raw {
            0 => Self::Invalid,
            1 => Self::Disconnected,
            2 => Self::Connecting,
            3 => Self::Connected,
            4 => Self::Reasserting,
            5 => Self::Disconnecting,
            other => Self::Unknown(other),
        }
    }

    /// Returns the corresponding `NEVPNStatus` raw value.
    pub const fn native_raw(self) -> i64 {
        match self {
            Self::Invalid => 0,
            Self::Disconnected => 1,
            Self::Connecting => 2,
            Self::Connected => 3,
            Self::Reasserting => 4,
            Self::Disconnecting => 5,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Read-only enabled flags from the calling app's Personal VPN configuration.
///
/// These values describe configuration properties after a successful preference load. They do
/// not report a connection state, prove Connect On Demand rules exist or run, or describe another
/// app's configuration. Apple may set `enabled` to `false` when another Personal VPN configuration
/// is enabled.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PersonalVpnConfigurationFlags {
    enabled: bool,
    on_demand_enabled: bool,
}

impl PersonalVpnConfigurationFlags {
    /// Creates a snapshot of the two public `NEVPNManager` enabled properties.
    pub const fn new(enabled: bool, on_demand_enabled: bool) -> Self {
        Self {
            enabled,
            on_demand_enabled,
        }
    }

    /// Returns whether the loaded Personal VPN configuration is enabled.
    ///
    /// This may be `false` because another Personal VPN configuration is enabled. It is not a
    /// device-wide VPN state or a guarantee that a tunnel can connect.
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    /// Returns whether Connect On Demand is enabled for the loaded configuration.
    ///
    /// This flag alone does not prove that rules are configured or that an automatic connection
    /// will occur.
    pub const fn on_demand_enabled(self) -> bool {
        self.on_demand_enabled
    }
}

/// An owned native error returned while loading Personal VPN preferences.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PersonalVpnLoadError {
    domain: String,
    code: i64,
}

impl PersonalVpnLoadError {
    /// Creates an owned error from the native error domain and `NSInteger` code.
    pub fn new(domain: String, code: i64) -> Self {
        Self { domain, code }
    }

    /// Returns the owned native error domain.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the native `NSInteger` error code.
    pub const fn code(&self) -> i64 {
        self.code
    }
}

/// An error from one Personal VPN status request.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PersonalVpnQueryError {
    /// Loading Personal VPN preferences failed with this native error.
    PreferenceLoad(PersonalVpnLoadError),
    /// The native completion did not run on the documented caller main thread.
    CallbackThreadMismatch,
    /// A Rust panic occurred while the native completion was processed.
    CallbackPanicked,
}
