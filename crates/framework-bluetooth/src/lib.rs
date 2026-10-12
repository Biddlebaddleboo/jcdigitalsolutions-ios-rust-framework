#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Bluetooth authorization, bounded discovery, central-connection, and GATT service-discovery contracts."]

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

/// A fixed-width opaque identifier assigned by a native Bluetooth implementation to a peer.
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

/// A normalized Bluetooth service UUID in canonical 128-bit UUID octet order.
///
/// Bluetooth 16-bit and 32-bit UUIDs are expanded with the Bluetooth Base UUID. This value
/// identifies a GATT service type; it is not a handle to a service instance or a characteristic.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BluetoothServiceUuid([u8; 16]);

impl BluetoothServiceUuid {
    /// Creates a service UUID from 16 octets in canonical UUID-string order.
    pub const fn from_uuid_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Returns the 16 service UUID octets in canonical UUID-string order.
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
    /// The backend has not yet reported radio readiness for this request.
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

/// The caller-visible state of one explicitly requested central connection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothConnectionState {
    /// No connection has been requested through this backend.
    Idle = 0,
    /// Native manager readiness or peer lookup is pending.
    Starting = 1,
    /// A requested connection is waiting for the central manager to become ready.
    WaitingForPower = 2,
    /// The backend accepted the native connection request and has not reported its outcome.
    Connecting = 3,
    /// The backend reported that the peer is connected.
    Connected = 4,
    /// The caller requested cancellation and the backend has not reported disconnection.
    Disconnecting = 5,
    /// The peer disconnected or a pending request was cancelled before native connection began.
    Disconnected = 6,
    /// The request ended with an error; inspect the queued failure event for its reason.
    Failed = 7,
}

/// A normalized failure or synchronous rejection from a central connection request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothConnectionError {
    /// Another connection request is active or awaiting native cancellation.
    AlreadyActive = 0,
    /// The backend could not resolve the caller-selected OS peer UUID.
    PeripheralUnavailable = 1,
    /// Bluetooth authorization does not permit a central connection.
    Unauthorized = 2,
    /// The current device or platform does not support the Bluetooth central role.
    Unsupported = 3,
    /// The manager state was unrecognized or a native callback could not be processed.
    Unavailable = 4,
    /// The native backend reported a connection or disconnect error.
    ConnectionFailed = 5,
}

/// A copied event for an explicitly requested central connection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BluetoothConnectionEvent {
    /// The backend observed a change in the request's lifecycle state.
    StateChanged {
        /// The caller-selected OS peer UUID.
        peripheral: BluetoothPeripheralId,
        /// The new backend-observed lifecycle state.
        state: BluetoothConnectionState,
    },
    /// The request failed; native error details are normalized and not exposed cross-platform.
    Failed {
        /// The caller-selected OS peer UUID.
        peripheral: BluetoothPeripheralId,
        /// The normalized failure reason.
        error: BluetoothConnectionError,
    },
}

/// A statically selected backend for one caller-selected Bluetooth central connection.
///
/// A request resolves the supplied OS-assigned peer UUID when the native manager is ready. It
/// returns before the native connection outcome is known; callers inspect state and poll copied
/// events. Backends may bound the event queue, drop the newest event on overflow, and report the
/// saturating drop count. Cancellation is nonblocking. The backend must keep any required native
/// peer object alive until the native terminal callback or backend teardown. A UUID is not a
/// guarantee that the operating system can retrieve that peer indefinitely.
pub trait BluetoothConnectionBackend: BluetoothCentralBackend {
    /// Explicitly requests a connection to this OS-assigned peer UUID.
    ///
    /// Returns [`BluetoothConnectionError::AlreadyActive`] while an earlier request remains
    /// active. Native requests finish at their terminal callback; pre-native failures and
    /// cancellations finish when reported. An accepted request may initialize a platform manager
    /// and cause an operating-system authorization prompt; the separate authorization query does
    /// not.
    fn connect_peripheral(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> Result<(), BluetoothConnectionError>;

    /// Nonblockingly cancels a pending or active connection; does nothing when idle or terminal.
    fn cancel_connection(&mut self);

    /// Returns the latest backend-observed connection lifecycle state.
    fn connection_state(&self) -> BluetoothConnectionState;

    /// Removes one already-delivered connection event, if one is queued.
    fn try_next_connection_event(&mut self) -> Option<BluetoothConnectionEvent>;

    /// Returns and resets the saturating count of connection events dropped by the bounded queue.
    fn take_dropped_connection_event_count(&mut self) -> u32;
}

/// The latest lifecycle state of one requested GATT service-discovery operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothServiceDiscoveryState {
    /// No service-discovery request has been made.
    Idle = 0,
    /// CoreBluetooth accepted a request and its asynchronous callback is pending.
    Discovering = 1,
    /// The backend delivered the full discovered-service list.
    Completed = 2,
    /// The request ended with an error or the connected peer disconnected.
    Failed = 3,
}

