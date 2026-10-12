# iOS Bluetooth central discovery and connection

`IosBluetoothCentralBackend` implements the portable [Bluetooth central contract](../capabilities/bluetooth.md)
with CoreBluetooth, including explicit unfiltered foreground discovery, one caller-selected
connection, and connected-peer GATT service discovery. It is independent of the stateless,
non-prompting [`IosBluetoothBackend`](bluetooth.md) authorization query.

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
CoreBluetooth reported a connection; the B570 connection slice did not include service or
characteristic discovery, GATT read/write, advertisement data, peripheral role, background mode, or
restoration API.

## GATT service discovery

After the selected peer reaches `BluetoothConnectionState::Connected`, call
`BluetoothCentral::discover_services` explicitly. The backend rechecks the selected peer UUID and
native `CBPeripheralState::Connected`; stale or disconnected state returns
`BluetoothServiceDiscoveryError::NotConnected`. Only one request can be active. The request uses
public `CBPeripheral::discoverServices` with nil filters, so CoreBluetooth discovers all services
and returns asynchronously on the main queue.

```rust,ignore
use framework_bluetooth::BluetoothServiceDiscoveryEvent;

central.discover_services().expect("selected peripheral is connected");
while let Some(event) = central.try_next_service_discovery_event() {
    match event {
        BluetoothServiceDiscoveryEvent::ServiceDiscovered { peripheral, service } => {
            let uuid_bytes: &[u8; 16] = service.as_uuid_bytes();
            let _ = (peripheral, uuid_bytes);
        }
        BluetoothServiceDiscoveryEvent::Completed { peripheral } => { let _ = peripheral; }
        BluetoothServiceDiscoveryEvent::Failed { peripheral, error } => { let _ = (peripheral, error); }
    }
}
let _ = central.service_discovery_state();
let dropped = central.take_dropped_service_discovery_event_count();
```

`BluetoothServiceUuid` is a fixed 16-byte value in canonical UUID-string octet order. Native
16-bit and 32-bit UUID forms expand using the Bluetooth Base UUID; 128-bit values preserve their
bytes. Each event copies only the peripheral UUID and service UUID; no `CBService` is retained or
exposed. The adapter queues up to 32 service results and terminal events, drops newest events on
overflow, and reports/reset a saturating `u32` count. Query `service_discovery_state` to distinguish
`Discovering`, `Completed`, or `Failed` even if queue overflow drops a terminal event.

Native errors map to `NativeFailure`; a disconnect during discovery maps to `Disconnected`; malformed
UUID data or a contained callback panic maps to `Unavailable`. Native error text is not exposed.
CoreBluetooth provides no cancellation operation for `discoverServices:`; cancelling the selected
connection terminates active service discovery with `Disconnected` immediately, clears the weak
peripheral delegate, and requests connection cancellation. Backend drop clears the weak peripheral
and central delegates before release. The B592 service-discovery slice alone adds no characteristic
discovery, GATT read/write, peripheral role, background restoration, or automatic reconnect.

## Read one connected-peer characteristic value

After successful service discovery, `BluetoothCentral::read_characteristic` selects one service and
one characteristic by UUID. The iOS backend verifies the same caller-selected peer remains connected,
requires the unique service and characteristic matches, checks the characteristic's public Read
property, then calls `CBPeripheral::readValueForCharacteristic`. Service/characteristic discovery
and value delivery are asynchronous CoreBluetooth callbacks on the main queue.

The completed B592 discovery record is scoped to that exact peer identity. A new discovery request,
connection retry, cancellation, or disconnect clears it; a cached nonnil service array is not proof
that the current connection completed discovery. Reconnect before reading requires fresh B592 service
discovery for that peer.

```rust,ignore
use framework_bluetooth::{BluetoothCharacteristicReadEvent, BluetoothCharacteristicUuid};

let service = BluetoothServiceUuid::from_uuid_bytes([0x18; 16]);
let characteristic = BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16]);
central.read_characteristic(service, characteristic)?;
while let Some(event) = central.try_next_characteristic_read_event() {
    match event {
        BluetoothCharacteristicReadEvent::Completed { value, .. } => {
            let bytes: &[u8] = value.as_bytes();
            let _ = bytes;
        }
        BluetoothCharacteristicReadEvent::Failed { error, .. } => { let _ = error; }
    }
}
let _ = central.characteristic_read_state();
let dropped = central.take_dropped_characteristic_read_event_count();
```

