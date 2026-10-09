#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "ActivityKit swiftcall thunk needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse activity_authorization_info_metadata(uint64_t request)
    __asm__("_$s11ActivityKit0A17AuthorizationInfoCMa")
    __attribute__((swiftcall, weak_import));
extern void *activity_authorization_info_init(void *metadata __attribute__((swift_context)))
    __asm__("_$s11ActivityKit0A17AuthorizationInfoCACycfC")
    __attribute__((swiftcall, weak_import));
extern bool activity_authorization_info_are_activities_enabled(
    void *info __attribute__((swift_context)))
    __asm__("_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg")
    __attribute__((swiftcall, weak_import));

uint8_t framework_activity_authorization_info_create(void **info_out) {
    if (info_out == NULL) {
        return 3;
    }
    *info_out = NULL;

    if (activity_authorization_info_metadata == NULL || activity_authorization_info_init == NULL ||
        activity_authorization_info_are_activities_enabled == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = activity_authorization_info_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }

    void *const info = activity_authorization_info_init((void *)response.metadata);
    if (info == NULL) {
        return 2;
    }

    *info_out = info;
    return 0;
}

uint8_t framework_activity_authorization_info_are_activities_enabled(
    void *info,
    uint8_t *enabled_out) {
    if (info == NULL || enabled_out == NULL) {
        return 3;
    }
    if (activity_authorization_info_are_activities_enabled == NULL) {
        return 1;
    }

    *enabled_out = activity_authorization_info_are_activities_enabled(info) ? 1 : 0;
    return 0;
}
