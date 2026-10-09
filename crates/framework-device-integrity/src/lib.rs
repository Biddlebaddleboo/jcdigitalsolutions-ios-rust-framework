#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, non-security-asserting App Attest and DeviceCheck support values."]

/// A point-in-time report of whether the platform APIs report DeviceCheck and App Attest support.
///
/// These booleans mirror the corresponding native `isSupported` properties. A `true` value does
/// not promise that token generation, key generation, attestation, or assertions will succeed. It
/// is not proof of device integrity, app identity, enrollment, or server-verified trust. Treat each
/// value as a transient capability hint, not as a durable or security-sensitive decision.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AvailabilitySnapshot {
    device_check_supported: bool,
    app_attest_supported: bool,
}

impl AvailabilitySnapshot {
    /// Creates a snapshot from the two platform-reported support values.
    pub const fn new(device_check_supported: bool, app_attest_supported: bool) -> Self {
        Self {
            device_check_supported,
            app_attest_supported,
        }
    }

    /// Returns whether the platform reports the DeviceCheck API as supported.
    pub const fn device_check_supported(self) -> bool {
        self.device_check_supported
    }

    /// Returns whether the platform reports the App Attest service as supported.
    pub const fn app_attest_supported(self) -> bool {
        self.app_attest_supported
    }
}
