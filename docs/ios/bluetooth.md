# iOS Bluetooth authorization status

`ios-bluetooth` implements the portable [`framework-bluetooth`](../capabilities/bluetooth.md)
status query through the public CoreBluetooth class property. The adapter is intentionally
stateless: it does not create a `CBManager`, `CBCentralManager`, or `CBPeripheralManager`.

```rust,ignore
use framework_bluetooth::Bluetooth;
use ios_bluetooth::IosBluetoothBackend;

fn current_bluetooth_authorization() -> framework_bluetooth::BluetoothAuthorization {
    Bluetooth::new(IosBluetoothBackend::new()).authorization_status()
}
```

`CBManager.authorization` is the class-level authorization query and can be read before allocating
a manager. This backend calls it only when `authorization_status()` is invoked. The query does not
create a manager, power the radio, request permission, present a prompt, scan, connect, advertise,
or prove that a Bluetooth operation will succeed. `NotDetermined` is only a status; this crate has
no permission-request operation.

Explicit central scan and its async pull queue are a separate opt-in API. See
[iOS Bluetooth central discovery](bluetooth-discovery.md); constructing the authorization backend
does not create or retain its manager.

## States and API floor

The installed Xcode 26.6 / iPhoneOS 26.5 SDK marks the `CBManager.authorization` class property as
available from iOS 13.1. This is the static query's API floor. The deprecated instance property is
available from iOS 13.0 until 13.1, but it would require a manager instance and is not used here.
The API floor is based on the installed SDK; this crate does not claim an Xcode 27 baseline or
support for calling the static query on earlier iOS versions.

CoreBluetooth maps Not Determined, Restricted, Denied, and Allowed Always to distinct portable
NotDetermined, Restricted, Denied, and Allowed values. Unknown future raw values map to Unknown.
Portable Allowed does not mean the hardware is powered on, a role is supported, or background
execution, scanning, or connection is available. This slice has no radio state or Bluetooth data
operation.

## Host app configuration

For an app that uses the device's Bluetooth interface, Apple requires the exact
`NSBluetoothAlwaysUsageDescription` key in the host app's `Info.plist`, with a user-understandable
reason. This crate does not edit the host property list. Apple's property-list documentation says
apps whose deployment target is earlier than iOS 13 must also include the deprecated
`NSBluetoothPeripheralUsageDescription` key; this backend's static API floor remains iOS 13.1.

## Dependency and validation limits

The authorization query calls only the `CBManager` class API. The package keeps
`objc2-core-bluetooth` 0.3.2 defaults disabled and also enables central-discovery bindings for the
separate opt-in API; see [iOS Bluetooth central discovery](bluetooth-discovery.md). The portable D27
API exposes no binding type. Package target checks compile the adapter against iOS SDKs, but do not
run a permission prompt, query a device's live authorization state, validate Bluetooth
hardware/radio behavior, or perform scans or connections.

References: [Apple CBManager](https://developer.apple.com/documentation/corebluetooth/cbmanager),
[Apple CBManager authorization](https://developer.apple.com/documentation/corebluetooth/cbmanager/authorization-swift.type.property),
[Apple authorization states](https://developer.apple.com/documentation/corebluetooth/cbmanagerauthorization),
and [Apple NSBluetoothAlwaysUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothalwaysusagedescription).
