# PLAN_IOS_KEY_SUPPORT.md — B69: iOS P-256 key suitability

## Objective

Implement the narrow D63 backend using public Security and Core Foundation bindings, with no Swift
source or handwritten ABI shim.

## Scope

`platform/ios/ios-key-support` implements
`P256VerificationSupportBackend`. It copies the caller's fixed 65-byte public-key value to
temporary `CFData`, imports it with public EC/P-256 attributes through
`SecKeyCreateWithData`, and calls `SecKeyIsAlgorithmSupported` for `Verify` with
`kSecKeyAlgorithmECDSASignatureMessageX962SHA256`.

The imported `SecKey`, attributes, data, and any error are temporary. This package does not
verify a signature, generate or use a private key, persist key material, call Keychain APIs, access
Secure Enclave, prompt, or provide a general crypto API.

## Availability

The Security declarations and algorithm constant used here are available from iOS 10.0. The crate
does not set a deployment target. No permission, usage-description key, or entitlement is required
for this in-memory public-key query.

## Dependency boundary

Use the existing `objc2-security` 0.3.2 generated bindings with only `SecBase`, `SecItem`, and
`SecKey`, and `objc2-core-foundation` 0.3.2 with `alloc`, `CFData`, `CFDictionary`,
`CFError`, `CFNumber`, and `CFString`. The `SecItem` feature is present in the selected
binding feature set, but this implementation imports no Keychain symbol.

## Evidence and limits

Run `sh platform/ios/ios-key-support/check.sh` for the portable and iOS compile/lint/rustdoc
gates. Run `sh platform/ios/ios-key-support/check-link-imports.sh` to build and inspect Release
probes without executing them. G63 results and exact imports are in
`PLAN_VALIDATION_IOS_KEY_SUPPORT.md`. No live key query, point-validation result, signature
verification, parity, or runtime behavior is claimed.
