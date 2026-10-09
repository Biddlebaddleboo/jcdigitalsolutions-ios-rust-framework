#ifndef FRAMEWORK_IOS_SECURE_STORAGE_H
#define FRAMEWORK_IOS_SECURE_STORAGE_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

#define FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_UNLOCK_REQUIRED UINT32_C(0x00000001)
#define FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_BOUND UINT32_C(0x00000002)
#define FRAMEWORK_IOS_SECURE_STORAGE_POLICY_MASK UINT32_C(0x00000003)

/*
 * These declarations are opt-in through the framework-c-api Cargo feature
 * `secure-storage`. On iOS, SecItemAdd without kSecAttrAccessGroup writes to the
 * app's default Keychain group. Read/update/remove queries omit that filter and
 * search all groups available to the app; update and remove may affect all matches.
 * This ABI exposes no group selector. On other targets the same symbols validate
 * arguments and return FRAMEWORK_STATUS_UNSUPPORTED.
 *
 * Inputs are borrowed for the synchronous call. Each non-empty pointer/length span must
 * refer to valid, readable memory and remain immutable for the full call; FrameworkStr
 * bytes must be UTF-8. Output storage must be valid, properly aligned, and writable for
 * the full call. Output ranges must be pairwise disjoint and disjoint from all non-empty
 * input spans; the API does not check overlap. Required output slots may be null; each
 * non-null output gets its default value before input checks. If a read output is null,
 * the other non-null read output gets its default before
 * FRAMEWORK_STATUS_INVALID_ARGUMENT. Read uses out_found = 0 and an empty out_secret;
 * store and remove use 0. On success, read
 * sets out_found to 1 for a present item, even when the secret is empty. out_secret must
 * not hold a live buffer on entry; pass each returned buffer to
 * framework_owned_buffer_destroy exactly once. Store leaves out_effective_policy_flags at
 * 0 on error. Remove leaves out_removed at 0 unless one or more matches existed and were
 * removed. A non-null native OSStatus output is optional; it is 0 before input checks and
 * for non-platform errors. A platform error returns FRAMEWORK_STATUS_PLATFORM_ERROR and
 * its exact nonzero OSStatus.
 */
FrameworkStatus framework_ios_secure_storage_read(
    FrameworkStr service,
    FrameworkStr item,
    uint8_t *out_found,
    FrameworkOwnedBuffer *out_secret,
    int32_t *out_native_os_status);

FrameworkStatus framework_ios_secure_storage_store(
    FrameworkStr service,
    FrameworkStr item,
    FrameworkSlice secret,
    uint32_t required_policy_flags,
    uint32_t *out_effective_policy_flags,
    int32_t *out_native_os_status);

FrameworkStatus framework_ios_secure_storage_remove(
    FrameworkStr service,
    FrameworkStr item,
    uint8_t *out_removed,
    int32_t *out_native_os_status);

#ifdef __cplusplus
}
#endif

#endif
