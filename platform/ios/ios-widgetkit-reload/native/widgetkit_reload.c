#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "WidgetKit swiftcall bridge needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse widget_center_metadata(uint64_t request)
    __asm__("_$s9WidgetKit0A6CenterCMa")
    __attribute__((swiftcall, weak_import));
extern void *widget_center_shared(void *metadata __attribute__((swift_context)))
    __asm__("_$s9WidgetKit0A6CenterC6sharedACvgZ")
    __attribute__((swiftcall, weak_import));
extern void widget_center_reload_all_timelines(void *center __attribute__((swift_context)))
    __asm__("_$s9WidgetKit0A6CenterC18reloadAllTimelinesyyFTj")
    __attribute__((swiftcall, weak_import));
extern void widget_center_invalidate_configuration_recommendations(
    void *center __attribute__((swift_context)))
    __asm__("_$s9WidgetKit0A6CenterC38invalidateConfigurationRecommendationsyyFTj")
    __attribute__((swiftcall, weak_import));

uint8_t framework_widgetkit_center_create(void **center_out) {
    if (center_out == NULL) {
        return 3;
    }
    *center_out = NULL;

    if (widget_center_metadata == NULL || widget_center_shared == NULL ||
        widget_center_reload_all_timelines == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = widget_center_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }

    void *const center = widget_center_shared((void *)response.metadata);
    if (center == NULL) {
        return 2;
    }

    *center_out = center;
    return 0;
}

uint8_t framework_widgetkit_reload_all_timelines(void *center) {
    if (center == NULL) {
        return 3;
    }
    if (widget_center_reload_all_timelines == NULL) {
        return 1;
    }

    widget_center_reload_all_timelines(center);
    return 0;
}

uint8_t framework_widgetkit_invalidate_configuration_recommendations(void *center) {
    if (center == NULL) {
        return 3;
    }
    if (widget_center_invalidate_configuration_recommendations == NULL) {
        return 1;
    }

    widget_center_invalidate_configuration_recommendations(center);
    return 0;
}