/// A synchronous rejection or normalized asynchronous GATT service-discovery failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum BluetoothServiceDiscoveryError {
    /// No caller-selected peripheral is currently connected.
    NotConnected = 0,
    /// An earlier service-discovery callback is still pending.
    AlreadyActive = 1,
    /// CoreBluetooth reported a service-discovery error.
    NativeFailure = 2,
    /// The selected peer disconnected before service discovery completed.
    Disconnected = 3,
    /// The backend could not safely convert or process a native result.
    Unavailable = 4,
}

/// A copied GATT service-discovery result or terminal event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BluetoothServiceDiscoveryEvent {
    /// One discovered service type, copied without retaining a native service object.
    ServiceDiscovered {
        /// The caller-selected OS peer UUID.
        peripheral: BluetoothPeripheralId,
        /// The normalized 128-bit service type UUID.
        service: BluetoothServiceUuid,
    },
    /// CoreBluetooth completed service discovery for the selected peer.
    Completed {
        /// The caller-selected OS peer UUID.
        peripheral: BluetoothPeripheralId,
    },
    /// Service discovery failed; native error details are not exposed cross-platform.
    Failed {
        /// The caller-selected OS peer UUID.
        peripheral: BluetoothPeripheralId,
        /// The normalized failure reason.
        error: BluetoothServiceDiscoveryError,
    },
}

/// A statically selected backend for explicit GATT service discovery on the active connection.
///
/// A request is accepted only while the caller-selected peripheral is connected, returns before
/// native results arrive, and discovers all of its GATT services. Results and one terminal event
/// are copied into a bounded queue for polling. Implementations may drop newest events on queue
/// overflow and report the saturating drop count. There is no discovery-only cancellation method;
/// cancelling or disconnecting the selected connection terminates the operation, and backend
/// teardown suppresses later callbacks. This contract does not read/write characteristics,
/// expose native service objects, or create a peripheral role.
pub trait BluetoothServiceDiscoveryBackend: BluetoothConnectionBackend {
    /// Explicitly requests discovery of all GATT services on the connected selected peer.
    fn discover_services(&mut self) -> Result<(), BluetoothServiceDiscoveryError>;

    /// Returns the latest service-discovery lifecycle state.
    fn service_discovery_state(&self) -> BluetoothServiceDiscoveryState;

    /// Removes one copied service-discovery event, if one is queued.
    fn try_next_service_discovery_event(&mut self) -> Option<BluetoothServiceDiscoveryEvent>;

    /// Returns and resets the saturating count of service-discovery events dropped by the bounded
    /// queue.
    fn take_dropped_service_discovery_event_count(&mut self) -> u32;
}

/// A thin facade over caller-owned central backend state.
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

impl<B: BluetoothConnectionBackend> BluetoothCentral<B> {
    /// Explicitly requests a connection to a caller-selected OS peer UUID.
    pub fn connect_peripheral(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> Result<(), BluetoothConnectionError> {
        self.backend.connect_peripheral(peripheral)
    }

    /// Nonblockingly cancels a pending or active connection.
    pub fn cancel_connection(&mut self) {
        self.backend.cancel_connection();
    }

    /// Returns the latest backend-observed connection lifecycle state.
    pub fn connection_state(&self) -> BluetoothConnectionState {
        self.backend.connection_state()
    }

    /// Removes one already-delivered connection event, if one is queued.
    pub fn try_next_connection_event(&mut self) -> Option<BluetoothConnectionEvent> {
        self.backend.try_next_connection_event()
    }

    /// Returns and resets the saturating count of connection events dropped by the bounded queue.
    pub fn take_dropped_connection_event_count(&mut self) -> u32 {
        self.backend.take_dropped_connection_event_count()
    }
}

impl<B: BluetoothServiceDiscoveryBackend> BluetoothCentral<B> {
    /// Explicitly requests GATT service discovery on the currently connected selected peer.
    pub fn discover_services(&mut self) -> Result<(), BluetoothServiceDiscoveryError> {
        self.backend.discover_services()
    }

