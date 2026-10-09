# iOS Bluetooth central discovery

`IosBluetoothCentralBackend` implements the portable [Bluetooth central contract](../capabilities/bluetooth.md)
with CoreBluetooth. It is independent of the stateless, non-prompting [`IosBluetoothBackend`](bluetooth.md)
authorization query.

```rust,ignore
use framework_bluetooth::{BluetoothCentral, BluetoothCentralBackend};
use ios_bluetooth::IosBluetoothCentralBackend;
use objc2::MainThreadMarker;

fn make_central(marker: MainThreadMarker) -> BluetoothCentral<IosBluetoothCentralBackend> {
    BluetoothCentral::new(IosBluetoothCentralBackend::new(marker))
}
```

Construction creates no `CBCentralManager` and requests no permission. The caller must explicitly
call `start_unfiltered_scan`. That call lazily creates `CBCentralManager` and can trigger the OS
Bluetooth authorization prompt. It returns before CoreBluetooth reports readiness or calls the
discovery delegate. `BluetoothScanState::Starting` and `BluetoothScanState::WaitingForPower` are
not scan success. `BluetoothScanState::Scanning` means the adapter issued the scan command after a
`PoweredOn` state callback; it is not proof that a radio is active or a peer is in range.

CoreBluetooth sends callbacks to the main queue because the adapter passes a nil dispatch queue to
`initWithDelegate:queue:`. Create, use, and drop the backend on the same main thread. The backend
retains its delegate because `CBCentralManager.delegate` is weak. Drop stops the scan and clears the
native delegate. A host app must keep the facade alive and must let its main run loop process native
callbacks. On a later main-thread app callback, drain ready values with
`BluetoothCentral::try_next_discovery`; no executor, future, user callback, or hidden polling loop is
provided.

## Results, capacity, and stop

The adapter passes nil services and nil options to CoreBluetooth. Each callback copies the peer's
`CBPeer.identifier` UUID into 16 bytes in canonical UUID-string octet order and copies the RSSI to
`i32` dBm. CoreBluetooth reserves RSSI 127 for unavailable; values outside the portable `i32`
range are also reported as `None`. The adapter does not retain or expose `CBPeripheral`,
advertisement data, service data, or names.

The event queue holds 32 values. When full, the newest callback is dropped; older queued values
stay in FIFO order. `take_dropped_discovery_count` returns and resets the saturating `u32` count.
`stop_scan` clears the active/pending scan request, stops native discovery, and preserves values
already queued for the caller to drain. Later native discovery callbacks are ignored. On power loss,
CoreBluetooth stops scanning; the adapter reports `WaitingForPower` and reissues the request after a
later powered-on callback if the caller has not stopped it.

The portable authorization query can report `Unknown`, `NotDetermined`, `Restricted`, `Denied`, or
`Allowed` before a scan. It is only a snapshot. The explicit scan can still prompt, return
`Unauthorized`, wait for power, or fail to discover peers. The scan lifecycle state does not
distinguish denied from restricted; query authorization separately for that detail.

## Scope and host configuration

The CoreBluetooth manager and scan method are available from iOS 5.0 per the installed Xcode 26.6 /
iPhoneOS 26.5 SDK headers. The adapter also reads `CBPeer.identifier`: the installed `CBPeer.h`
marks the `CBPeer` class available from iOS 8.0 and the property from iOS 7.0, so the effective
identity/discovery adapter floor is iOS 8.0. `NSUUID` is available from iOS 6.0. The separate
`CBManager.authorization` static query has an iOS 13.1 floor. The adapter does not set a minimum
deployment target and does not claim a newer Xcode baseline.

For apps linked on or after iOS 13, Apple requires the exact `NSBluetoothAlwaysUsageDescription`
key in the host app's `Info.plist`. For apps whose deployment target is earlier than iOS 13, Apple
documents both `NSBluetoothAlwaysUsageDescription` and deprecated
`NSBluetoothPeripheralUsageDescription`. The host app owns these values and any user-facing
rationale.

This API intentionally scans with nil service UUIDs, which returns all discovered peripherals.
Apple recommends a service filter; its documented `bluetooth-central` background mode requires at
least one service UUID. This API has no service filter, background-mode setup, restoration,
connection, or peripheral data operation. It also does not implement the distinct iOS 26 Live
Activity background path. It provides no background delivery guarantee. The host app owns lifecycle
policy and must stop scans when appropriate.

## Dependency and validation limits

The iOS crate uses `objc2-core-bluetooth` 0.3.2 with default features disabled and only
`CBManager`, `CBCentralManager`, `CBPeer`, `CBPeripheral`, `CBUUID`, and `dispatch2` enabled. It uses
`objc2-foundation` only for `NSString`, `NSObject`, `NSUUID`, and `NSValue`. The portable contract
exposes no Apple binding type.

The device/simulator check and Clippy gates compile the bindings only. They do not instantiate a
manager, request permission, show UI, scan a radio, discover a device, validate RSSI, prove queue
callback timing/drop teardown, or test background behavior.

References: [CBCentralManager scan](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/scanforperipherals%28withservices%3Aoptions%3A%29),
[CBCentralManagerDelegate discovery](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate/centralmanager%28_%3Adiddiscover%3Aadvertisementdata%3Arssi%3A%29),
[CBPeer identifier](https://developer.apple.com/documentation/corebluetooth/cbpeer/identifier),
[Core Bluetooth framework](https://developer.apple.com/documentation/corebluetooth/), and
[NSBluetoothPeripheralUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothperipheralusagedescription).
