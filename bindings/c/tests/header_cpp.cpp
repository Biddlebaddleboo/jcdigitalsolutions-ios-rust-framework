#include "framework.h"

#include <cstddef>
#include <type_traits>

extern "C" {
using FrameworkExpectedCompletionCallback = void (*)(void *,
                                                     FrameworkOperationHandle,
                                                     FrameworkStatus,
                                                     FrameworkSlice);
using FrameworkExpectedOptionsValidator =
    FrameworkStatus (*)(const FrameworkOptionsV1 *);
}

static_assert(sizeof(FrameworkStatus) == 4);
static_assert(sizeof(FrameworkOperationHandle) == 8);
static_assert(sizeof(FrameworkErrorHandle) == 8);
static_assert(std::is_same_v<FrameworkCompletionCallback,
                             FrameworkExpectedCompletionCallback>);
static_assert(sizeof(FrameworkCompletionCallback) ==
              sizeof(FrameworkExpectedCompletionCallback));
static_assert(alignof(FrameworkCompletionCallback) ==
              alignof(FrameworkExpectedCompletionCallback));
static_assert(sizeof(FrameworkOptionsV1) == 16);
static_assert(alignof(FrameworkOptionsV1) == 4);
static_assert(offsetof(FrameworkOptionsV1, struct_size) == 0);
static_assert(offsetof(FrameworkOptionsV1, abi_version) == 4);
static_assert(offsetof(FrameworkOptionsV1, flags) == 8);
static_assert(offsetof(FrameworkOptionsV1, reserved) == 12);
static_assert(std::is_standard_layout_v<FrameworkOptionsV1>);
static_assert(std::is_same_v<decltype(&framework_options_v1_validate),
                             FrameworkExpectedOptionsValidator>);
static_assert(std::is_standard_layout_v<FrameworkSlice>);
static_assert(std::is_standard_layout_v<FrameworkOwnedBuffer>);
