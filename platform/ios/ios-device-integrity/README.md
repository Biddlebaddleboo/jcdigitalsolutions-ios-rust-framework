# iOS DeviceCheck and App Attest support snapshot

`ios-device-integrity::availability_snapshot()` reports only the native `DCDevice.isSupported`
and `DCAppAttestService.isSupported` values in the portable
`framework_device_integrity::AvailabilitySnapshot` type. The snapshot is synchronous and does not
prompt, generate a DeviceCheck token or App Attest key, attest or assert a key, or contact a server.

```rust
let support = ios_device_integrity::availability_snapshot();
let device_check_supported = support.device_check_supported();
let app_attest_supported = support.app_attest_supported();
```

The backend's minimum API floor and required app deployment target are iOS 11.0 because the typed
binding links `DeviceCheck.framework` directly and `DCDevice.currentDevice` and `isSupported` are
available from iOS 11.0. The query also checks runtime availability before calling these APIs.
`DCAppAttestService.sharedService` and `isSupported` are queried only when the runtime is iOS 14.0
or later; iOS 11.0 through 13.x report App Attest support as `false`. Both API sets are in
`DeviceCheck.framework`.

On non-iOS targets, both support values are `false` because these APIs are iOS-specific.

The native support booleans are not an integrity result or a guarantee that later operations will
succeed. They do not establish app identity, registration, enrollment, entitlement configuration,
or server-side verification. In particular, Apple documents that App Attest support can vary by
extension type and that a positive support value does not guarantee that key generation is valid
from an extension. Keep these values transient and do not use them as a security decision.

The iOS dependency uses `objc2-device-check` 0.3.2 with default features disabled and only the
`DCAppAttestService` and `DCDevice` class features enabled. This excludes the crate's `block2`
feature, so token-generation, key-generation, attestation, and assertion completion-handler APIs
are not enabled by this backend.

Targeted non-test validation:

```sh
cargo fmt --check -p framework-device-integrity -p ios-device-integrity
cargo check --locked -p framework-device-integrity
cargo check --locked --no-default-features -p framework-device-integrity
cargo check --locked -p ios-device-integrity
cargo check --locked -p ios-device-integrity --target aarch64-apple-ios
cargo check --locked -p ios-device-integrity --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-device-integrity --target aarch64-apple-ios --lib -- -D warnings
cargo clippy --locked -p ios-device-integrity --target aarch64-apple-ios-sim --lib -- -D warnings
cargo doc --locked --no-deps -p framework-device-integrity -p ios-device-integrity
```
