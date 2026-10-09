# PLAN_VALIDATION_IOS_KEY_SUPPORT.md — G63: P-256 key suitability gates

## Gate

G63 validates the D63/B69 portable P-256 public-key contract and its iOS Security suitability
query. The focused commands are:

- `sh platform/ios/ios-key-support/check.sh`: portable no-default-features check, strict Clippy,
  rustdoc, iOS device and Simulator checks, strict all-target Clippy, formatting, and shell syntax.
- `sh platform/ios/ios-key-support/check-link-imports.sh`: build Release device and Simulator
  probes, audit imports and symbols, and inspect deployment metadata. The probes are not executed.

## Recorded result (2026-10-08)

Both focused commands passed on Rust 1.94.1, Xcode 26.6 (build 17F113), and iPhoneOS/iPhoneSimulator
SDK 26.5. Device and Simulator imports are CoreFoundation.framework, Security.framework, and
`libSystem.B.dylib`. Required symbols are `SecKeyCreateWithData` and
`SecKeyIsAlgorithmSupported`; no Swift, Objective-C messaging, Keychain, key-generation, or
cryptographic-operation symbols are imported. Probe minimum deployment versions are iOS 10.0 and
iOS Simulator 14.0.

The key point marker, API feature boundary, rustdoc, host/device/Simulator compile and Clippy gates
passed. Probes were built and inspected, not run. No live key import, device behavior, signature
verification, Keychain persistence, Secure Enclave operation, or parity is established. CI invokes
both focused scripts; no passing CI workflow run is recorded.
