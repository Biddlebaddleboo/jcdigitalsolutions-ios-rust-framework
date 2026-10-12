#![cfg(target_os = "ios")]

use framework_bluetooth::{
    BluetoothCentral, BluetoothCharacteristicNotificationState, BluetoothCharacteristicReadEvent,
    BluetoothCharacteristicReadState, BluetoothCharacteristicUuid,
    BluetoothCharacteristicWriteState, BluetoothPeripheralId, BluetoothServiceDiscoveryState,
    BluetoothServiceUuid,
};
use ios_bluetooth::IosBluetoothCentralBackend;
use objc2::MainThreadMarker;

#[test]
fn explicit_connection_and_gatt_surface_links_without_starting_core_bluetooth() {
    if std::env::var_os("IOS_BLUETOOTH_RUN_CONNECTION_RUNTIME_PROBE").is_none() {
        return;
    }
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let mut central = BluetoothCentral::new(IosBluetoothCentralBackend::new(marker));
    let _ = central.discover_services();
    assert_eq!(
        central.service_discovery_state(),
        BluetoothServiceDiscoveryState::Idle
    );
    let _ = central.try_next_service_discovery_event();
    let _ = central.take_dropped_service_discovery_event_count();
    assert_eq!(
        central.characteristic_read_state(),
        BluetoothCharacteristicReadState::Idle
    );
    let service = BluetoothServiceUuid::from_uuid_bytes([0x18; 16]);
    let characteristic = BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16]);
    let _ = central.read_characteristic(service, characteristic);
    while let Some(event) = central.try_next_characteristic_read_event() {
        if let BluetoothCharacteristicReadEvent::Completed { value, .. } = event {
            let _ = value.as_bytes();
        }
    }
    let _ = central.take_dropped_characteristic_read_event_count();
    assert_eq!(
        central.characteristic_write_state(),
        BluetoothCharacteristicWriteState::Idle
    );
    let _ = central.write_characteristic(service, characteristic, &[0xA5; 20]);
    let _ = central.try_next_characteristic_write_event();
    let _ = central.take_dropped_characteristic_write_event_count();
    let _ = central.enable_characteristic_notifications(service, characteristic);
    assert_eq!(
        central.characteristic_notification_state(),
        BluetoothCharacteristicNotificationState::Disabled
    );
    let _ = central.last_characteristic_notification_error();
    let _ = central.try_next_characteristic_notification_event();
    let _ = central.take_dropped_characteristic_notification_event_count();
    let _ = central.disable_characteristic_notifications();
    let id = BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]);
    let _ = central.connect_peripheral(id);
    central.cancel_connection();
    let _ = central.connection_state();
    let _ = central.try_next_connection_event();
    let _ = central.take_dropped_connection_event_count();
}
