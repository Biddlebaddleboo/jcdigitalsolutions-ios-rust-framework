# PLAN_BINDINGS_F19.md — F19: iOS CommonCrypto SHA-256 C ABI

## Objective

Expose only B66's synchronous `ios_crypto::sha256` operation through one opt-in C function. Preserve its iOS-only, one-shot scope and exact 32-bit CommonCrypto input-length bound. Do not add a portable crypto contract, replacement/default claim, or other cryptographic operation

## Status

F19 is integrated in the root C ABI feature graph, public exports, ABI manifest, Cargo.lock, macOS CI, aggregate plan, and docs index. `sh bindings/c/check-ios-crypto.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact `libSystem.B.dylib`/`_CC_SHA256` imports, and minos 10.0/14.0 passed. No tests or consumer/probe binaries were executed

The pointer-contract audit adds static gate checks for byte ranges, span overlap, output zeroing, full-call lifetime, memory-validity limits, sync rules, and no pointer retention. The ABI manifest also records full-call input immutability and output synchronization preconditions; the gate asserts those ownership strings. Shell syntax, Rust format, C11/C++17 syntax, and these static checks passed. This audit did not run Cargo, links, tests, or probes

Root reran `sh bindings/c/check-ios-crypto.sh` after the source-order and manifest assertions were added. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact `libSystem.B.dylib`/`_CC_SHA256` imports, export parity, and minos 10.0/14.0 passed. Linked probes were not executed; no tests ran. The Apple linker emitted a nonfatal duplicate `-lSystem` warning

## Feasibility and backend contract

B66's `ios_crypto::sha256(input: &[u8]) -> Result<[u8; 32], Sha256Error>` calls Apple's public `CC_SHA256` C API. The backend rejects input above `u32::MAX` before FFI, supports empty input with a valid pointer and zero length, returns a fixed-size owned digest, and maps a native return pointer other than the supplied digest buffer to `Sha256Error::NativeFailure`. The inspected Xcode 26.6 / iOS SDK 26.5 declares `CC_LONG` as `uint32_t`, the SHA-256 digest length as 32, and `CC_SHA256` available from iOS 2.0. See [B66](PLAN_IOS_CRYPTO.md), Apple's [`CC_SHA256` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/CC_SHA256.3cc.html), and the [Common Crypto manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/Common%20Crypto.3cc.html)

This is a narrow C slice: input is borrowed only for the call; output is a fixed caller-owned 32-byte region; no pointer is retained; no object, handle, callback, or framework-owned buffer crosses C. The wrapper rejects input/output overlap before it creates a Rust input slice or writes output. The app must keep input immutable and prevent unsynchronized output access for the full call. The wrapper checks pointer presence, byte-length bounds, checked address ranges, and input/output overlap, but cannot prove that memory is valid, readable, writable, or live. The direct native dependency is `libSystem`, not a separate CommonCrypto framework. Device and Simulator link-probe minima are 10.0 and 14.0, distinct from the iOS 2.0 native API floor

## Exact C contract

- Optional Cargo feature: `ios-crypto`, backed only by an optional target-iOS dependency on `ios-crypto`
- Header: `framework_ios_crypto.h`
- Export: `FrameworkStatus framework_ios_crypto_sha256(FrameworkSlice input, uint8_t *out_digest)`
- Digest size: exactly 32 bytes, named `FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE`
- Input: borrowed bytes; on iOS, a nonzero length within the supported limits must name readable bytes that stay immutable for the full call. Null is valid only at length zero. Empty input is valid. Length above `UINT32_MAX` or above Rust's maximum slice length returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`
- Output: required caller-owned writable 32-byte region for the full call, disjoint from nonempty input, never retained. After pointer-presence, length, address-range, and non-overlap checks pass, initialize all 32 bytes to zero before the `UINT32_MAX`, platform, or native-result checks. Malformed span metadata or overlap returns invalid without a write to output
- Synchronization and memory validity: the app keeps input immutable and prevents unsynchronized output access for the full call. The wrapper checks pointer presence, byte-length bounds, checked address ranges, and input/output overlap, but cannot prove that memory is valid, readable, writable, or live
- Status: successful digest `OK`; invalid pointer metadata, overlap, or oversized input `INVALID_ARGUMENT`; valid non-iOS request `UNSUPPORTED` with zero output; native digest-buffer return mismatch `PLATFORM_ERROR`; caught Rust panic `PANIC` with zero output. The iOS 2.0 API floor is below both link-probe deployment minima, so no runtime unavailable branch is needed
- Threading: synchronous on the caller's thread; adds no main-thread rule or thread-safety guarantee
- Ownership: no allocation or native pointer crosses C; no `FrameworkOwnedBuffer` creator is added
- Limits: one SHA-256 operation only; no portable `framework-crypto` contract, replacement/default claim, digest parity/correctness evidence, certification, security review, performance claim, key operation, or other algorithm

## F19-owned files

- `bindings/c/src/ios_crypto.rs`
- `bindings/c/include/framework_ios_crypto.h`
- `bindings/c/check-ios-crypto.sh`
- `docs/bindings/ios-crypto.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, aggregate binding status, and documentation indexes

## Focused gate

`sh bindings/c/check-ios-crypto.sh` checks default/iOS/host feature trees, the source-level non-iOS `UNSUPPORTED` branch, host C11/C++17 links and archive import isolation, host/device/Simulator compilation and strict Clippy, Release archives, device/Simulator C11/C++17 compile/link, exact `libSystem.B.dylib` imports, `_CC_SHA256`, one C ABI export, and deployment minima 10.0/14.0. It passed after root wiring; C/C++ consumers and Apple probes were build-only and not executed. No tests are in scope

Do not alter B66 backend files
