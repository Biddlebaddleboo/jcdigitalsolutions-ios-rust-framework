# iOS P-256 key suitability query

## Scope

`framework-key-support` defines a `no_std` contract for one question: whether an imported P-256
public key is suitable for ECDSA/SHA-256 message verification. `ios-key-support` implements it with
the public Security C APIs `SecKeyCreateWithData` and `SecKeyIsAlgorithmSupported`.

The caller supplies a borrowed 65-byte ANSI X9.63 uncompressed P-256 point (`04 || X || Y`). The
portable constructor checks the `04` uncompressed-point marker and length; Security validates the
point when it imports the key. The iOS backend copies the bytes to temporary `CFData`, imports a
public EC key with its type, public class, and 256-bit size, then asks whether the `Verify` operation
supports `kSecKeyAlgorithmECDSASignatureMessageX962SHA256`. It releases the temporary objects and
does not retain or persist the key.

The result is only Security's key/operation/algorithm suitability Boolean. It does not verify a
signature, prove that a later verification succeeds, generate or use a private key, write to the
Keychain, or access the Secure Enclave. It does not report device integrity or general cryptographic
support.

## Availability and host requirements

The iOS SDK declares `SecKeyCreateWithData`, `SecKeyIsAlgorithmSupported`, and the selected ECDSA
algorithm constant available from iOS 10.0. The crate does not set a deployment target; an app must
select iOS 10.0 or later. The scoped link gate uses iOS 10.0 for the arm64 device target and iOS
14.0 for the Rust arm64 Simulator target. Those are link-probe deployment floors, not different
Security API floors.

This in-memory public-key import and suitability query does not access a keychain item, create a
private key, or request user authorization. No permission prompt or usage-description key is part of
this slice. The app must not infer Secure Enclave availability or Keychain access from this result.

## Errors and execution

The portable constructor returns `InvalidInput` for a non-`04` point marker. Invalid point data
rejected by Security returns `Platform`, with the native `CFError` code when it fits the framework's
`i32` platform-code type; the `CFError` domain is not retained. Failure to allocate the temporary
Core Foundation data or attributes returns `ResourceExhausted`.

The query is synchronous. It copies the 65 public-key bytes once into `CFData`, constructs a small
attributes dictionary, and creates a temporary Security key object. It starts no callback or task,
shows no UI, and performs no signature operation. The support result is a point-in-time platform
answer, not a guarantee about a later operation.

## Bindings and validation

The backend uses the existing `objc2-security` 0.3.2 generated surface with `SecBase`, `SecKey`,
and `SecItem`, plus `objc2-core-foundation` 0.3.2 with `alloc`, `CFData`, `CFDictionary`, `CFError`,
`CFNumber`, and `CFString`. `SecKeyCreateWithData` and `SecKeyIsAlgorithmSupported` are generated
typed bindings; no handwritten ABI shim or Swift source is used. The expected direct link set is
Security.framework, CoreFoundation.framework, and `libSystem.B.dylib`.

Run `sh platform/ios/ios-key-support/check.sh` for the portable `no_std` check, strict Clippy,
rustdoc, iOS device/Simulator check and strict Clippy, formatting, and shell syntax gates. Run
`sh platform/ios/ios-key-support/check-link-imports.sh` to build—but never execute—arm64 device and
Simulator link probes, audit exact imports and required/forbidden symbols, and inspect deployment
metadata. These gates do not establish point validity, key suitability on live hardware, actual
signature verification, persistence, Secure Enclave behavior, or runtime behavior.
