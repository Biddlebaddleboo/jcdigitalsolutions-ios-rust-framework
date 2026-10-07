#include "framework.h"

#include <stddef.h>

_Static_assert(sizeof(FrameworkStatus) == 4, "FrameworkStatus size");
_Static_assert(sizeof(FrameworkOperationHandle) == 8, "FrameworkOperationHandle size");
_Static_assert(sizeof(FrameworkErrorHandle) == 8, "FrameworkErrorHandle size");
_Static_assert(FRAMEWORK_STATUS_OK == 0, "OK status");
_Static_assert(FRAMEWORK_STATUS_INVALID_ARGUMENT == 1, "invalid argument status");
_Static_assert(FRAMEWORK_STATUS_UNSUPPORTED == 2, "unsupported status");
_Static_assert(FRAMEWORK_STATUS_UNAVAILABLE == 3, "unavailable status");
_Static_assert(FRAMEWORK_STATUS_PERMISSION_DENIED == 4, "permission status");
_Static_assert(FRAMEWORK_STATUS_CANCELLED == 5, "cancelled status");
_Static_assert(FRAMEWORK_STATUS_TIMEOUT == 6, "timeout status");
_Static_assert(FRAMEWORK_STATUS_NOT_FOUND == 7, "not found status");
_Static_assert(FRAMEWORK_STATUS_ALREADY_EXISTS == 8, "already exists status");
_Static_assert(FRAMEWORK_STATUS_RESOURCE_EXHAUSTED == 9, "resource status");
_Static_assert(FRAMEWORK_STATUS_PLATFORM_ERROR == 10, "platform status");
_Static_assert(FRAMEWORK_STATUS_INTERNAL_ERROR == 11, "internal status");
_Static_assert(FRAMEWORK_STATUS_PANIC == 12, "panic status");
_Static_assert(sizeof(FrameworkCompletionCallback) == sizeof(void (*)(void)), "callback size");
_Static_assert(_Alignof(FrameworkCompletionCallback) == _Alignof(void (*)(void)), "callback align");
_Static_assert(sizeof(FrameworkOptionsV1) == 16, "FrameworkOptionsV1 size");
_Static_assert(_Alignof(FrameworkOptionsV1) == 4, "FrameworkOptionsV1 align");
_Static_assert(offsetof(FrameworkOptionsV1, struct_size) == 0, "struct_size offset");
_Static_assert(offsetof(FrameworkOptionsV1, abi_version) == 4, "abi_version offset");
_Static_assert(offsetof(FrameworkOptionsV1, flags) == 8, "flags offset");
_Static_assert(offsetof(FrameworkOptionsV1, reserved) == 12, "reserved offset");
