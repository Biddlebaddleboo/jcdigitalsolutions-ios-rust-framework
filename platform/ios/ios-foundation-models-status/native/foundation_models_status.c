#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

_Static_assert(sizeof(void *) == 8, "Foundation Models swiftcall bridge needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");

extern struct SwiftMetadataResponse system_language_model_metadata(uint64_t request)
    __asm__("_$s16FoundationModels19SystemLanguageModelCMa")
    __attribute__((swiftcall, weak_import));
extern void *system_language_model_default(void *metadata __attribute__((swift_context)))
    __asm__("_$s16FoundationModels19SystemLanguageModelC7defaultACvgZ")
    __attribute__((swiftcall, weak_import));
extern bool system_language_model_is_available(void *model __attribute__((swift_context)))
    __asm__("_$s16FoundationModels19SystemLanguageModelC11isAvailableSbvg")
    __attribute__((swiftcall, weak_import));

uint8_t framework_system_language_model_default_create(void **model_out) {
    if (model_out == NULL) {
        return 3;
    }
    *model_out = NULL;

    if (system_language_model_metadata == NULL || system_language_model_default == NULL ||
        system_language_model_is_available == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = system_language_model_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }

    void *const model = system_language_model_default((void *)response.metadata);
    if (model == NULL) {
        return 2;
    }

    *model_out = model;
    return 0;
}

uint8_t framework_system_language_model_is_available(void *model, uint8_t *available_out) {
    if (model == NULL || available_out == NULL) {
        return 3;
    }
    if (system_language_model_is_available == NULL) {
        return 1;
    }

    *available_out = system_language_model_is_available(model) ? 1 : 0;
    return 0;
}
