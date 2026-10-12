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

## B570 — explicit caller-selected central connection

D29's exclusion of connection was a historical scope boundary. B570 adds one explicit connection
to an OS-known peer UUID while preserving D29's unfiltered scan and B32's stateless,
non-prompting authorization query. The portable `BluetoothConnectionBackend` is `no_std` and
statically dispatched through `BluetoothCentral`; the iOS implementation reuses the existing lazy
main-queue `CBCentralManager` and delegate.

The caller selects `BluetoothPeripheralId`. Once the manager is `PoweredOn`, iOS resolves exactly
that UUID with `retrievePeripheralsWithIdentifiers`, verifies the returned peer ID, retains at most
one `CBPeripheral`, and calls `connectPeripheral:options:` with nil options. A missing match emits
`PeripheralUnavailable`; the UUID does not promise indefinite CoreBluetooth retrievability. Manager
authorization/role failures and native connection errors map to bounded portable error values, not
native `NSError` text. One request remains active until its terminal callback; overlapping requests
return `AlreadyActive` while the prior request remains active. Pre-native failure or cancellation
ends when reported; native requests end at their terminal callback. Cancellation before native
connection ends immediately; later cancellation is nonblocking, retains the peer until the
terminal callback, and suppresses a late connect success. CoreBluetooth callbacks identify the peer
but provide no request token; a callback for an earlier request arriving after a new request to the
same UUID begins cannot be distinguished from a callback for that new request.
Backend drop cancels any started request, stops discovery, clears the weak delegate, and releases
native objects on the required main thread. Delegate panics are caught and active requests are
cancelled.

Connection events have a separate 16-entry FIFO. Overflow drops the newest event and saturates a
`u32` drop count; the current state remains queryable if an event is lost. No service/characteristic
discovery, GATT IO, advertisement data, peripheral role, background execution, restoration, or
automatic reconnect is included. The effective identifier API floor remains iOS 8.0 because
`CBPeer` is iOS 8.0 even though `retrievePeripheralsWithIdentifiers:` is iOS 7.0.

Host policy regressions cover power/authorization/unsupported transitions, missing UUID lookup,
overlap rejection, cancellation before and after native start, late/mismatched callbacks, teardown,
panic cancellation, UUID byte order, and bounded queue FIFO/overflow. The registered
`ios-bluetooth` validator profile runs host tests, formatting, Clippy, Rustdoc, source guards, and
the focused device/Simulator connection link fixture through `check-link.sh`. The link fixture does
not run CoreBluetooth.

On Xcode 26.6 / iPhoneOS 26.5, the focused commands passed:

- `cargo +1.94.1 fmt -p framework-bluetooth -p ios-bluetooth -- --check`
- `cargo +1.94.1 test --locked --offline -p framework-bluetooth` — 5 passed
- `cargo +1.94.1 check --locked --offline -p framework-bluetooth --no-default-features`
- `cargo +1.94.1 test --locked --offline -p ios-bluetooth` — 15 passed; target-only link test not run on host
- `cargo +1.94.1 clippy --locked --offline -p framework-bluetooth -p ios-bluetooth --all-targets -- -D warnings`
- `cargo +1.94.1 doc --locked --offline --no-deps -p framework-bluetooth -p ios-bluetooth`
- `sh platform/ios/ios-bluetooth/check-link.sh` — device/Simulator connection link fixtures passed

No Simulator or physical-device runtime test ran. Compile/link success does not prove manager
authorization UI, peer retrieval, radio connection, callback timing, cancellation behavior on-device,
or background behavior. See [iOS Bluetooth central discovery and connection](docs/ios/bluetooth-discovery.md)
and the official CoreBluetooth references above.

## B592 — connected-peer GATT service discovery

B592 extends the existing caller-selected central connection with explicit discovery of all GATT
services on that connected peer. The portable contract adds `BluetoothServiceUuid`, a fixed 16-byte
UUID in canonical octet order, plus a statically dispatched `BluetoothServiceDiscoveryBackend` with
typed lifecycle state, normalized failures, and bounded copied service/result events. Native 16-bit
and 32-bit UUID strings expand through the Bluetooth Base UUID; full 128-bit UUIDs preserve their
canonical bytes.

