#include "framework_ios_secure_storage.h"

#include <type_traits>

static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_UNLOCK_REQUIRED == 1);
static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_BOUND == 2);
static_assert(FRAMEWORK_IOS_SECURE_STORAGE_POLICY_MASK == 3);
static_assert(std::is_same_v<decltype(&framework_ios_secure_storage_read), FrameworkStatus (*)(FrameworkStr, FrameworkStr, uint8_t *, FrameworkOwnedBuffer *, int32_t *)>);
static_assert(std::is_same_v<decltype(&framework_ios_secure_storage_store), FrameworkStatus (*)(FrameworkStr, FrameworkStr, FrameworkSlice, uint32_t, uint32_t *, int32_t *)>);
static_assert(std::is_same_v<decltype(&framework_ios_secure_storage_remove), FrameworkStatus (*)(FrameworkStr, FrameworkStr, uint8_t *, int32_t *)>);

int secure_storage_header_cpp() { return 0; }
