#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "swift_abi_runtime.h"

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

    const struct SwiftValueWitnessTable *witnesses = NULL;
    uintptr_t value_size = 0;
    uintptr_t alignment = 0;
    if (!swift_abi_value_storage_layout_from_metadata(
            limits_response.metadata,
            &witnesses,
            &value_size,
            &alignment)) {
        return 3;
    }

    void *limits = NULL;
    if (!swift_abi_allocate_value_storage(&limits, alignment, value_size)) {
        return 4;
    }

    photogrammetry_session_limits_get(limits, (void *)session_response.metadata);
    const int64_t maximum_input_image_dimension =
        photogrammetry_limits_maximum_input_image_dimension(limits);
    const int64_t maximum_number_of_input_images =
        photogrammetry_limits_maximum_number_of_input_images(limits);
    swift_abi_destroy_and_free_value(witnesses, limits, limits_response.metadata);

    *maximum_input_image_dimension_out = maximum_input_image_dimension;
    *maximum_number_of_input_images_out = maximum_number_of_input_images;
    return 0;
}
