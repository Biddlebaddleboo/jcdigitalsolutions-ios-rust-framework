# iOS CommonCrypto SHA-256

`ios-crypto` is an iOS-only, no-std wrapper around Apple's one-shot `CC_SHA256` API. The sole
operation is
`ios_crypto::sha256(input: &[u8]) -> Result<[u8; 32], Sha256Error>`. It borrows input for the call
and returns an owned 32-byte digest

Input above `u32::MAX` bytes returns `Sha256Error::InputTooLarge` before FFI because CommonCrypto
uses a 32-bit `CC_LONG` length. `Sha256Error::NativeFailure` covers a native return pointer other
than the provided digest buffer. Empty input is passed with zero length and a valid pointer. The
Rust wrapper creates no heap-backed input/output buffer and retains no pointer; it adds no
main-thread check or cross-call state and makes no added thread-safety or CommonCrypto internal
allocation claim

The inspected iOS 26.5 SDK marks `CC_SHA256` available from iOS 2.0 and exports it from
`libSystem`. Apple's [`CC_SHA256` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/CC_SHA256.3cc.html)
documents the one-shot digest call, and the [`Common Crypto` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/Common%20Crypto.3cc.html)
identifies its library as `libSystem`. Rust 1.94.1 Release link probes use iOS 10.0 for device and iOS 14.0 for Simulator;
these are target link floors rather than API availability declarations. The crate sets no
deployment target

This is an Apple-backed partial slice of capability row 018, not a portable `framework-crypto`
contract or a Rust replacement/default. It makes no parity, certification, security-review, or
performance claim. It adds no key management, secure storage, HMAC, streaming digest, permission,
entitlement, Info.plist key, UI, network, or service behavior. The focused scope is in
`PLAN_CAPABILITIES_CRYPTO.md` and `PLAN_IOS_CRYPTO.md`; the package-local link check builds and
inspects but does not execute its consumers
