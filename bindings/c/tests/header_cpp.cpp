#include "framework.h"

#include <cstddef>
#include <type_traits>

static_assert(sizeof(FrameworkStatus) == 4);
static_assert(sizeof(FrameworkOperationHandle) == 8);
static_assert(sizeof(FrameworkErrorHandle) == 8);
static_assert(sizeof(FrameworkCompletionCallback) == sizeof(void *));
static_assert(sizeof(FrameworkOptionsV1) == 16);
static_assert(alignof(FrameworkOptionsV1) == 4);
static_assert(offsetof(FrameworkOptionsV1, reserved) == 12);
static_assert(std::is_standard_layout_v<FrameworkSlice>);
static_assert(std::is_standard_layout_v<FrameworkOwnedBuffer>);
