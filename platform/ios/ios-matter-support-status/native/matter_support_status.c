#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

_Static_assert(sizeof(void *) == 8, "MatterSupport swiftcall thunk needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "MatterSupport swiftcall thunk needs 64-bit Apple targets");

extern bool matter_add_device_request_is_supported_swift(void)
    __asm__("_$s13MatterSupport0A16AddDeviceRequestV11isSupportedSbvgZ")
    __attribute__((swiftcall, weak_import));

uint8_t framework_matter_add_device_request_is_supported(uint8_t *supported_out) {
    if (supported_out == NULL) {
        return 2;
    }
    *supported_out = 0;

    if (matter_add_device_request_is_supported_swift == NULL) {
        return 1;
    }

    *supported_out = matter_add_device_request_is_supported_swift() ? 1 : 0;
    return 0;
}