The initial request rejects not-connected or unsupported state, incomplete B592 service discovery,
and overlapping service/read operations. It also rejects absent or duplicate matching service UUIDs
before native characteristic discovery. Characteristic absence, duplicate UUIDs, no Read property,
CoreBluetooth errors, malformed values, disconnect, and contained callback panic produce one copied
terminal failure. A successful value is copied into a fixed 512-byte inline buffer; an empty value is
valid and an oversize native value fails without truncation. No native handle or borrowed `NSData`
buffer crosses the portable boundary.

The iOS adapter retains the selected native service and characteristic only while the read is active,
and queues at most four terminal read events, dropping newest on overflow. `cancel_connection` ends
an active read as `Disconnected` and clears the weak peripheral delegate before native cancellation;
backend drop clears delegates and retained objects, suppressing later callbacks. CoreBluetooth has no
per-read cancellation token or timeout here. Reads do not write, subscribe to notifications, discover
descriptors, or add peripheral-role behavior. Compile and link checks do not prove radio access,
permission, callback delivery, or live value behavior; no Simulator or physical-device runtime test
is claimed.

## Write one connected-peer characteristic value

`BluetoothCentral::write_characteristic` accepts a borrowed payload, copies it before returning, and
requests one `CBCharacteristicWriteWithResponse` operation for a unique service/characteristic UUID
pair. It requires fresh B592 service discovery for the exact connected peer and the characteristic's
public Write property. The backend rejects payloads above 512 bytes or CoreBluetooth's current
write-with-response limit; on iOS 8, before that limit query exists, it conservatively caps a write
at 20 bytes. Other active service-discovery/read/write operations are rejected synchronously.

```rust,ignore
use framework_bluetooth::BluetoothCharacteristicWriteEvent;

central.write_characteristic(service, characteristic, payload)?;
while let Some(event) = central.try_next_characteristic_write_event() {
    match event {
        BluetoothCharacteristicWriteEvent::Completed { .. } => { /* CoreBluetooth reported success */ }
        BluetoothCharacteristicWriteEvent::Failed { error, .. } => { let _ = error; }
    }
}
let _ = central.characteristic_write_state();
let dropped = central.take_dropped_characteristic_write_event_count();
```

The backend retains its copied `NSData` and selected native objects through the terminal
`didWriteValueForCharacteristic:error:` callback, then queues one terminal event in a separate
four-entry FIFO. A disconnect or `cancel_connection` during native characteristic discovery or a
write reports `Disconnected`; it retains the native objects until the disconnect callback or
backend drop. Because CoreBluetooth characteristic-discovery and write callbacks carry no operation
token, that backend instance rejects later writes with `Unavailable` rather than risk attributing a
delayed callback to a later write. Drop and recreate
the backend before issuing another write. This completion is CoreBluetooth's
write-with-response result, not proof of an application-level effect on the peripheral. No
write-without-response, notification subscription, descriptor operation, peripheral role,
Simulator run, or physical-device behavior is claimed.

## Subscribe to one connected-peer characteristic

`BluetoothCentral::enable_characteristic_notifications` starts one opt-in notification or
indication subscription after fresh B592 service discovery. The selected service and characteristic
UUIDs must be unique, and the characteristic must expose the public `Notify` or `Indicate` property.
The backend reports `Enabling`, `Enabled`, `Disabling`, `Disabled`, or `Failed`; native enable and
disable confirmation arrive asynchronously through CoreBluetooth. If both properties are present,
CoreBluetooth selects notifications; this API does not expose transport preference.

```rust,ignore
use framework_bluetooth::{BluetoothCharacteristicNotificationState};

central.enable_characteristic_notifications(service, characteristic)?;
while let Some(event) = central.try_next_characteristic_notification_event() {
    match event.result {
        Ok(value) => { let bytes = value.as_bytes(); let _ = bytes; }
        Err(error) => { let _ = error; }
    }
}
if central.characteristic_notification_state() == BluetoothCharacteristicNotificationState::Enabled {
    central.disable_characteristic_notifications()?;
}
let dropped = central.take_dropped_characteristic_notification_event_count();
```