    /// Returns the latest service-discovery lifecycle state.
    pub fn service_discovery_state(&self) -> BluetoothServiceDiscoveryState {
        self.backend.service_discovery_state()
    }

    /// Removes one copied service-discovery event, if one is queued.
    pub fn try_next_service_discovery_event(&mut self) -> Option<BluetoothServiceDiscoveryEvent> {
        self.backend.try_next_service_discovery_event()
    }

    /// Returns and resets the saturating count of service-discovery events dropped by the bounded
    /// queue.
    pub fn take_dropped_service_discovery_event_count(&mut self) -> u32 {
        self.backend.take_dropped_service_discovery_event_count()
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

    struct FakeConnectionCentral {
        state: BluetoothConnectionState,
        target: Option<BluetoothPeripheralId>,
        event: Option<BluetoothConnectionEvent>,
        dropped: u32,
    }

    impl BluetoothCentralBackend for FakeConnectionCentral {
        fn start_unfiltered_scan(&mut self) -> Result<(), BluetoothScanError> {
            Ok(())
        }

        fn stop_scan(&mut self) {}

        fn scan_state(&self) -> BluetoothScanState {
            BluetoothScanState::Stopped
        }

        fn try_next_discovery(&mut self) -> Option<BluetoothDiscovery> {
            None
        }

        fn take_dropped_discovery_count(&mut self) -> u32 {
            0
        }
    }

    impl BluetoothConnectionBackend for FakeConnectionCentral {
        fn connect_peripheral(
            &mut self,
            peripheral: BluetoothPeripheralId,
        ) -> Result<(), BluetoothConnectionError> {
            if self.target.is_some() {
                return Err(BluetoothConnectionError::AlreadyActive);
            }
            self.target = Some(peripheral);
            self.state = BluetoothConnectionState::Starting;
            self.event = Some(BluetoothConnectionEvent::StateChanged {
                peripheral,
                state: self.state,
            });
            Ok(())
        }

        fn cancel_connection(&mut self) {
            if let Some(peripheral) = self.target.take() {
                self.state = BluetoothConnectionState::Disconnected;
                self.event = Some(BluetoothConnectionEvent::StateChanged {
                    peripheral,
                    state: self.state,
                });
            }
        }

        fn connection_state(&self) -> BluetoothConnectionState {
            self.state
        }

        fn try_next_connection_event(&mut self) -> Option<BluetoothConnectionEvent> {
            self.event.take()
        }

        fn take_dropped_connection_event_count(&mut self) -> u32 {
            core::mem::take(&mut self.dropped)
        }
    }

    #[test]
    fn central_facade_exposes_explicit_connection_and_rejects_overlap() {
        let id = BluetoothPeripheralId::from_uuid_bytes([0x3C; 16]);
        let backend = FakeConnectionCentral {
            state: BluetoothConnectionState::Idle,
            target: None,
            event: None,
            dropped: 2,
        };
        let mut central = BluetoothCentral::new(backend);
        assert_eq!(central.connect_peripheral(id), Ok(()));
        assert_eq!(
            central.connection_state(),
            BluetoothConnectionState::Starting
        );
        assert_eq!(
            central.connect_peripheral(BluetoothPeripheralId::from_uuid_bytes([0x4D; 16])),
            Err(BluetoothConnectionError::AlreadyActive)
        );
        assert_eq!(
            central.try_next_connection_event(),
            Some(BluetoothConnectionEvent::StateChanged {
                peripheral: id,
                state: BluetoothConnectionState::Starting,
            })
        );
        central.cancel_connection();
        central.cancel_connection();
        assert_eq!(
            central.connection_state(),
            BluetoothConnectionState::Disconnected
        );
        assert_eq!(
            central.try_next_connection_event(),
            Some(BluetoothConnectionEvent::StateChanged {
                peripheral: id,
                state: BluetoothConnectionState::Disconnected,
            })
        );
        assert_eq!(central.take_dropped_connection_event_count(), 2);
    }
}
