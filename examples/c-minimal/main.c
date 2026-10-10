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
    if (major != 1 || minor != 3) {
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
    const uint8_t input_bytes[] = {4, 5, 6};
    const FrameworkSlice input = {input_bytes, sizeof(input_bytes)};
    FrameworkOwnedBuffer copied = {NULL, 0, 0};
    const FrameworkStatus copy_status = framework_owned_buffer_copy(input, &copied);
    if (copy_status != FRAMEWORK_STATUS_OK) {
        fprintf(stderr, "owned-buffer copy failed with status %" PRIu32 "\n", copy_status);
        return 3;
    }
    if (copied.length != sizeof(input_bytes) || copied.data == NULL ||
        copied.data[0] != input_bytes[0] || copied.data[1] != input_bytes[1] ||
        copied.data[2] != input_bytes[2]) {
        framework_owned_buffer_destroy(&copied);
        fprintf(stderr, "owned-buffer copy returned unexpected bytes\n");
        return 4;
    }
    framework_owned_buffer_destroy(&copied);
    framework_owned_buffer_destroy(NULL);
    static const uint8_t diagnostic_bytes[] = "native diagnostic";
    const FrameworkStr diagnostic = {diagnostic_bytes, sizeof(diagnostic_bytes) - 1};
    FrameworkErrorDetailHandle detail = NULL;
    const FrameworkStatus create_status = framework_error_detail_create(
        FRAMEWORK_STATUS_PLATFORM_ERROR, diagnostic, &detail);
    if (create_status != FRAMEWORK_STATUS_OK || detail == NULL) {
        fprintf(stderr, "error-detail creation failed with status %" PRIu32 "\n", create_status);
        return 5;
    }
    FrameworkStatus detail_status = FRAMEWORK_STATUS_OK;
    FrameworkStr detail_message = {NULL, 0};
    const FrameworkStatus view_status = framework_error_detail_view(
        detail, &detail_status, &detail_message);
    if (view_status != FRAMEWORK_STATUS_OK || detail_status != FRAMEWORK_STATUS_PLATFORM_ERROR ||
        detail_message.length != sizeof(diagnostic_bytes) - 1 || detail_message.data == NULL) {
        framework_error_detail_destroy(detail);
        fprintf(stderr, "error-detail view returned unexpected values\n");
        return 6;
    }
    for (uint64_t index = 0; index < detail_message.length; ++index) {
        if (detail_message.data[index] != diagnostic_bytes[index]) {
            framework_error_detail_destroy(detail);
            fprintf(stderr, "error-detail view returned unexpected text\n");
            return 7;
        }
    }
    framework_error_detail_destroy(detail);
    framework_error_detail_destroy(NULL);
    printf("framework ABI %" PRIu32 ".%" PRIu32 "\n", major, minor);
    return 0;
}