Each copied value is capped at 512 bytes and queued in a separate four-event FIFO; overflow drops
the newest event and saturates the dropped-event count. Reads are rejected while the subscription is
active because CoreBluetooth uses the same untagged value callback for reads and notification values.
After disable confirmation, disconnect, or an interrupted transition, the backend keeps the native
characteristic as a callback tombstone and rejects later reads or subscriptions with `Unavailable`;
recreate the backend before starting those operations again. This prevents a late value callback from
being attributed to a later read or subscription. Write-with-response uses a distinct callback and
remains available after enable confirmation. Cancellation and backend teardown stop the subscription
and clear the peripheral delegate. No descriptor or live central runtime behavior is claimed.

## Local read-only peripheral service (B600)

`IosBluetoothPeripheralBackend` implements the portable `BluetoothPeripheralBackend` with public
CoreBluetooth APIs. It creates `CBPeripheralManager` only after the caller explicitly starts one
service, publishes that service, and advertises its UUID only after service publication succeeds.
Each configured characteristic is a CoreBluetooth `Read`/`Readable` attribute with a copied static
value; CoreBluetooth serves that value without a dynamic read-request callback. The portable builder
bounds the service to one through eight unique characteristic UUIDs and values to 512 bytes.

Construct, use, and drop the backend on the main thread. Its manager uses the main queue, and all
operations and lifecycle callbacks remain on that thread. The caller owns the service definition;
start may prompt for Bluetooth access, and `BluetoothPeripheralState` distinguishes startup, power
wait, service publication, advertising, unauthorized, unsupported, native failure, and unavailable
callback states. A powered-off manager clears its local GATT database; an active request republishes
the service when power returns. `stop_advertising` stops this manager's advertisement and removes
only its owned service when CoreBluetooth permits it; backend drop also stops and releases the weak
delegate. The host app owns `NSBluetoothAlwaysUsageDescription` and, for pre-iOS 13 deployment
targets, Apple's documented deprecated `NSBluetoothPeripheralUsageDescription`. Background
operation is not promised; apps that need it must configure Apple's `bluetooth-peripheral` background
mode and still account for system scheduling and advertising limits.

The registered `ios-bluetooth` profile compiles the peripheral link probe for device and Simulator
and runs host policy/contract regressions. Neither compilation nor link inspection proves permission
prompts, live advertisement, remote GATT reads, foreground lifecycle behavior, or background runtime.
No Simulator or physical-device peripheral test has been performed.

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
enables `CBManager`, `CBCentralManager`, `CBPeer`, `CBPeripheral`, `CBAttribute`, `CBService`,
`CBUUID`, `CBAdvertisementData`, `CBPeripheralManager`, `CBPeripheralManagerConstants`, and
`dispatch2`. It
disables `objc2-foundation` defaults and directly enables only `NSArray`, `NSString`, `NSObject`,
`NSUUID`, and `NSValue`; the selected generated CoreBluetooth APIs also activate their required
Foundation and `bitflags` feature dependencies. The portable contract exposes no Apple binding type.

The package script checks and Clippy gates compile device and Simulator targets; its link probe
cross-links the test executable without running it. The optional peripheral runtime probe is gated by
`IOS_BLUETOOTH_RUN_PERIPHERAL_RUNTIME_PROBE` and was not run. These checks do not request
permission, show UI, scan a radio, discover a device/service, validate RSSI, connect to a peer,
advertise a service, validate remote reads, prove callback timing/drop teardown on-device, or test
background behavior.

References: [CBCentralManager scan](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/scanforperipherals%28withservices%3Aoptions%3A%29),
[retrieve peripherals](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/retrieveperipherals%28withidentifiers%3A%29),
[connect](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/connect%28_%3Aoptions%3A%29),
[cancel connection](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/cancelperipheralconnection%28_%3A%29),
[connection delegate callbacks](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate),
[CBCentralManagerDelegate discovery](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate/centralmanager%28_%3Adiddiscover%3Aadvertisementdata%3Arssi%3A%29),
[CBPeer identifier](https://developer.apple.com/documentation/corebluetooth/cbpeer/identifier),
[CBPeripheral discover services](https://developer.apple.com/documentation/corebluetooth/cbperipheral/discoverservices%28_%3A%29),
[CBPeripheralDelegate service callback](https://developer.apple.com/documentation/corebluetooth/cbperipheraldelegate/peripheral%28_%3Adiddiscoverservices%3A%29),
[Core Bluetooth framework](https://developer.apple.com/documentation/corebluetooth/), and
[NSBluetoothPeripheralUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothperipheralusagedescription).
