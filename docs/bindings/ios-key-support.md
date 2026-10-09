# iOS P-256 key-suitability C API

This opt-in API asks whether iOS Security reports one supplied P-256 public key suitable for ECDSA/SHA-256 message verification. It wraps B69's in-memory suitability query only. It does not verify a signature or promise a later verification call

## Call contract

`framework_ios_key_support_p256_ecdsa_sha256_message_supported` accepts a borrowed `FrameworkSlice` of exactly 65 bytes in ANSI X9.63 uncompressed form (`04 || X || Y`) and one output byte. The input must remain readable and immutable through return. A non-null output must address valid, properly aligned writable memory for one byte during the synchronous call and be disjoint from input. The client must prevent unsynchronized concurrent access. The API checks pointer-range arithmetic and overlap but cannot validate memory; it does not retain the output address. A valid, disjoint output starts at zero; malformed span metadata or overlap returns `INVALID_ARGUMENT` without a write

On iOS, a missing `0x04` marker returns `INVALID_ARGUMENT`. The backend imports the public key in memory with P-256 attributes, then asks Security about `Verify` with `kSecKeyAlgorithmECDSASignatureMessageX962SHA256`. A successful result writes exactly zero or one. Invalid curve coordinates or another Security import failure may map to `PLATFORM_ERROR`; temporary Core Foundation allocation failure maps to `RESOURCE_EXHAUSTED`. Neither failure leaves a nonzero output

A valid non-iOS call returns `UNSUPPORTED` and leaves output zero. A caught panic returns `PANIC` with zero output. No pointer is retained. The call is synchronous, with no added main-thread rule or thread-safety promise

The API is available from iOS 10.0 and needs no permission, usage-description key, or entitlement. It does not create or use private key material, persist a key, access Keychain or Secure Enclave, or verify data. `true` means only that Security reports the imported key suitable for the selected operation and algorithm

## Integration boundary

The header is opt-in. Root integration must add the target-iOS optional `framework-key-support` and `ios-key-support` dependencies plus the `ios-key-support` feature edge; gate the source module; add the export and ABI manifest entry; refresh Cargo.lock; run and add both F23 gates to macOS CI; update the aggregate binding plan and docs index. The link gate builds and inspects probes but does not execute them

See [B69's package plan](../../PLAN_IOS_KEY_SUPPORT.md), [D63's contract](../../PLAN_CAPABILITIES_KEY_SUPPORT.md), and [G63's validation evidence](../../PLAN_VALIDATION_IOS_KEY_SUPPORT.md)
