# Portable Bluetooth

`framework-bluetooth` defines portable authorization and central-discovery contracts. It contains no
platform manager, connection, advertisement, peripheral, beacon, or radio-control API.

## Authorization states

`BluetoothAuthorization` distinguishes Unknown, NotDetermined, Restricted, Denied, and Allowed.
`Allowed` reports that the backend's platform authorization status permits Bluetooth use. It does
not say that the radio is powered on, that a Bluetooth role is supported, or that a later scan or
connection will succeed. A status query is a snapshot and does not request permission.

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

See [iOS Bluetooth central discovery](../ios/bluetooth-discovery.md) for queue, lifecycle, privacy,
and runtime limits.
