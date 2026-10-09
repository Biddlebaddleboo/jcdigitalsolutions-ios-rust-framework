#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "ProximityReader swiftcall thunk supports 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response layout changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse payment_card_reader_metadata(uint64_t request)
    __asm__("_$s15ProximityReader011PaymentCardB0CMa") __attribute__((swiftcall));
extern bool payment_card_reader_is_supported(void *metadata __attribute__((swift_context)))
    __asm__("_$s15ProximityReader011PaymentCardB0C11isSupportedSbvgZ") __attribute__((swiftcall));

uint8_t framework_proximity_reader_tap_to_pay_device_model_supported(void) {
    const struct SwiftMetadataResponse response = payment_card_reader_metadata(0);
    if (response.metadata == NULL) {
        return 0;
    }
    return payment_card_reader_is_supported(response.metadata) ? 1 : 0;
}