The iOS adapter rechecks its selected connection state and the native
`CBPeripheralState::Connected` before a request. It installs its retained peripheral delegate,
issues public `CBPeripheral::discoverServices(None)`, then copies each `CBService::UUID` into a
fixed-width value. The queue holds 32 service and terminal events, drops newest on overflow, and
reports a saturating drop count; `service_discovery_state` remains queryable if a terminal event is
dropped. A native error, disconnect, malformed UUID, or contained callback panic ends the request
with a typed failure. CoreBluetooth offers no discovery-only cancellation method. Cancelling the
selected connection ends active discovery with `Disconnected` immediately and removes the weak
peripheral delegate; backend teardown suppresses later events and clears both weak delegates before
releasing the selected peer. Characteristic discovery and
GATT reads/writes, peripheral role, background restoration, and automatic reconnect remain out of
scope.

Host regressions cover fixed-width UUID byte order, 16/32/128-bit normalization, malformed UUIDs,
disconnected preflight, overlapping requests, mismatched/late callbacks, terminal failure, bounded
FIFO overflow, dropped-event count reset, and public facade behavior. The registered
`ios-bluetooth` profile includes the existing device/Simulator link fixture and asserts the new
callback, request, and negative scope guards. These checks establish host policy behavior and
compile/linkability only; no CoreBluetooth manager, permission prompt, live peer, radio result,
device/Simulator runtime behavior, or App Store package was exercised.

## B594 — read-only connected-peer GATT characteristic value

B594 adds `BluetoothCharacteristicUuid`, a fixed 16-byte canonical UUID, plus a static-dispatch
`BluetoothCharacteristicReadBackend` operation that requires an active caller-selected connection
and successful B592 service discovery for the same peer. The peer-scoped proof is invalidated on
disconnect, cancellation, connection retry, and new discovery, so a cached service array alone does
not allow reads after reconnect. The iOS backend requires exactly one service and characteristic
UUID match, requires the public GATT Read property, and invokes CoreBluetooth characteristic discovery
then value read. It copies at most 512 value bytes into an owned inline portable value and queues
terminal results in a four-event bounded FIFO. A missing/duplicate match, overlap, native error,
disconnect, unavailable/oversize value, or contained callback panic is rejected or reported once;
connection cancellation and backend drop suppress later callbacks. No write, notification,
descriptor, peripheral role, Swift source, or native handle is added.

The B594 host policy/selection/value/queue tests cover negative and terminal cases; the existing link
fixture now references the read facade and copied value API so device and Simulator profile gates
include its code path. The registered `ios-bluetooth` profile keeps native connection, callback,
cancellation, `no_std`, and ABI/link requirements, and checks the new characteristic/read API
markers. The peer-scope regression rejects stale global Completed state for another or reconnected
peer while preserving B592's public Completed state. The final
`ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json"
--capability ios-bluetooth` run passed 31 checks with 0 FAIL, 0 BLOCKED, and 2 SKIPPED. Skips are
Simulator execution and physical-device proof. Host tests passed `framework-bluetooth` 6/6 and
`ios-bluetooth` 36/36 (33 unit + 3 service-contract tests); device/Simulator compile, Clippy,
Rustdoc, and link checks passed. No live GATT read was performed; CoreBluetooth exposes no per-read
cancellation ID or timeout.

## B597 — connected-peer GATT characteristic writes

B597 adds `BluetoothCharacteristicWriteBackend` and a caller-owned `write_characteristic` facade for
one-at-a-time write-with-response operations over the existing connected-peer selection path. It
requires peer-scoped successful B592 service discovery, unique service/characteristic UUID matches,
the public GATT Write property, and a payload within both the 512-byte attribute bound and
CoreBluetooth's current write-with-response limit. The iOS adapter copies the borrowed bytes into
`NSData`, retains them through `didWriteValueForCharacteristic:error:`, and reports one terminal
event through a separate four-entry FIFO. Service discovery, reads, and writes cannot overlap. The
existing iOS 8.0 package floor is preserved: `maximumWriteValueLengthForType:` is called only when
the iOS 9.0 selector is present; older systems use a conservative 20-byte cap. CoreBluetooth
characteristic-discovery and write callbacks carry no operation token, so disconnect or callback
panic during either step quarantines later writes on that backend instance until it is recreated.
Write-without-response, notifications, descriptors, peripheral
role, and per-operation native cancellation remain out of scope.

