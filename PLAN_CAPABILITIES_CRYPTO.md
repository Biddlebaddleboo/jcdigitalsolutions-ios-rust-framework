# PLAN_CAPABILITIES_CRYPTO.md — Workstream D60: CommonCrypto SHA-256 Partial

## Status

D60 adds no portable `framework-crypto` contract. B66 is an iOS-only partial slice of capability
row 018: a safe Rust wrapper around Apple's one-shot CommonCrypto `CC_SHA256` API

## Objective

Expose one fixed-output SHA-256 operation over borrowed input bytes. Delegate the digest operation
to Apple's public C API; do not add a Rust implementation or claim replacement status

## API and behavior

- `ios-crypto` exposes `sha256(&[u8]) -> Result<[u8; 32], Sha256Error>` on iOS only
- `Sha256Error::InputTooLarge` is returned before FFI if input length exceeds `u32::MAX`, the
  inspected SDK's `CC_LONG` width
- `Sha256Error::NativeFailure` is returned if the native API does not return the supplied
  32-byte output buffer
- Empty input is passed to `CC_SHA256` with length zero; the returned digest is owned by Rust
- The Rust wrapper creates no heap-backed input/output buffer, retains no pointers, and has no
  thread-affinity requirement; CommonCrypto internal allocation behavior is outside this contract
- The iOS 2.0 availability declaration is the API floor. Rust 1.94.1 device and Simulator link
  probes use iOS 10.0 and iOS 14.0, respectively; the crate sets no deployment target

## Boundaries

- No portable contract, other-platform implementation, or non-iOS fallback is added
- This wrapper does not replace/default over an Apple implementation; it directly calls Apple code
- No parity, certification, security review, or performance claim is made
- No key generation, key management, secure storage, HMAC, streaming digest, or other algorithm
  is included
- No permission, Info.plist key, entitlement, UI, network, or service behavior is involved

## Validation and handoff

- Compile and strict-Clippy the iOS device and Simulator targets
- Inspect the Release consumer for only `libSystem.B.dylib`, symbol `_CC_SHA256`, and the expected
  deployment `minos`
- Run formatting, rustdoc, documentation-index, zero-Swift-source, and diff gates
- Do not execute Apple link probes or claim a digest-parity test or live-device result
