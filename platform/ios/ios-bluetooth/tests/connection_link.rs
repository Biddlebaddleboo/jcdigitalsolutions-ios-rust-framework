#![cfg(target_os = "ios")]

use framework_bluetooth::{BluetoothCentral, BluetoothPeripheralId};
use ios_bluetooth::IosBluetoothCentralBackend;
use objc2::MainThreadMarker;

#[test]
fn explicit_connection_surface_links_without_starting_core_bluetooth() {
    if std::env::var_os("IOS_BLUETOOTH_RUN_CONNECTION_RUNTIME_PROBE").is_none() {
        return;
    }
    let Some(marker) = MainThreadMarker::new() else {
        return;
    };
    let mut central = BluetoothCentral::new(IosBluetoothCentralBackend::new(marker));
    let id = BluetoothPeripheralId::from_uuid_bytes([0xA5; 16]);
    let _ = central.connect_peripheral(id);
    central.cancel_connection();
    let _ = central.connection_state();
    let _ = central.try_next_connection_event();
    let _ = central.take_dropped_connection_event_count();
}
