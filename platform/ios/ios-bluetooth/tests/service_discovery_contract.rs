use std::collections::VecDeque;

use framework_bluetooth::{
    BluetoothCentral, BluetoothCentralBackend, BluetoothConnectionBackend,
    BluetoothConnectionError, BluetoothConnectionEvent, BluetoothConnectionState,
    BluetoothDiscovery, BluetoothPeripheralId, BluetoothScanError, BluetoothScanState,
    BluetoothServiceDiscoveryBackend, BluetoothServiceDiscoveryError,
    BluetoothServiceDiscoveryEvent, BluetoothServiceDiscoveryState, BluetoothServiceUuid,
};

struct FakeCentral {
    connection: BluetoothConnectionState,
    service_state: BluetoothServiceDiscoveryState,
    events: VecDeque<BluetoothServiceDiscoveryEvent>,
    dropped: u32,
    finish_immediately: bool,
}

impl BluetoothCentralBackend for FakeCentral {
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

impl BluetoothConnectionBackend for FakeCentral {
    fn connect_peripheral(
        &mut self,
        _peripheral: BluetoothPeripheralId,
    ) -> Result<(), BluetoothConnectionError> {
        self.connection = BluetoothConnectionState::Connected;
        Ok(())
    }

    fn cancel_connection(&mut self) {
        self.connection = BluetoothConnectionState::Disconnected;
    }

    fn connection_state(&self) -> BluetoothConnectionState {
        self.connection
    }

    fn try_next_connection_event(&mut self) -> Option<BluetoothConnectionEvent> {
        None
    }

    fn take_dropped_connection_event_count(&mut self) -> u32 {
        0
    }
}

impl BluetoothServiceDiscoveryBackend for FakeCentral {
    fn discover_services(&mut self) -> Result<(), BluetoothServiceDiscoveryError> {
        if self.connection != BluetoothConnectionState::Connected {
            return Err(BluetoothServiceDiscoveryError::NotConnected);
        }
        if self.service_state == BluetoothServiceDiscoveryState::Discovering {
            return Err(BluetoothServiceDiscoveryError::AlreadyActive);
        }
        self.service_state = BluetoothServiceDiscoveryState::Discovering;
        self.events
            .push_back(BluetoothServiceDiscoveryEvent::ServiceDiscovered {
                peripheral: BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]),
                service: BluetoothServiceUuid::from_uuid_bytes([
                    0, 0, 0x18, 0x0D, 0, 0, 0x10, 0, 0x80, 0, 0, 0x80, 0x5F, 0x9B, 0x34, 0xFB,
                ]),
            });
        if self.finish_immediately {
            self.service_state = BluetoothServiceDiscoveryState::Completed;
            self.events
                .push_back(BluetoothServiceDiscoveryEvent::Completed {
                    peripheral: BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]),
                });
        }
        Ok(())
    }

    fn service_discovery_state(&self) -> BluetoothServiceDiscoveryState {
        self.service_state
    }

    fn try_next_service_discovery_event(&mut self) -> Option<BluetoothServiceDiscoveryEvent> {
        self.events.pop_front()
    }

    fn take_dropped_service_discovery_event_count(&mut self) -> u32 {
        std::mem::take(&mut self.dropped)
    }
}

fn fake(finish_immediately: bool) -> FakeCentral {
    FakeCentral {
        connection: BluetoothConnectionState::Idle,
        service_state: BluetoothServiceDiscoveryState::Idle,
        events: VecDeque::new(),
        dropped: 2,
        finish_immediately,
    }
}

fn assert_service_backend_uses_connection_contract<B: BluetoothServiceDiscoveryBackend>(
    central: &BluetoothCentral<B>,
) {
    let _ = central.connection_state();
}

#[test]
fn service_uuid_is_fixed_width_and_preserves_canonical_octet_order() {
    let uuid = BluetoothServiceUuid::from_uuid_bytes([
        0x00, 0x00, 0x18, 0x0D, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34,
        0xFB,
    ]);
    assert_eq!(std::mem::size_of::<BluetoothServiceUuid>(), 16);
    assert_eq!(uuid.as_uuid_bytes()[2..4], [0x18, 0x0D]);
}

#[test]
fn facade_requires_connection_and_polls_service_results_and_completion() {
    let peer = BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]);
    let mut central = BluetoothCentral::new(fake(true));
    assert_service_backend_uses_connection_contract(&central);
    assert_eq!(
        central.discover_services(),
        Err(BluetoothServiceDiscoveryError::NotConnected)
    );
    assert_eq!(central.connect_peripheral(peer), Ok(()));
    assert_eq!(central.discover_services(), Ok(()));
    assert_eq!(
        central.service_discovery_state(),
        BluetoothServiceDiscoveryState::Completed
    );
    assert!(matches!(
        central.try_next_service_discovery_event(),
        Some(BluetoothServiceDiscoveryEvent::ServiceDiscovered { peripheral, service })
            if peripheral == peer && service.as_uuid_bytes()[2..4] == [0x18, 0x0D]
    ));
    assert_eq!(
        central.try_next_service_discovery_event(),
        Some(BluetoothServiceDiscoveryEvent::Completed { peripheral: peer })
    );
    assert_eq!(central.take_dropped_service_discovery_event_count(), 2);
    assert_eq!(central.take_dropped_service_discovery_event_count(), 0);
}

#[test]
fn facade_rejects_overlapping_service_discovery_requests() {
    let peer = BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]);
    let mut central = BluetoothCentral::new(fake(false));
    central.connect_peripheral(peer).unwrap();
    assert_eq!(central.discover_services(), Ok(()));
    assert_eq!(
        central.discover_services(),
        Err(BluetoothServiceDiscoveryError::AlreadyActive)
    );
    assert_eq!(
        central.service_discovery_state(),
        BluetoothServiceDiscoveryState::Discovering
    );
}
