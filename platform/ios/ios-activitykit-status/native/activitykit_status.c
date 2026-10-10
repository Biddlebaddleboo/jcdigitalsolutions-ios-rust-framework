#include <stdbool.h>
#include <stdint.h>
#include "swift_abi_runtime.h"

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
