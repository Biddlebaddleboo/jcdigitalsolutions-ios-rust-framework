#include "framework.h"

#include <inttypes.h>
#include <stdio.h>

typedef struct FrameworkOptionsV1Future {
    uint32_t struct_size;
    uint32_t abi_version;
    uint32_t flags;
    uint32_t reserved;
    uint32_t future_field;
} FrameworkOptionsV1Future;

int main(void) {
    const uint64_t version = framework_abi_version();
    const uint32_t major = (uint32_t)(version >> 32);
    const uint32_t minor = (uint32_t)version;
    if (major != 1 || minor != 1) {
        fprintf(stderr, "unsupported framework ABI: %" PRIu32 ".%" PRIu32 "\n", major, minor);
        return 1;
    }
    const FrameworkOptionsV1Future future_options = {
        .struct_size = (uint32_t)sizeof(FrameworkOptionsV1Future),
        .abi_version = major,
        .flags = 0,
        .reserved = 0,
        .future_field = 0
    };
    const FrameworkStatus options_status = framework_options_v1_validate(
        (const FrameworkOptionsV1 *)(const void *)&future_options);
    if (options_status != FRAMEWORK_STATUS_OK) {
        fprintf(stderr, "invalid future-sized options header\n");
        return 2;
    }
    framework_owned_buffer_destroy(NULL);
    printf("framework ABI %" PRIu32 ".%" PRIu32 "\n", major, minor);
    return 0;
}
