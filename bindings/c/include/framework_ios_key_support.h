#ifndef FRAMEWORK_IOS_KEY_SUPPORT_H
#define FRAMEWORK_IOS_KEY_SUPPORT_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-key-support`. The iOS Security API floor is iOS 10.0. The linked
 * Release probes use target minima 10.0 for device and 14.0 for Simulator;
 * those probe minima are distinct from the API floor.
 *
 * `x963_public_key` must contain exactly 65 readable bytes in ANSI X9.63
 * uncompressed P-256 form. The bytes must remain immutable for this
 * synchronous call. A non-null `out_supported` must address valid, properly
 * aligned writable memory for one byte during this synchronous call and must
 * be disjoint from the input. The client must prevent unsynchronized concurrent access.
 * The API checks pointer-range arithmetic and overlap but
 * cannot validate the memory; it does not retain the output address. The
 * wrapper initializes a structurally valid,
 * disjoint output to zero before marker or platform handling. Invalid span
 * metadata or overlap returns FRAMEWORK_STATUS_INVALID_ARGUMENT without
 * writing output. On iOS, a first byte other than 0x04 returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT with zero output.
 *
 * FRAMEWORK_STATUS_OK writes exactly zero or one. One means Security reports
 * the imported public key suitable for Verify with
 * ECDSASignatureMessageX962SHA256; zero means Security reports it unsuitable.
 * A valid non-iOS call returns FRAMEWORK_STATUS_UNSUPPORTED with zero output.
 * Security import and allocation errors use the stable framework status map;
 * a caught Rust panic returns FRAMEWORK_STATUS_PANIC with zero output.
 *
 * The query imports the caller's public key only for this query. It does not
 * verify a signature, create/use private key material, persist a key, access
 * Keychain or Secure Enclave, or promise that a later verification call will
 * succeed. It requires no permission, usage-description key, or entitlement.
 * The API adds no main-thread rule or thread-safety guarantee.
 */
FrameworkStatus framework_ios_key_support_p256_ecdsa_sha256_message_supported(
    FrameworkSlice x963_public_key,
    uint8_t *out_supported);

#ifdef __cplusplus
}
#endif

#endif
