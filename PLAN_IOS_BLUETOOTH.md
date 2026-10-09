# PLAN_IOS_BLUETOOTH.md — Workstream B32: iOS Bluetooth Authorization Query

## Objective

Implement D27's non-prompting authorization query with the public CoreBluetooth `CBManager.authorization` class property. Central discovery is specified separately in [PLAN_IOS_BLUETOOTH_DISCOVERY.md](PLAN_IOS_BLUETOOTH_DISCOVERY.md).

## Dependencies

- Foundation A and iOS runtime B are integrated
- D27 `framework-bluetooth` is integrated
- Inspect installed SDK headers, generated binding signatures/features, and Apple documentation before stating availability or permission requirements

## Write scope

- `platform/ios/ios-bluetooth/**`
- `docs/ios/bluetooth.md`
- `PLAN_IOS_BLUETOOTH.md`

Do not edit the portable D27 crate, root workspace configuration or lockfile, capability status data, aggregate indexes, CI, other iOS backends, Swift ABI, C bindings, or unrelated capability families.

## Required implementation

- Implement the portable authorization-query trait with a stateless backend; do not create `CBManager`, `CBCentralManager`, or `CBPeripheralManager` instances
- Query the public static `CBManager::authorization_class()` binding and map NotDetermined, Restricted, Denied, and AllowedAlways to distinct portable values
- Map unrecognized native raw values to Unknown
- Keep permission request, manager lifecycle, radio state, scans, connections, advertisements, peripheral APIs, and Bluetooth data out of scope
- Keep CoreBluetooth types and binding dependency private to the iOS crate
- For the authorization-only B32 path, `CBManager` is the only needed feature. The D29 extension separately enables central-discovery bindings; keep the crate's `objc2-core-bluetooth` default features disabled
- Record the `NSBluetoothAlwaysUsageDescription` host Info.plist key and any Apple-documented compatibility note for older deployment targets
- Document that the query does not itself prompt or prove radio/role availability and that no runtime Bluetooth operation is implemented

## Availability and limits

- The installed Xcode 26.6 / iPhoneOS 26.5 SDK marks `CBManager.authorization` available from iOS 13.1
- The deprecated instance authorization property is available from iOS 13.0 through 13.1; it is not used because this slice excludes manager allocation
- The CoreBluetooth framework and `CBManager` type have earlier availability; the adapter's static query floor remains iOS 13.1
- Do not claim support below iOS 13.1 or an Xcode 27 baseline

## Validation and handoff

- Add pure raw-status mapping tests; do not show a permission prompt or initialize a manager
- Run iOS device/simulator package checks, strict Clippy, format/docs/diff gates, and a package feature audit
- A package check is not a live authorization or Bluetooth-use test; do not claim simulator/device hardware behavior
- Report the iOS 13.1 floor, exact usage-description key, generated binding signature/features, runtime limits, and unresolved assumptions
