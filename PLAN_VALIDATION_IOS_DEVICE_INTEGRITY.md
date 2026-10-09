# PLAN_VALIDATION_IOS_DEVICE_INTEGRITY.md — Workstream G36: DeviceCheck Support Gates

## Objective

Gate D37/B42's support-only API without generating security credentials or contacting Apple services.

## Required gates

- `cargo check --locked -p framework-device-integrity --no-default-features`
- strict Clippy for the portable contract
- iOS device and Simulator `cargo check` and strict Clippy for `ios-device-integrity`
- Release link/import audit for `DeviceCheck`, Foundation, libobjc, and libSystem; check Objective-C symbols and selected class/selector strings
- docs, zero-Swift-source, and diff checks

The probe is linked, not run. No live support result, token/key operation, attestation/assertion, server request, entitlement validation, or integrity result is claimed.
