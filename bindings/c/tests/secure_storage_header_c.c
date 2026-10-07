#include "framework_ios_secure_storage.h"

_Static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_UNLOCK_REQUIRED == 1, "unlock policy bit");
_Static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_BOUND == 2, "device-bound policy bit");
_Static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_MASK == 3, "policy mask");

int secure_storage_header_c(void) {
    FrameworkStatus (*read_fn)(FrameworkStr, FrameworkStr, uint8_t *, FrameworkOwnedBuffer *, int32_t *) = framework_ios_secure_storage_read;
    FrameworkStatus (*store_fn)(FrameworkStr, FrameworkStr, FrameworkSlice, uint32_t, uint32_t *, int32_t *) = framework_ios_secure_storage_store;
    FrameworkStatus (*remove_fn)(FrameworkStr, FrameworkStr, uint8_t *, int32_t *) = framework_ios_secure_storage_remove;
    return read_fn != 0 && store_fn != 0 && remove_fn != 0 ? 0 : 1;
}
