# Portable Bluetooth

`framework-bluetooth` defines portable authorization, central-discovery, and explicit central-
connection contracts. Platform managers and peripheral objects remain private to native backends.
The portable API has no advertisement, peripheral-role, beacon, or radio-control contract.

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
This contract performs no service or characteristic discovery and no GATT reads or writes.

See [iOS Bluetooth central discovery](../ios/bluetooth-discovery.md) for queue, lifecycle, privacy,
and runtime limits.
