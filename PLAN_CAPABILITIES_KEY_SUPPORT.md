# PLAN_CAPABILITIES_KEY_SUPPORT.md — D63: bounded P-256 key suitability

## Objective

Define one portable value and backend contract for asking whether a caller-supplied P-256 public
key is suitable for ECDSA/SHA-256 message verification.

## Scope

- `framework-key-support` is `no_std` and accepts a borrowed 65-byte ANSI X9.63 uncompressed
  point (`04 || X || Y`).
- The portable constructor validates the fixed length by type and checks only the `0x04` point
  marker. It does not validate curve coordinates.
- The backend imports the public key in memory and returns a point-in-time platform suitability
  Boolean for Verify with ECDSA/SHA-256 message signing.
- Invalid public-key coordinates may be rejected during platform import.
- No signature verification, private-key operation or generation, Keychain persistence, Secure
  Enclave access, general crypto facade, parity, or performance claim is in scope.

## Implementation

`crates/framework-key-support` defines `P256PublicKey`,
`P256VerificationSupport`, and the statically selected
`P256VerificationSupportBackend` contract. The iOS implementation is tracked in
`PLAN_IOS_KEY_SUPPORT.md`.

The portable constructor returns `InvalidInput` when the uncompressed-point marker is absent.
The backend maps Security import failure to `Platform`, including a native `CFError` code only
when it fits the framework error type; temporary Core Foundation allocation failures return
`ResourceExhausted`.

## Availability and permissions

The iOS Security API floor is iOS 10.0. This in-memory public-key query needs no permission,
usage-description key, or entitlement. It does not imply keychain or Secure Enclave access.

## Evidence

- Portable no-default-features check, strict Clippy, and rustdoc pass.
- See `PLAN_VALIDATION_IOS_KEY_SUPPORT.md` for iOS target, import, deployment-floor, and CI
  evidence.
- Public-key validity, live device behavior, later signature verification, and Apple parity remain
  untested.
