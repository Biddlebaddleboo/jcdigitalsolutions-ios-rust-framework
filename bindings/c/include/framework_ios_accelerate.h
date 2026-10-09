#ifndef FRAMEWORK_IOS_ACCELERATE_H
#define FRAMEWORK_IOS_ACCELERATE_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-accelerate`. It exposes only B65's single-precision vector addition
 * through the public vDSP_vadd API, available from iOS 4.0. Linked Release
 * probes use iOS deployment minima 10.0 for device and 14.0 for Simulator;
 * these probe minima are distinct from the API floor.
 *
 * Length parameters count float elements, not bytes, and must all match. A
 * mismatch returns FRAMEWORK_STATUS_INVALID_ARGUMENT without reading inputs
 * or changing output. For a nonzero equal count on iOS, each input span must
 * name readable, aligned float elements and stay valid for the full call;
 * output must name writable, aligned elements for the full call and remain
 * disjoint from both inputs. Input spans may overlap each other. Null pointers
 * are valid only when all three lengths are zero. On iOS, equal zero lengths
 * return OK without calling Accelerate. On success, every output element is
 * overwritten. On invalid or unsupported status, output is unchanged. If a
 * caught Rust panic occurs, output contents are unspecified.
 *
 * The app must keep both inputs immutable and prevent unsynchronized output access
 * for the full call. The wrapper checks count equality, byte-length bounds,
 * non-nullness, alignment, address-range overflow, and output/input overlap,
 * but cannot prove that memory is valid, readable, writable, or live. No
 * pointer is retained after return. The non-iOS stub checks metadata, then
 * returns FRAMEWORK_STATUS_UNSUPPORTED without a data-byte read or write
 *
 * No main-thread rule or thread-safety guarantee is added. This API makes no
 * numerical-parity, certification, or performance
 * claim and does not expose other Accelerate APIs.
 */
FrameworkStatus framework_ios_accelerate_vector_add(
    const float *a,
    uint64_t a_length,
    const float *b,
    uint64_t b_length,
    float *output,
    uint64_t output_length);

#ifdef __cplusplus
}
#endif

#endif
