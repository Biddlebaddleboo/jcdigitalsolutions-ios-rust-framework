# iOS Bluetooth central discovery and connection

`IosBluetoothCentralBackend` implements the portable [Bluetooth central contract](../capabilities/bluetooth.md)
with CoreBluetooth, including explicit unfiltered foreground discovery and one caller-selected
connection. It is independent of the stateless, non-prompting [`IosBluetoothBackend`](bluetooth.md)
authorization query.

```rust,ignore
use framework_bluetooth::{BluetoothCentral, BluetoothPeripheralId};
use ios_bluetooth::IosBluetoothCentralBackend;
use objc2::MainThreadMarker;

fn connect_known_peer(
    marker: MainThreadMarker,
    peer: BluetoothPeripheralId,
) -> BluetoothCentral<IosBluetoothCentralBackend> {
    let mut central = BluetoothCentral::new(IosBluetoothCentralBackend::new(marker));
    central.connect_peripheral(peer).expect("no other request is active");
    central
}
```

Construction creates no `CBCentralManager` and requests no permission. The caller must explicitly
call `start_unfiltered_scan` or `connect_peripheral`. Either operation lazily creates the manager
and can trigger the OS Bluetooth authorization prompt. Each returns before CoreBluetooth reports
readiness or a connection outcome. `BluetoothScanState::Starting` and
`BluetoothScanState::WaitingForPower` are not scan success. `BluetoothScanState::Scanning` means
the adapter issued the scan command after a `PoweredOn` state callback; it is not proof that a
radio is active or a peer is in range.

CoreBluetooth sends callbacks to the main queue because the adapter passes a nil dispatch queue to
`initWithDelegate:queue:`. Create, use, and drop the backend on the same main thread. The backend
retains its delegate because `CBCentralManager.delegate` is weak. Drop stops the scan, cancels an
active connection, and clears the native delegate. A host app must keep the facade alive and must let its main run loop process native
callbacks. On a later main-thread app callback, drain ready values with
`BluetoothCentral::try_next_discovery` or `BluetoothCentral::try_next_connection_event`; no
executor, future, user callback, or hidden polling loop is provided.

## Results, capacity, and stop

The adapter passes nil services and nil options to CoreBluetooth. Each callback copies the peer's
`CBPeer.identifier` UUID into 16 bytes in canonical UUID-string octet order and copies the RSSI to
`i32` dBm. CoreBluetooth reserves RSSI 127 for unavailable; values outside the portable `i32`
range are also reported as `None`. Discovery does not retain or expose `CBPeripheral`, advertisement
data, service data, or names.

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

## Caller-selected connection

Pass a `BluetoothPeripheralId` obtained from an earlier discovery or another supported source to
`BluetoothCentral::connect_peripheral`. The backend uses the public
`CBCentralManager::retrievePeripheralsWithIdentifiers` API with exactly that UUID, verifies the
returned peer identifier, and requests a connection with nil options. CoreBluetooth may return no
match; in that case the backend emits `BluetoothConnectionEvent::Failed` with
`BluetoothConnectionError::PeripheralUnavailable`. The UUID is an OS-assigned peer identifier, not
a hardware address, and does not guarantee indefinite retrieval.

The backend permits one active request. It reports `Starting` while manager readiness/lookup is
pending, `WaitingForPower` while the central manager is powered off, `Connecting` after issuing the
native request, `Connected` only after `centralManager:didConnectPeripheral:`, and `Disconnecting`
after explicit cancellation of a native request. A manager authorization/role failure or native
connection error emits a normalized failure event and leaves state `Failed`; a normal or
caller-cancelled terminal callback leaves state `Disconnected`. A second connect request returns
`AlreadyActive` while a request remains active; pre-native failure or cancellation is terminal when
reported. CoreBluetooth connection attempts have no timeout; this adapter adds no timeout or
automatic retry.

