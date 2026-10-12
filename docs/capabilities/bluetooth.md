# Portable Bluetooth

`framework-bluetooth` defines portable authorization, central-discovery, explicit central-
connection, connected-peer GATT service-discovery, characteristic-read, and write-with-response
contracts. Platform managers and peripheral objects remain private to native backends. The
portable API has no advertisement, peripheral-role, beacon, write-without-response,
characteristic-notification, or radio-control contract.

## Authorization states

`BluetoothAuthorization` distinguishes Unknown, NotDetermined, Restricted, Denied, and Allowed.
`Allowed` reports that the backend's platform authorization status permits Bluetooth use. It does
not say that the radio is powered on, that a Bluetooth role is supported, or that a later scan or
connection will succeed. A backend maps any unrecognized or unclassifiable native authorization
state to `Unknown`. A status query is a snapshot and does not request permission.

## Backend and scope

`Bluetooth<B>` owns a backend supplied by its caller and uses static dispatch. The query does not
require a global registry, allocation, an executor, or hidden initialization. A backend must not
prompt or begin Bluetooth use while answering the query. This contract does not imply that a
Bluetooth permission request exists; no request operation is represented here.

## Central discovery

`BluetoothCentral<B>` owns a caller-supplied `BluetoothCentralBackend`. Its explicit
`start_unfiltered_scan` and `stop_scan` methods control one scan request. Start returns before the
backend is ready or any results arrive. Later `BluetoothDiscovery` values contain a fixed-width
`BluetoothPeripheralId` (16 UUID bytes in canonical-string octet order) and optional `i32` RSSI in
dBm. The ID is an OS-assigned peer UUID, not a hardware address or cross-platform identity promise.

The caller polls already-delivered events with `try_next_discovery` and checks the backend's
`BluetoothScanState`. The iOS adapter uses a 32-event queue, drops the newest event on overflow, and
reports a saturating drop count. Stopping preserves values already in the queue. The portable
contract has no executor, user callback, Apple type, global runtime, allocation, or dynamic dispatch.
An explicit native scan can prompt for permission; the separate authorization query does not.

## Central connection

`BluetoothConnectionBackend` adds one explicit caller-selected connection to a
`BluetoothCentral` backend that also implements discovery. `connect_peripheral` accepts a previously
reported `BluetoothPeripheralId`; the native backend resolves that OS-assigned UUID and reports a
typed `PeripheralUnavailable` failure if it can no longer retrieve the peer. The UUID is not a
promise that the operating system retains a retrievable peripheral forever.

The caller checks `connection_state`, polls copied `BluetoothConnectionEvent` values, and invokes
`cancel_connection` to cancel a pending or active request. A backend may have only one active native
request; a second request returns `AlreadyActive` while the first remains active. Native requests
finish at their terminal callback; failures and cancellation before native connection finish when
reported. Cancellation is nonblocking, and a late native connect result does not revive a request
after cancellation. The backend keeps its native peripheral alive until a terminal callback or
backend teardown.

The iOS adapter holds at most 16 connection events, drops the newest event on overflow, and reports
a saturating dropped-event count. `connection_state` remains a snapshot if events are dropped, but a
dropped failure event can lose its detailed normalized reason. Native error text/codes do not cross
this portable boundary. An explicit connection can initialize CoreBluetooth and prompt for
Bluetooth authorization; `Bluetooth::authorization_status` remains a separate non-prompting query.
The connection contract alone performs no service or characteristic discovery; the separate
service-discovery and characteristic-read contracts below add those explicit operations.

## Connected-peer GATT service discovery

`BluetoothServiceDiscoveryBackend` adds an explicit `discover_services` request to a backend that
also supports a caller-selected central connection. The request is accepted only when that
selected peripheral is currently connected; an active request cannot overlap another request.
The backend discovers all GATT services and returns before native results arrive.

`BluetoothServiceUuid` stores 16 bytes in canonical UUID-string octet order. Native 16-bit and
32-bit Bluetooth UUID forms are expanded with the Bluetooth Base UUID; 128-bit values retain their
canonical bytes. `BluetoothServiceDiscoveryEvent` reports copied `ServiceDiscovered`, `Completed`,
or normalized `Failed` events. `service_discovery_state` remains queryable and
`try_next_service_discovery_event` drains the FIFO. The iOS adapter bounds this queue at 32 events,
drops newest on overflow, and reports/reset the saturating dropped-event count. State remains
queryable if a terminal event is among those dropped.

The B592 iOS service-discovery implementation uses public `CBPeripheral.discoverServices:` with nil
service filters, then copies only each `CBService.UUID` without retaining a service object. It reports
no characteristic result. Disconnect ends an active request with `Disconnected`; native errors and
conversion/callback failures have distinct normalized terminal errors. CoreBluetooth provides no
discovery-only cancellation method; cancelling the selected connection terminates active
discovery with `Disconnected` and removes the weak peripheral delegate. Backend drop clears both
weak delegates before release. Callbacks remain on the main queue and no executor or user callback
is introduced.

## Connected-peer GATT characteristic read

`BluetoothCharacteristicReadBackend` adds one explicit, read-only value request after successful
service discovery. `read_characteristic` selects a service and characteristic by their fixed-width,
canonical 128-bit UUIDs; the iOS backend rejects an absent or duplicate service immediately, then
discovers characteristics and rejects an absent, duplicate, or non-readable match. An active service
discovery, read, or write blocks another GATT operation with `AlreadyActive`.

The iOS adapter uses public `CBPeripheral.discoverCharacteristics:forService:` and
`readValueForCharacteristic:` APIs. It copies a complete `NSData` result into a 512-byte inline
`BluetoothCharacteristicValue`; larger values fail rather than truncate. A four-entry FIFO stores
copied terminal events and drops newest on overflow with a saturating drop count. The operation
state remains queryable if a terminal event is dropped. Disconnect or `cancel_connection` ends an
active read with `Disconnected`; backend drop suppresses later callbacks. CoreBluetooth has no
separate read-cancel or timeout request here, so a read otherwise remains active until a native
callback, disconnect, or backend teardown.

The portable value exposes no CoreBluetooth handle, borrowed buffer, or native error details. This
contract does not write values, subscribe to notifications, discover descriptors, or implement a
peripheral role. Host policy tests and device/Simulator compile/link checks do not prove a live GATT
read.

`BluetoothCharacteristicWriteBackend` adds one explicit write-with-response operation for a unique
connected-peer service/characteristic UUID pair with the public GATT Write property. The iOS backend
copies and retains the caller payload through CoreBluetooth's terminal write callback; it rejects
payloads above the 512-byte attribute bound or the current native limit (20 bytes conservatively on
iOS 8). A disconnect/cancel during native characteristic discovery or a write reports
`Disconnected` and quarantines later writes on that backend instance because CoreBluetooth
callbacks carry no operation token. Its
four-event FIFO reports only CoreBluetooth's write result, not an application-level effect. No
write-without-response, notification subscription, descriptor operation, or peripheral role is
implemented; static checks do not prove live radio behavior.

See [iOS Bluetooth central discovery](../ios/bluetooth-discovery.md) for queue, lifecycle, privacy,
and runtime limits.
