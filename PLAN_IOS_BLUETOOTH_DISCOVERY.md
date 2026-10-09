# PLAN_IOS_BLUETOOTH_DISCOVERY.md — Workstream D29

## Objective

Extend D27's no-std Bluetooth contract and B32's iOS adapter with explicit, unfiltered central discovery. Report copied peer UUID bytes and optional RSSI; do not add Swift, connect, retain a native peripheral, expose advertisement data, or add peripheral-role APIs.

## Evidence reviewed before implementation

- Installed Xcode 26.6 / iPhoneOS 26.5 SDK headers: `CBCentralManager` and `scanForPeripheralsWithServices:options:` are available from iOS 5.0; `initWithDelegate:queue:` sends central events to its supplied queue and documents nil as the main queue; `delegate` is weak; `stopScan` stops discovery; RSSI value 127 means unavailable. `CBPeer` is available from iOS 8.0 and its `identifier` property is annotated iOS 7.0, so this identity-returning adapter's effective floor is iOS 8.0. `NSUUID` is available from iOS 6.0.
- `objc2-core-bluetooth` 0.3.2 generated API: `CBCentralManager::initWithDelegate_queue`, `setDelegate`, `scanForPeripheralsWithServices_options`, `stopScan`, required `CBCentralManagerDelegate::centralManagerDidUpdateState`, optional `centralManager_didDiscoverPeripheral_advertisementData_RSSI`, `CBPeer::identifier`, and `CBManagerState` are present behind the package-local features.
- Apple docs: nil service UUIDs return all discovered peers; Apple recommends a service filter. The documented `bluetooth-central` background mode requires one or more service UUIDs. `CBPeer.identifier` is the unique UUID that CoreBluetooth assigns when it first encounters a peer.
- Apple Bluetooth privacy docs: apps linked on or after iOS 13 need `NSBluetoothAlwaysUsageDescription`. Apps with deployment targets earlier than iOS 13 need both that key and deprecated `NSBluetoothPeripheralUsageDescription`.

## Portable contract

- `BluetoothPeripheralId` stores 16 UUID bytes in canonical UUID-string octet order. It is an OS-assigned peer UUID, not a MAC address, serial number, or cross-platform identity promise.
- `BluetoothDiscovery` stores that ID and optional `i32` dBm. CoreBluetooth's reserved RSSI 127 maps to `None`.
- `BluetoothCentralBackend` / `BluetoothCentral` expose only explicit `start_unfiltered_scan`, `stop_scan`, lifecycle state, pull of already delivered discoveries, and a resettable dropped-event count.
- Start returns before CoreBluetooth readiness or any discovery callback. No executor, callback into user code, platform type, global runtime, allocation, or dynamic dispatch is part of `framework-bluetooth`.
- The iOS adapter has a fixed 32-event queue; when full it drops the newest event and increments a saturating counter. Stop preserves already queued discoveries and ignores later callbacks.

## iOS contract and limits

- `IosBluetoothCentralBackend::new(MainThreadMarker)` is idle and creates no native manager. Only an explicit `start_unfiltered_scan` lazily creates `CBCentralManager`; that operation may prompt for Bluetooth authorization. The non-prompting D27 authorization query remains a separate API with an iOS 13.1 floor.
- The manager uses a nil callback queue, hence CoreBluetooth's documented main queue. The backend retains the delegate because `CBCentralManager.delegate` is weak. Construct, call, and drop the backend on the same main thread. Drop stops the scan and clears the native delegate.
- A scan command is issued only after CoreBluetooth reports `PoweredOn`. `BluetoothScanState::Scanning` means the adapter issued the command; it is not a hardware or radio proof. Power loss maps to `WaitingForPower`; a later powered-on callback resumes a still-requested scan.
- The scan API itself has an iOS 5.0 floor, but this D29 adapter also calls `CBPeer.identifier`; its effective API floor is iOS 8.0 from the installed SDK header. The separate `CBManager.authorization` class query remains iOS 13.1.
- This is an unfiltered foreground API: it passes nil services and nil options, exposes no service filter, does not configure `bluetooth-central` or the distinct iOS 26 Live Activity background path, and provides no background/restoration guarantee. The host app owns its privacy keys, background modes, lifecycle, and user-facing rationale.
- No device or simulator Bluetooth behavior, prompt, radio, permission, peer reach, or background execution is claimed by compile checks.

## Validation

See [PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md](PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md) for portable tests/no-default checks, iOS device/simulator checks, Clippy, docs, feature review, and diff gates. Target checks compile the Objective-C bindings only; they do not run a live scan.

## Apple references

- [CBCentralManager scan](https://developer.apple.com/documentation/corebluetooth/cbcentralmanager/scanforperipherals%28withservices%3Aoptions%3A%29)
- [CBCentralManagerDelegate discovery callback](https://developer.apple.com/documentation/corebluetooth/cbcentralmanagerdelegate/centralmanager%28_%3Adiddiscover%3Aadvertisementdata%3Arssi%3A%29)
- [CBPeer identifier](https://developer.apple.com/documentation/corebluetooth/cbpeer/identifier)
- [Core Bluetooth framework privacy requirements](https://developer.apple.com/documentation/corebluetooth/)
- [NSBluetoothPeripheralUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsbluetoothperipheralusagedescription)
