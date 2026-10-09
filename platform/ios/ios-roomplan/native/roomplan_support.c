#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "RoomPlan Swift ABI thunk supports 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response layout changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse room_capture_session_metadata(uint64_t request)
    __asm__("_$s8RoomPlan0A14CaptureSessionCMa") __attribute__((swiftcall));
extern bool room_capture_session_is_supported(void *metadata __attribute__((swift_context)))
    __asm__("_$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ") __attribute__((swiftcall));

uint8_t framework_roomplan_is_supported(void) {
    const struct SwiftMetadataResponse response = room_capture_session_metadata(0);
    if (response.metadata == NULL) {
        return 0;
    }
    return room_capture_session_is_supported(response.metadata) ? 1 : 0;
}