Host regressions cover disconnected and incomplete discovery, foreign-peer proof, absent/duplicate
service and characteristic matches, missing Write property, 512-byte and negotiated-limit bounds,
overlap, native callback result policy, disconnect during discovery and write, panic during
discovery and write, late callback suppression, quarantine, and FIFO overflow. The link fixture
references the new facade; the registered profile adds a guard
for `didWriteValueForCharacteristic:` and already snapshots the package source/test tree. Using
`/Users/john/.local/bin/ios-rust-validate` (`source_sha=350971094cc3b77191cd6b3d07e01af04d71943d`,
SHA-256 `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`), the registered
`ios-bluetooth` profile passed 32 checks with 0 FAIL, 0 BLOCKED, and 2 SKIPPED at snapshot
`9c209b2f3fed1cb7b6a3dd6fd64c455941460067f67e8946fb041cb4e7cb4e35`. Device iOS 8.0 and Simulator
iOS 14.0 compilation, strict Clippy, Rustdoc, and link fixture passed; SKIPPED were Simulator
execution and physical-device proof. No radio write, peer response, or runtime callback is claimed.

## B598 — connected-peer GATT notification and indication subscription (row 039)

B598 adds one UUID-unique connected-peer Notify/Indicate subscription, async enable/disable state,
bounded copied values, and a separate four-event FIFO. It requires fresh B592 peer-scoped service
discovery, permits write-with-response after enable, and rejects reads while active because
CoreBluetooth shares an untagged value callback between reads and notifications. A confirmed disable
or interrupted callback path retains the native characteristic as a tombstone and quarantines later
reads/subscriptions until backend recreation, preventing stale notification values from being
misattributed. Host policy regressions cover selection, properties, state changes, native errors,
disable quarantine, disconnect, panic, and stale callbacks. No live radio or runtime callback is
claimed; the existing `ios-bluetooth` profile registers package tests, target builds, Clippy,
Rustdoc, and ABI/link checks for this source tree.

## B600 — bounded read-only local GATT peripheral (row 039)

B600 adds the no_std `BluetoothPeripheralService` contract for one caller-owned service with one
to eight unique static characteristics, each capped at 512 bytes, plus a statically dispatched
`BluetoothPeripheral` facade. `IosBluetoothPeripheralBackend` uses public `CBPeripheralManager` APIs
to publish the read-only service, advertise its UUID, report asynchronous authorization/power and
native failures, recover service publication after radio power loss, and stop/remove only its own
service. The backend is main-thread-owned; no Swift source, dynamic read callback, background
restoration, or live radio behavior is included.

Host policy regressions cover permission/power transitions, service-publication ordering and failure,
advertising failure, stop cleanup, callback ambiguity, and power-loss re-publication. Host contract
regressions cover invalid bounds, duplicate UUIDs, empty/excess service definitions, static value
copying, and start/stop facade semantics. `/Users/john/.local/bin/ios-rust-validate` (source SHA
`b6d422528c80671d828a4b0d40e9fc739652409a`, SHA-256
`4835271071ca3b7292f09a50d6d6c682f0c58f7d219045dd05e556553a451262`) ran
`ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-bluetooth` at snapshot `999a1ff5b563abdaa0549cef176bf6a44ba0f78ffca96a03b1d6d6e1c3ad209f`:
32 PASS, 0 FAIL, 0 BLOCKED, 2 SKIPPED. Host tests, device/Simulator compilation and strict
Clippy, Rustdoc, and Release link fixture passed. The skips are Simulator execution and
physical-device proof; no permission prompt, advertisement, remote GATT read, or background behavior
was tested.
