#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Bluetooth authorization and bounded central-discovery contracts."]

/// The normalized authorization state for Bluetooth use.
///
/// `Allowed` means the operating system authorizes Bluetooth use. It does not report whether the
/// radio is powered on, whether a particular Bluetooth role is supported, or whether a scan or
/// connection can succeed. This authorization value itself does not start a scan, connection, or
/// peripheral operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothAuthorization {
    /// The native state is not recognized or cannot be classified.
    Unknown = 0,
    /// The user has not yet made an authorization choice.
    NotDetermined = 1,
    /// Access is blocked by policy or device restrictions.
    Restricted = 2,
    /// The user denied Bluetooth access.
    Denied = 3,
    /// Bluetooth access is authorized by the operating system.
    Allowed = 4,
}

impl BluetoothAuthorization {
    /// Reports whether the state authorizes Bluetooth use.
    pub const fn allows_bluetooth_use(self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// A fixed-width opaque identifier assigned by CoreBluetooth to a discovered peer.
///
/// The 16 bytes use the octet order of the peer's canonical UUID string, not host integer byte
/// order. This is an operating-system peer identifier, not a Bluetooth address, hardware serial
/// number, or cross-platform identity guarantee.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BluetoothPeripheralId([u8; 16]);

impl BluetoothPeripheralId {
    /// Creates an identifier from the 16 UUID bytes in canonical UUID-string order.
    pub const fn from_uuid_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the 16 UUID bytes in canonical UUID-string order.
    pub const fn as_uuid_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// One asynchronous central-discovery callback copied into portable fixed-width values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BluetoothDiscovery {
    /// The opaque operating-system peer identifier.
    pub peripheral: BluetoothPeripheralId,
    /// Received signal strength in dBm, or `None` when unavailable or outside the portable range.
    pub rssi_dbm: Option<i32>,
}

/// The current lifecycle state of an explicitly requested central scan.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothScanState {
    /// No scan has been requested, or the caller stopped it.
    Stopped = 0,
    /// CoreBluetooth has not yet reported `PoweredOn` for this request.
    Starting = 1,
    /// The backend issued the scan after the manager reported powered-on.
    Scanning = 2,
    /// A requested scan is waiting for the radio or manager to become ready.
    WaitingForPower = 3,
    /// The manager reports that Bluetooth use is not authorized.
    Unauthorized = 4,
    /// The device or platform does not support the central role.
    Unsupported = 5,
    /// The backend saw an unrecognized manager state or could not process a native callback.
    Unavailable = 6,
}

/// A synchronous error from explicitly requesting a scan that is already active or pending.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothScanError {
    /// A scan is already requested; stop it before starting another scan.
    AlreadyActive = 0,
}

/// A statically selected backend for explicit, unfiltered central discovery.
///
/// `start_unfiltered_scan` only requests a scan and returns before manager readiness or discovery
/// results are known. A caller observes later callback results by polling
/// `try_next_discovery`; a backend may bound its queue and report how many callbacks it drops.
/// Calling `stop_scan` ends the request but does not revoke values already queued. This contract
/// does not request a connection, retain a native peripheral object, or expose advertisement
/// data. Authorization may be queried separately through [`BluetoothAuthorizationBackend`]; an
/// explicit scan request may itself trigger a platform permission prompt.
pub trait BluetoothCentralBackend {
    /// Explicitly requests an unfiltered scan, returning before asynchronous readiness/results.
    fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError>;

    /// Stops an active or pending scan. Already queued discoveries remain available to drain.
    fn stop_scan(&mut self);

    /// Returns the current backend-observed lifecycle state.
    fn scan_state(&self) -> BluetoothScanState;

    /// Removes one previously delivered discovery, if one is queued.
    fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery>;

    /// Returns and resets the number of discovery callbacks dropped due to bounded queue capacity.
    fn take_dropped_discovery_count(&mut self) -> u32;
}

/// A thin facade over caller-owned central-discovery backend state.
pub struct BluetoothCentral<B> {
    backend: B,
}

impl<B: BluetoothCentralBackend> BluetoothCentral<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Explicitly requests an unfiltered scan.
    pub fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError> {
        self.backend.start_unfiltered_scan()
    }

    /// Stops an active or pending scan.
    pub fn stop_scan(&mut self) {
        self.backend.stop_scan();
    }

    /// Returns the backend-observed scan lifecycle state.
    pub fn scan_state(&self) -> BluetoothScanState {
        self.backend.scan_state()
    }

    /// Removes one asynchronously delivered discovery, if one is queued.
    pub fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery> {
        self.backend.try_next_discovery()
    }

    /// Returns and resets the number of discovery callbacks dropped due to bounded queue capacity.
    pub fn take_dropped_discovery_count(&mut self) -> u32 {
        self.backend.take_dropped_discovery_count()
    }
}

