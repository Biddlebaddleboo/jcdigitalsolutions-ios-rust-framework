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
 * `secure-storage`. On iOS they use the app's default Keychain group. On other
 * targets the same symbols validate arguments and return FRAMEWORK_STATUS_UNSUPPORTED.
 *
 * Inputs are borrowed for the synchronous call. Each non-empty pointer/length span
 * must reference readable memory; FrameworkStr bytes must be UTF-8. Output pointers
 * must be writable and must not alias. A non-null native OSStatus output is optional.
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
