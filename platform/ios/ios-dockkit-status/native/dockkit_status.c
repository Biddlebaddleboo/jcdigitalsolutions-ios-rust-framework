#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "DockKit swiftcall bridge needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse dock_accessory_manager_metadata(uint64_t request)
    __asm__("_$s7DockKit0A16AccessoryManagerCMa")
    __attribute__((swiftcall, weak_import));
extern void *dock_accessory_manager_shared(void *metadata __attribute__((swift_context)))
    __asm__("_$s7DockKit0A16AccessoryManagerC6sharedACvgZ")
    __attribute__((swiftcall, weak_import));
extern bool dock_accessory_manager_is_system_tracking_enabled(
    void *manager __attribute__((swift_context)))
    __asm__("_$s7DockKit0A16AccessoryManagerC23isSystemTrackingEnabledSbvgTj")
    __attribute__((swiftcall, weak_import));

uint8_t framework_dockkit_manager_create(void **manager_out) {
    if (manager_out == NULL) {
        return 3;
    }
    *manager_out = NULL;

    if (dock_accessory_manager_metadata == NULL || dock_accessory_manager_shared == NULL ||
        dock_accessory_manager_is_system_tracking_enabled == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = dock_accessory_manager_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }

    void *const manager = dock_accessory_manager_shared((void *)response.metadata);
    if (manager == NULL) {
        return 2;
    }

    *manager_out = manager;
    return 0;
}

uint8_t framework_dockkit_is_system_tracking_enabled(void *manager, uint8_t *enabled_out) {
    if (manager == NULL || enabled_out == NULL) {
        return 3;
    }
    if (dock_accessory_manager_is_system_tracking_enabled == NULL) {
        return 1;
    }

    *enabled_out = dock_accessory_manager_is_system_tracking_enabled(manager) ? 1 : 0;
    return 0;
}