/// A statically selected backend for a non-prompting Bluetooth authorization query.
///
/// The query must not instantiate a central/peripheral manager, request permission, prompt the
/// user, power on the radio, scan, connect, or advertise. The returned state is only an
/// authorization snapshot and is not proof that Bluetooth is available or that an operation can
/// succeed. Backends must map unrecognized or unclassifiable native authorization states to
/// [`BluetoothAuthorization::Unknown`]. No global service, dynamic dispatch, or executor is
/// required by this contract.
pub trait BluetoothAuthorizationBackend {
    /// Queries the normalized authorization state without prompting or starting Bluetooth use.
    fn authorization_status(&self) -> BluetoothAuthorization;
}

/// A thin facade over caller-owned, statically selected Bluetooth backend state.
pub struct Bluetooth<B> {
    backend: B,
}

impl<B: BluetoothAuthorizationBackend> Bluetooth<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Queries authorization without initializing a manager or prompting.
    pub fn authorization_status(&self) -> BluetoothAuthorization {
        self.backend.authorization_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend(BluetoothAuthorization);

    impl BluetoothAuthorizationBackend for FakeBackend {
        fn authorization_status(&self) -> BluetoothAuthorization {
            self.0
        }
    }

    #[test]
    fn status_has_fixed_width_distinct_values_and_only_allowed_means_authorized() {
        assert_eq!(core::mem::size_of::<BluetoothAuthorization>(), 1);
        assert_eq!(core::mem::size_of::<BluetoothScanState>(), 1);
        assert_eq!(core::mem::size_of::<BluetoothScanError>(), 1);
        assert_eq!(
            [
                BluetoothAuthorization::Unknown as u8,
                BluetoothAuthorization::NotDetermined as u8,
                BluetoothAuthorization::Restricted as u8,
                BluetoothAuthorization::Denied as u8,
                BluetoothAuthorization::Allowed as u8,
            ],
            [0, 1, 2, 3, 4],
        );
        assert!(BluetoothAuthorization::Allowed.allows_bluetooth_use());
        assert!(!BluetoothAuthorization::Unknown.allows_bluetooth_use());
        assert!(!BluetoothAuthorization::NotDetermined.allows_bluetooth_use());
        assert!(!BluetoothAuthorization::Restricted.allows_bluetooth_use());
        assert!(!BluetoothAuthorization::Denied.allows_bluetooth_use());
    }

    #[test]
    fn facade_uses_caller_selected_backend_and_preserves_status() {
        for status in [
            BluetoothAuthorization::Unknown,
            BluetoothAuthorization::NotDetermined,
            BluetoothAuthorization::Restricted,
            BluetoothAuthorization::Denied,
            BluetoothAuthorization::Allowed,
        ] {
            let bluetooth = Bluetooth::new(FakeBackend(status));
            assert_eq!(bluetooth.authorization_status(), status);
        }
    }

    struct FakeCentral {
        state: BluetoothScanState,
        discovery: Option<BluetoothDiscovery>,
        dropped: u32,
    }

    impl BluetoothCentralBackend for FakeCentral {
        fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError> {
            if self.state != BluetoothScanState::Stopped {
                return Err(BluetoothScanError::AlreadyActive);
            }
            self.state = BluetoothScanState::Starting;
            Ok(())
        }

        fn stop_scan(&mut self) {
            self.state = BluetoothScanState::Stopped;
        }

        fn scan_state(&self) -> BluetoothScanState {
            self.state
        }

        fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery> {
            self.discovery.take()
        }

        fn take_dropped_discovery_count(&mut self) -> u32 {
            core::mem::take(&mut self.dropped)
        }
    }

    #[test]
    fn discovery_values_are_fixed_width_and_preserve_optional_rssi() {
        let id = BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]);
        let discovery = BluetoothDiscovery {
            peripheral: id,
            rssi_dbm: None,
        };
        assert_eq!(core::mem::size_of::<BluetoothPeripheralId>(), 16);
        assert_eq!(id.as_uuid_bytes(), &[0xA5; 16]);
        assert_eq!(discovery.rssi_dbm, None);
        assert_eq!(
            BluetoothDiscovery {
                peripheral: id,
                rssi_dbm: Some(-42),
            }
            .rssi_dbm,
            Some(-42)
        );
    }

    #[test]
    fn central_facade_exposes_explicit_scan_and_poll_contract() {
        let discovery = BluetoothDiscovery {
            peripheral: BluetoothPeripheralId::from_uuid_bytes([0x2A; 16]),
            rssi_dbm: Some(-58),
        };
        let backend = FakeCentral {
            state: BluetoothScanState::Stopped,
            discovery: Some(discovery),
            dropped: 3,
        };
        let mut central = BluetoothCentral::new(backend);
        assert_eq!(central.scan_state(), BluetoothScanState::Stopped);
        assert_eq!(central.start_unfiltered_scan(), Ok(()));
        assert_eq!(central.scan_state(), BluetoothScanState::Starting);
        assert_eq!(
            central.start_unfiltered_scan(),
            Err(BluetoothScanError::AlreadyActive)
        );
        assert_eq!(central.try_next_discovery(), Some(discovery));
        assert_eq!(central.take_dropped_discovery_count(), 3);
        central.stop_scan();
        assert_eq!(central.scan_state(), BluetoothScanState::Stopped);
    }
}