`cancel_connection` is idempotent and nonblocking. If called before peer lookup starts a native
request, it ends the request immediately. After native connect starts, it calls
`cancelPeripheralConnection` and retains the peer object until CoreBluetooth reports failure or
disconnection. A late `didConnect` after cancellation is not exposed as `Connected`; cancellation
is issued again and the request remains `Disconnecting` until a terminal callback. Callbacks that do
not match the active UUID are ignored, as are callbacks received while no request is active. Native
callbacks carry a peer identifier, not a request token, so a callback for an older attempt cannot be
distinguished from a callback for a newer attempt to the same UUID. On backend drop,
the adapter cancels an active connection, stops discovery, clears the weak native delegate, then
releases its retained peer and delegate on the supplied main thread.

Connection events use a separate 16-entry FIFO. When full, the newest event is dropped and the
saturating `u32` counter is observable through `take_dropped_connection_event_count`. The current
state remains queryable, but a dropped failure event can lose its normalized reason. Native
`NSError` details do not cross the portable contract. A `Connected` state only proves that
CoreBluetooth reported a connection; this workstream exposes no peripheral delegate, service or
characteristic discovery, GATT read/write, advertisement data, peripheral role, background mode, or
restoration API.

## Scope and host configuration

The CoreBluetooth manager and scan method are available from iOS 5.0 per the installed Xcode 26.6 /
iPhoneOS 26.5 SDK headers. The adapter also reads `CBPeer.identifier`: the installed `CBPeer.h`
marks the `CBPeer` class available from iOS 8.0 and the property from iOS 7.0, so the effective
identity/discovery/connection adapter floor is iOS 8.0. `retrievePeripheralsWithIdentifiers:` is
available from iOS 7.0 and `NSUUID` is available from iOS 6.0. The separate
`CBManager.authorization` static query has an iOS 13.1 floor. The adapter does not set a minimum
deployment target and does not claim a newer Xcode baseline.

For apps linked on or after iOS 13, Apple requires the exact `NSBluetoothAlwaysUsageDescription`
key in the host app's `Info.plist`. For apps whose deployment target is earlier than iOS 13, Apple
documents both `NSBluetoothAlwaysUsageDescription` and deprecated
`NSBluetoothPeripheralUsageDescription`. The host app owns these values and any user-facing
rationale.

The discovery API intentionally scans with nil service UUIDs, which returns all discovered peripherals.
Apple recommends a service filter; its documented `bluetooth-central` background mode requires at
least one service UUID. The discovery API has no service filter. Neither discovery nor connection
configures background execution, manager restoration, or the distinct iOS 26 Live Activity
background path. The adapter provides no background delivery or reconnect guarantee. The host app
owns lifecycle policy and must stop scans and cancel connections when appropriate.

## Dependency and validation limits

The iOS crate uses `objc2-core-bluetooth` 0.3.2 with default features disabled and explicitly
enables `CBManager`, `CBCentralManager`, `CBPeer`, `CBPeripheral`, `CBUUID`, and `dispatch2`. It
disables `objc2-foundation` defaults and directly enables only `NSArray`, `NSString`, `NSObject`,
`NSUUID`, and `NSValue`; the selected generated CoreBluetooth APIs also activate their required
Foundation and `bitflags` feature dependencies. The portable contract exposes no Apple binding type.

The package script checks and Clippy gates compile device and Simulator targets; its connection
link probes cross-link the test executable without running it. They do not instantiate a manager,
request permission, show UI, scan a radio, discover a device, validate RSSI, connect to a peer,
prove callback timing/drop teardown on-device, or test background behavior.

References: [CBCentralManager scan](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/scanforperipherals%28withservices%3Aoptions%3A%29),
[retrieve peripherals](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/retrieveperipherals%28withidentifiers%3A%29),
[connect](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/connect%28_%3Aoptions%3A%29),
[cancel connection](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/cancelperipheralconnection%28_%3A%29),
[connection delegate callbacks](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate),
[CBCentralManagerDelegate discovery](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate/centralmanager%28_%3Adiddiscover%3Aadvertisementdata%3Arssi%3A%29),
[CBPeer identifier](https://developer.apple.com/documentation/corebluetooth/cbpeer/identifier),
[Core Bluetooth framework](https://developer.apple.com/documentation/corebluetooth/), and
[NSBluetoothPeripheralUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothperipheralusagedescription).
