#ifndef FRAMEWORK_IOS_CRYPTO_H
#define FRAMEWORK_IOS_CRYPTO_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

#define FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE 32u

/*
 * This header is opt-in through framework-c-api's Cargo feature `ios-crypto`.
 * It exposes only Apple's one-shot CC_SHA256 API, available from iOS 2.0.
 * The linked Release probes use iOS deployment minima 10.0 for device and
 * 14.0 for Simulator; these probe minima are distinct from the API floor.
 *
 * `input` is borrowed for this synchronous call and its length counts bytes.
 * On iOS, a nonzero input within the supported length limits must name
 * readable bytes that stay immutable for the full call. A null data pointer
 * is valid only with zero length; empty input is supported. Lengths above
 * UINT32_MAX are rejected because CC_LONG is 32-bit. A length that cannot fit
 * a Rust slice is also invalid.
 * `out_digest` must name 32 writable bytes for the full call and must not
 * overlap a nonempty input range. After pointer-presence, length, address-range,
 * and overlap checks pass, the wrapper writes zero to all 32 output bytes
 * before UINT32_MAX, platform, or native-result handling. Malformed span
 * metadata or overlap returns FRAMEWORK_STATUS_INVALID_ARGUMENT without
 * writing output. The app must keep input immutable and prevent unsynchronized
 * output access for the full call. The wrapper checks pointer presence,
 * byte-length bounds, checked address ranges, and input/output overlap, but
 * cannot prove that memory is valid, readable, writable, or live. Neither
 * pointer is retained after return; the C ABI adds no allocation.
 *
 * FRAMEWORK_STATUS_OK means the 32-byte digest was written. A valid non-iOS
 * call returns FRAMEWORK_STATUS_UNSUPPORTED with zero output. Invalid span,
 * overlap, null output, or length above UINT32_MAX returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT. A native return-buffer mismatch maps to
 * FRAMEWORK_STATUS_PLATFORM_ERROR. A caught Rust panic maps to
 * FRAMEWORK_STATUS_PANIC with zero output. The API adds no main-thread rule
 * or thread-safety guarantee. It makes no portable replacement, parity,
 * certification, security-review, or performance claim.
 */
FrameworkStatus framework_ios_crypto_sha256(
    FrameworkSlice input,
    uint8_t *out_digest);

#ifdef __cplusplus
}
#endif

#endif
