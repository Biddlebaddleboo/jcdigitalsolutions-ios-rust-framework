# iOS CommonCrypto SHA-256 C ABI

F19 proposes the opt-in `ios-crypto` feature and one synchronous C function, `framework_ios_crypto_sha256`, over B66's `ios_crypto::sha256` wrapper

The function accepts one borrowed `FrameworkSlice` whose length counts bytes and writes one fixed 32-byte digest into caller-owned output storage. On iOS, a nonzero input within the supported length limits must name readable bytes that stay immutable for the full call; empty input is valid, including a null data pointer with length zero. The output must name 32 writable bytes for the full call and be disjoint from nonempty input. Lengths above `UINT32_MAX` are rejected to match CommonCrypto's 32-bit `CC_LONG` parameter; a length that cannot fit a Rust slice is also invalid

After pointer-presence, length, address-range, and disjointness checks pass, the wrapper zeros all 32 output bytes before the `UINT32_MAX`, platform, or native-result checks. Malformed span metadata or overlap returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write to output. Success writes all 32 digest bytes and returns `FRAMEWORK_STATUS_OK`; oversized input returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`; a non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` with zero output; a native return-buffer mismatch maps to `FRAMEWORK_STATUS_PLATFORM_ERROR`; a caught Rust panic maps to `FRAMEWORK_STATUS_PANIC` with zero output

The app must keep input immutable and prevent unsynchronized output access for the full call. The wrapper checks pointer presence, byte-length bounds, checked address ranges, and input/output overlap, but cannot prove that memory is valid, readable, writable, or live. Neither pointer is retained after return

The underlying `CC_SHA256` API is available from iOS 2.0. The focused link probes use the Rust iOS target minima of iOS 10.0 for device and 14.0 for Simulator; these are probe deployment minima, not the native API floor. The expected direct C import is `libSystem.B.dylib`, with `_CC_SHA256` as the native symbol. C++ consumers may also import `libc++.1.dylib`

The C wrapper creates no C-owned allocation and retains no caller pointer. This scope does not add a portable `framework-crypto` contract or claim a replacement/default implementation, digest parity, certification, security review, or performance. It does not expose other algorithms, key operations, or native handles. See [B66](../../PLAN_IOS_CRYPTO.md) and [F19](../../PLAN_BINDINGS_COMPLETED_C_ABI.md)
