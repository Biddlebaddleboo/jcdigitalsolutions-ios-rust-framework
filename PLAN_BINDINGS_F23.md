# PLAN_BINDINGS_F23.md — F23: iOS P-256 key-suitability C ABI

## Objective

Expose one opt-in C query over D63/B69. It reports only the iOS Security suitability Boolean for one caller-supplied P-256 public key and the ECDSA/SHA-256 message-verification algorithm

## Status

F23-owned source, public header, static check, device/Simulator link-import gate, guide, and this plan are ready for root wiring. Rust formatting, shell syntax, C11/C++17 header syntax-only checks, Rust/header symbol parity, scalar output memory/lifetime/concurrent-access precondition checks, and F23 whitespace checks pass. Cargo feature isolation, compile/Clippy, native link/import, and deployment checks remain pending root feature/dependency wiring and Cargo.lock refresh. The link-import gate is authored but was not executed. No tests, linked consumers, or probes ran

## Backend and API bounds

The candidate matrix row `019-security-auth-seckey-secure-enclave` lists `framework-key-support` and `ios-key-support` as partial support. The existing C crypto slice F19 covers only CommonCrypto SHA-256; no current C symbol exposes the separate B69 public-key suitability query. F22 remains the independent Accelerate vector-add slice

B69 exposes `P256PublicKey::from_x963_uncompressed(&[u8; 65])` and `IosP256VerificationSupport::query`. The portable constructor checks only the `0x04` uncompressed marker. The iOS backend imports the key into temporary Core Foundation values and calls `SecKeyIsAlgorithmSupported` for `Verify` with `kSecKeyAlgorithmECDSASignatureMessageX962SHA256`. The iOS API floor is 10.0; no permission, usage-description key, or entitlement is required

The sole export is `framework_ios_key_support_p256_ecdsa_sha256_message_supported(FrameworkSlice x963_public_key, uint8_t *out_supported)`. The input length must be exactly 65 bytes; the non-null input must remain readable and immutable for the call. A non-null output must address valid, properly aligned writable memory for one byte during this synchronous call and be disjoint from input. The client must prevent unsynchronized concurrent access. The API checks pointer-range arithmetic and overlap but cannot validate memory; it does not retain the output address. A structurally valid output is initialized to zero before marker/platform handling. Malformed length, null input/output, pointer-range overflow, or overlap returns `INVALID_ARGUMENT` without output write. On iOS, a missing uncompressed marker returns `INVALID_ARGUMENT`; invalid curve coordinates may fail at Security import and map to `PLATFORM_ERROR`

On success, output is exactly zero or one and status is `OK`. A false result means Security reports the key unsuitable for this operation; it is not a signature-verification attempt. Temporary Core Foundation allocation failure maps to `RESOURCE_EXHAUSTED`; native import failure maps to `PLATFORM_ERROR`; a caught panic maps to `PANIC` with zero output. A valid non-iOS call returns `UNSUPPORTED` with zero output. No pointer or key object is retained. The call is synchronous and adds no main-thread rule or thread-safety promise

This API does not verify a signature; create or use private key material; persist a key; access Keychain or Secure Enclave; or establish that a later verification succeeds. It makes no general crypto, parity, certification, or performance claim

## F23-owned files

- `bindings/c/src/ios_key_support.rs`
- `bindings/c/include/framework_ios_key_support.h`
- `bindings/c/check-ios-key-support.sh`
- `bindings/c/check-ios-key-support-link.sh`
- `docs/bindings/ios-key-support.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, `PLAN_BINDINGS.md`, and documentation indexes

## Static gate and root integration

`sh bindings/c/check-ios-key-support.sh` checks shell syntax for both F23 gates, Rust formatting, source/header symbol parity, trailing whitespace, and standalone C11/C++17 header syntax. It performs no Cargo command, native link, test, consumer execution, or probe

`sh bindings/c/check-ios-key-support-link.sh` is authored but pending root wiring and execution. It checks manifest semantics and feature isolation; runs host/device/Simulator checks and strict Clippy; builds Release archives; links C11/C++17 consumers without running them; checks exact CoreFoundation/Security/libSystem imports (+ libc++ for C++), `_SecKeyCreateWithData`, `_SecKeyIsAlgorithmSupported`, forbidden-symbol absence, export parity, and minos 10.0/device and 14.0/Simulator

After root wiring, add target-iOS optional dependencies on `framework-key-support` and `ios-key-support`, plus `ios-key-support = ["dep:framework-key-support", "dep:ios-key-support"]`. Gate the source module and public function re-export with that feature. Add `.optional_capabilities.ios_key_support` to the ABI manifest with only the sole export, borrowed input and one-byte output contract, status map, iOS API floor 10.0, direct imports CoreFoundation/Security/`libSystem.B.dylib`, and link-probe minima 10.0/device and 14.0/Simulator. Refresh Cargo.lock, run both F23 gates, add both to macOS CI, then record F23 in the aggregate binding plan and link the guide from the binding documentation index

The root-owned full target gate should mirror G63's host/device/Simulator feature isolation, strict Clippy, Release archive and C11/C++17 links. Inspect only CoreFoundation, Security, `libSystem.B.dylib` (+ `libc++.1.dylib` for C++), `_SecKeyCreateWithData`, `_SecKeyIsAlgorithmSupported`, absence of Keychain/key-generation/signature-operation imports, export parity, and target minima 10.0/device and 14.0/Simulator. Link probes are build-and-inspect only, never execute. This plan records no linked ABI or live query evidence
