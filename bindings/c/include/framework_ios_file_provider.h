#ifndef FRAMEWORK_IOS_FILE_PROVIDER_H
#define FRAMEWORK_IOS_FILE_PROVIDER_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosFileProviderOperation FrameworkIosFileProviderOperation;
typedef void (*FrameworkIosFileProviderReady)(void *context);

typedef struct FrameworkIosFileProviderResultV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkStatus status;
    uint8_t has_registered_domains;
    uint8_t reserved[3];
    FrameworkOwnedBuffer native_error_domain;
    int64_t native_error_code;
} FrameworkIosFileProviderResultV1;

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-file-provider`. It wraps only the iOS 11.0+ query
 * NSFileProviderManager.getDomainsWithCompletionHandler for registered
 * domains belonging to the calling app's own File Provider extension.
 * It does not expose File Provider domain identifiers, names, URLs, objects,
 * file access, provider state, or synchronization state.
 *
 * A successful start returns FRAMEWORK_STATUS_OK and one unique operation
 * handle. The Apple request starts during the call. The readiness callback
 * signals only that the host should poll; it carries no result and may run
 * before start returns or later on Apple's unspecified callback queue. It may
 * run concurrently with host work. Do not unwind or re-enter this API from the
 * callback. Schedule a later poll and serialize all calls on one handle. The C
 * ABI adds no main-thread rule.
 *
 * Poll initializes out_ready and the complete output record. Both output
 * pointers must be aligned, writable, mutually disjoint, and disjoint from the
 * live operation handle. Pass a writable result record whose
 * native_error_domain is empty on entry; after
 * every poll, destroy a non-empty native_error_domain with
 * framework_owned_buffer_destroy before reusing the record. Inspect semantic
 * fields only when the call returns FRAMEWORK_STATUS_OK and out_ready is 1.
 * The result's status is the terminal query outcome. On a native NSError,
 * status is PLATFORM_ERROR, native_error_domain contains its UTF-8 NSError
 * domain bytes (not a File Provider domain ID), and native_error_code is its
 * exact NSInteger code represented as int64_t. The returned domain buffer is
 * owned by the framework and must be destroyed exactly once.
 *
 * Apple does not expose cancellation for this request. Destroy detaches Rust
 * interest, suppresses future readiness notification, waits for an in-flight
 * readiness callback to return, clears the original handle slot, and drops
 * the C operation. The native request and its private callback state may live
 * until Apple's callback returns. Destroy does not cancel the Apple request.
 * Keep callback/context valid until the callback returns, a ready result is
 * consumed, or destroy returns. No callback can start after destroy returns.
 *
 * Non-iOS builds validate inputs, initialize valid outputs, then return
 * FRAMEWORK_STATUS_UNSUPPORTED; they create no mock operation and link no
 * Apple framework. A caught Rust panic returns FRAMEWORK_STATUS_PANIC; no
 * panic unwinds across C. The linked probes are build-only and are not run.
 */
FrameworkStatus framework_ios_file_provider_registered_domain_presence_start(
    FrameworkIosFileProviderReady completion,
    void *context,
    FrameworkIosFileProviderOperation **out_operation);

FrameworkStatus framework_ios_file_provider_operation_poll(
    FrameworkIosFileProviderOperation *operation,
    uint8_t *out_ready,
    FrameworkIosFileProviderResultV1 *out_result);

FrameworkStatus framework_ios_file_provider_operation_destroy(
    FrameworkIosFileProviderOperation **operation);

#ifdef __cplusplus
}
#endif

#endif
