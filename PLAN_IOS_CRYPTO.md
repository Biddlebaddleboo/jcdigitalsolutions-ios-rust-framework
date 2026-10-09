# PLAN_IOS_CRYPTO.md — Workstream B66: CommonCrypto SHA-256

## Status

B66 adds one iOS-only safe wrapper around `CC_SHA256`; no portable crypto facade or Rust digest
implementation is included

The focused package checks pass in a temporary standalone workspace copied from the root source:
device and Simulator `cargo check --all-targets`, strict `cargo clippy --all-targets -- -D
warnings`, device `cargo doc --no-deps`, and `check-link-imports.sh`. The Release consumers import
only `libSystem.B.dylib`, retain undefined symbol `_CC_SHA256`, and report deployment versions
10.0 (device) and 14.0 (Simulator). Probes were built and inspected, never run. Root `--locked`
checks remain pending the parent update to `Cargo.lock`

## SDK and API evidence

- The inspected Xcode 26.6 / iOS SDK 26.5 public `CommonCrypto/CommonDigest.h` defines
  `CC_LONG` as `uint32_t`, declares `CC_SHA256(const void *, CC_LONG, unsigned char *)`, defines
  `CC_SHA256_DIGEST_LENGTH` as 32, and marks `CC_SHA256` available from iOS 2.0. In the inspected
  SDK, these are at `CommonCrypto/CommonDigest.h:63`, `:193`, and `:205-206`
- The inspected iPhoneOS SDK `libSystem.tbd:125` exports `_CC_SHA256`; the C function is linked
  from `libSystem`, not a separate framework
- Apple's public [`CC_SHA256` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/CC_SHA256.3cc.html)
  describes the one-shot digest signature; the [`Common Crypto` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/Common%20Crypto.3cc.html)
  identifies Common Crypto as a `libSystem` digest library. The inspected SDK header specifies
  that one-shot functions write to and return the caller-provided `md` buffer
- Device and Simulator link probes use Rust 1.94.1's supported iOS target minimums of 10.0 and
  14.0. These are link-probe floors, not the API availability declaration; the crate sets no
  deployment target

## Public surface and safety

- `ios_crypto::sha256(input: &[u8]) -> Result<[u8; 32], Sha256Error>` returns an owned fixed-size
  digest
- `Sha256Error::InputTooLarge` rejects inputs above `u32::MAX` before FFI, preserving the exact
  `CC_LONG` length bound
- `Sha256Error::NativeFailure` covers a native return pointer that is not the provided digest
  buffer
- Empty input calls the one-shot API with zero length and a valid pointer; the Rust wrapper creates no heap-backed
  input/output buffer and CommonCrypto retains neither input nor output pointer. CommonCrypto
  internal allocation behavior is outside this contract
- The wrapper adds no main-thread check or cross-call state and makes no added thread-safety claim

## Scope and non-goals

- The crate is available only on iOS and has no non-iOS stub
- There is no `framework-crypto` portable contract and no assertion that this is a framework
  replacement or default implementation
- No parity/correctness test, certification, security review, performance result, or benchmark is
  claimed
- No other crypto algorithm or key operation is included
- No permission, entitlement, Info.plist key, UI, network, or service behavior is in scope

## Validation

`check-link-imports.sh` builds but does not execute a minimal Release consumer for iOS arm64 and
Simulator arm64, then inspects `libSystem.B.dylib`, `_CC_SHA256`, and deployment metadata. See
`PLAN_CAPABILITIES_CRYPTO.md`; no tests are required for this partial Apple-backed wrapper. In the
integrated checkout, locked device and Simulator checks, strict Clippy for both targets, and the
link/import script passed. The probes import only `libSystem.B.dylib`, retain `_CC_SHA256`, and
report minos 10.0/device and 14.0/Simulator; they were built and inspected, not executed. Rustfmt,
rustdoc, docs-check, zero-Swift-source, and diff checks passed. No tests or parity/performance
checks ran, and no passing CI run is recorded. The host is Xcode 26.6 / SDK 26.5, below the Xcode
27.x plan baseline
