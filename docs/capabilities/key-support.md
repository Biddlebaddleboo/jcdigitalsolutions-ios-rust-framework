# P-256 verification suitability contract

The `framework-key-support` crate is a `no_std` portable contract for one operation-specific
query: whether a platform considers a caller-supplied P-256 public key suitable for ECDSA/SHA-256
message verification.

## Public-key value

`P256PublicKey` borrows exactly 65 bytes in ANSI X9.63 uncompressed form,
`04 || X || Y`. The constructor checks the uncompressed-point marker. It does not validate the
curve coordinates; the platform backend may reject invalid point data while importing the key.
The bytes are public-key material, not a private key or secret.

## Query contract

`P256VerificationSupportBackend` reports the platform's point-in-time suitability result for
the Verify operation and ECDSA/SHA-256 message algorithm. A positive result does not verify a
signature or guarantee that a later verification succeeds.

This contract does not define private-key generation/use, Keychain persistence, Secure Enclave
behavior, signing, general cryptography, or a portable cryptographic implementation. It has no
permission or entitlement requirement. See the [iOS Security guide](../ios/key-support.md) for
the bounded backend and error mapping.
