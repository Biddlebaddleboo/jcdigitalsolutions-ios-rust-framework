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

## 2026-10-10 hosted CI recheck

[CI run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431)
completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; the Ubuntu
24.04, macOS 15, and xcode-27 jobs all passed. The DeviceCheck/App Attest gate
(`sh platform/ios/ios-device-integrity/check.sh`, step 173) passed on both Apple jobs; its macOS-only
step was skipped on Ubuntu. The xcode-27 job recorded Xcode 27.0 build `27A266a`, iPhoneOS SDK
27.0, and iPhoneSimulator SDK 27.0. The run SHA is an ancestor of current `main`
(`f7197e7b48bb35136f308792ae51617011e8e619`); the targeted plan, CI workflow, Cargo
manifests/lockfile, and DeviceCheck capability paths are unchanged since the run.

This records hosted static support, compile/lint, and link/import checks only. The linked probe was
not run; all live support, credential, service, entitlement, and integrity behavior remains unverified.
