#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

struct SwiftMetadataResponse {
    void *metadata;
    uintptr_t state;
};

typedef void (*SwiftDestroyWitness)(void *, const void *);

struct SwiftValueWitnessTable {
    const void *initialize_buffer_with_copy_of_buffer;
    SwiftDestroyWitness destroy;
    const void *initialize_with_copy;
    const void *assign_with_copy;
    const void *initialize_with_take;
    const void *assign_with_take;
    const void *get_enum_tag_single_payload;
    const void *store_enum_tag_single_payload;
    uintptr_t size;
    uintptr_t stride;
    uint32_t flags;
    uint32_t extra_inhabitant_count;
    const void *get_enum_tag;
    const void *destructive_project_enum_data;
    const void *destructive_inject_enum_tag;
};

#define SWIFT_VALUE_WITNESS_ALIGNMENT_MASK UINT32_C(0x000000ff)

_Static_assert(sizeof(void *) == 8, "RealityFoundation swiftcall thunk needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, destroy) == 8, "Swift destroy witness offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, size) == 64, "Swift value size offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, flags) == 80, "Swift value flags offset changed");
_Static_assert(sizeof(int64_t) == sizeof(intptr_t), "Swift Int snapshot needs 64-bit Apple targets");

extern struct SwiftMetadataResponse photogrammetry_session_metadata(uint64_t request)
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionCMa")
    __attribute__((swiftcall));
extern bool photogrammetry_session_is_supported(void *metadata __attribute__((swift_context)))
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionC11isSupportedSbvgZ")
    __attribute__((swiftcall));

extern struct SwiftMetadataResponse photogrammetry_limits_metadata(uint64_t request)
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionC6LimitsVMa")
    __attribute__((swiftcall));
extern void photogrammetry_session_limits_get(
    void *limits __attribute__((swift_indirect_result)),
    void *metadata __attribute__((swift_context)))
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionC6limitsAC6LimitsVvgZ")
    __attribute__((swiftcall));
extern int64_t photogrammetry_limits_maximum_input_image_dimension(
    void *limits __attribute__((swift_context)))
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumInputImageDimensionSivg")
    __attribute__((swiftcall));
extern int64_t photogrammetry_limits_maximum_number_of_input_images(
    void *limits __attribute__((swift_context)))
    __asm__("_$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumNumberOfInputImagesSivg")
    __attribute__((swiftcall));

uint8_t framework_photogrammetry_session_is_supported(uint8_t *supported_out) {
    if (supported_out == NULL) {
        return 2;
    }
    *supported_out = 0;

    const struct SwiftMetadataResponse response = photogrammetry_session_metadata(0);
    if (response.metadata == NULL) {
        return 1;
    }

    *supported_out = photogrammetry_session_is_supported(response.metadata) ? 1 : 0;
    return 0;
}

uint8_t framework_photogrammetry_session_limits(
    int64_t *maximum_input_image_dimension_out,
    int64_t *maximum_number_of_input_images_out) {
    if (maximum_input_image_dimension_out == NULL || maximum_number_of_input_images_out == NULL) {
        return 2;
    }
    *maximum_input_image_dimension_out = 0;
    *maximum_number_of_input_images_out = 0;

    const struct SwiftMetadataResponse limits_response = photogrammetry_limits_metadata(0);
    const struct SwiftMetadataResponse session_response = photogrammetry_session_metadata(0);
    if (limits_response.metadata == NULL || session_response.metadata == NULL) {
        return 1;
    }

    const struct SwiftValueWitnessTable *const witnesses =
        ((const struct SwiftValueWitnessTable *const *)limits_response.metadata)[-1];
    if (witnesses == NULL || witnesses->size == 0 || witnesses->destroy == NULL) {
        return 3;
    }

    uintptr_t alignment =
        (uintptr_t)(witnesses->flags & SWIFT_VALUE_WITNESS_ALIGNMENT_MASK) + 1;
    if (alignment == 0 || (alignment & (alignment - 1)) != 0) {
        return 3;
    }
    if (alignment < sizeof(void *)) {
        alignment = sizeof(void *);
    }

    void *limits = NULL;
    if (posix_memalign(&limits, alignment, witnesses->size) != 0) {
        return 4;
    }

    photogrammetry_session_limits_get(limits, (void *)session_response.metadata);
    const int64_t maximum_input_image_dimension =
        photogrammetry_limits_maximum_input_image_dimension(limits);
    const int64_t maximum_number_of_input_images =
        photogrammetry_limits_maximum_number_of_input_images(limits);
    witnesses->destroy(limits, limits_response.metadata);
    free(limits);

    *maximum_input_image_dimension_out = maximum_input_image_dimension;
    *maximum_number_of_input_images_out = maximum_number_of_input_images;
    return 0;
}
