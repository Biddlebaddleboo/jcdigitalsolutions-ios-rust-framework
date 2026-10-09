# DeviceCheck and App Attest support

`framework-device-integrity` exposes `AvailabilitySnapshot`, a copied report of whether the selected platform APIs report support for DeviceCheck and App Attest.

```rust
let snapshot = ios_device_integrity::availability_snapshot();
let device_check_supported = snapshot.device_check_supported();
let app_attest_supported = snapshot.app_attest_supported();
```

The booleans are transient capability hints. `true` does not establish device integrity, app identity, enrollment, entitlement configuration, server trust, or success of token/key generation, attestation, or assertions. The facade does not expose these operations.

See the [iOS support-query guide](../ios/device-integrity.md) for API floors and backend limits.
