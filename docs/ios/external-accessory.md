# iOS external-accessory presence snapshot

`ios-accessory` implements the portable `ExternalAccessoryBackend` contract by calling
`EAAccessoryManager.sharedAccessoryManager()` and reading
`EAAccessoryManager.connectedAccessories`. It maps `count() == 0` to
`AccessoryPresenceSnapshot::NoneAvailable` and a nonzero count to
`AccessoryPresenceSnapshot::OneOrMoreAvailable`.

Apple documents `connectedAccessories` as the list of accessory objects currently connected and
available for the app to use. Its contents can change dynamically, so the adapter does not cache
the native array or retain a prior snapshot. The scalar result describes only that API list at
call time; an empty list does not prove that no physical accessory is attached or that the device
lacks ExternalAccessory support.

The adapter does not call the Bluetooth picker, register for connection notifications, inspect
protocol strings, create `EASession`, obtain input/output streams, or communicate with hardware.
It does not verify the host app's `UISupportedExternalAccessoryProtocols`, MFi eligibility,
entitlements, or other provisioning metadata; no requirement or permission claim is made here.
Accessory communication and AccessorySetupKit are outside this slice.

The `EAAccessoryManager` and `connectedAccessories` declarations in the installed iOS SDK are
available from iOS 3.0. The binding is `objc2-external-accessory` 0.3.2 with default features
disabled and only `EAAccessory` plus `EAAccessoryManager` enabled. The `EAAccessoryManager`
binding feature gates the getter's supporting Foundation types; no picker block, session, stream,
Wi-Fi accessory browser, or transfer feature is enabled.

## Package check gate

From the repository root, run
`platform/ios/ios-accessory/scripts/check.sh`. It checks formatting, the portable `no_std` crate
on the host, strict Clippy and rustdoc, iOS device and simulator target compilation, and a source
guard for this snapshot-only API. It does not run tests, use a live accessory, or open a hardware
session. The target checks require Xcode plus `aarch64-apple-ios` and `aarch64-apple-ios-sim` Rust
targets.

## References

- [Apple `EAAccessoryManager`](https://developer.apple.com/documentation/externalaccessory/eaaccessorymanager)
- [Apple `connectedAccessories`](https://developer.apple.com/documentation/externalaccessory/eaaccessorymanager/connectedaccessories?language=objc)
- [Apple `UISupportedExternalAccessoryProtocols`](https://developer.apple.com/documentation/bundleresources/information-property-list/uisupportedexternalaccessoryprotocols)
