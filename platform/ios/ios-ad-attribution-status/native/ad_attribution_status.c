#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

_Static_assert(sizeof(void *) == 8, "AdAttributionKit swiftcall thunk needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "AdAttributionKit swiftcall thunk needs 64-bit Apple targets");

extern bool app_impression_is_supported_swift(void)
    __asm__("_$s16AdAttributionKit13AppImpressionV11isSupportedSbvgZ")
    __attribute__((swiftcall, weak_import));

uint8_t framework_ad_attribution_app_impression_is_supported(uint8_t *supported_out) {
    if (supported_out == NULL) {
        return 2;
    }
    *supported_out = 0;

    if (app_impression_is_supported_swift == NULL) {
        return 1;
    }

    *supported_out = app_impression_is_supported_swift() ? 1 : 0;
    return 0;
}
