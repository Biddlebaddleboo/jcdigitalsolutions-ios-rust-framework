#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

struct SwiftValueWitnessTable {
    const void *witnesses[8];
    uint64_t size;
    uint64_t stride;
    uint32_t flags;
    uint32_t extra_inhabitant_count;
};

_Static_assert(sizeof(void *) == 8, "Family Controls Swift ABI bridge needs 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, size) == 64, "Swift VWT size offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, stride) == 72, "Swift VWT stride offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, flags) == 80, "Swift VWT flags offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, extra_inhabitant_count) == 84, "Swift VWT extra-inhabitant offset changed");

extern struct SwiftMetadataResponse family_controls_status_metadata(uint64_t request)
    __asm__("_$s14FamilyControls19AuthorizationStatusOMa")
    __attribute__((swiftcall, weak_import));
extern struct SwiftMetadataResponse family_controls_center_metadata(uint64_t request)
    __asm__("_$s14FamilyControls19AuthorizationCenterCMa")
    __attribute__((swiftcall, weak_import));
extern void *family_controls_center_shared(void *metadata __attribute__((swift_context)))
    __asm__("_$s14FamilyControls19AuthorizationCenterC6sharedACvgZ")
    __attribute__((swiftcall, weak_import));
extern void family_controls_center_authorization_status(
    void *status __attribute__((swift_indirect_result)),
    void *center __attribute__((swift_context)))
    __asm__("_$s14FamilyControls19AuthorizationCenterC19authorizationStatusAA0cF0OvgTj")
    __attribute__((swiftcall, weak_import));
extern int64_t family_controls_status_raw_value(void *status __attribute__((swift_context)))
    __asm__("_$s14FamilyControls19AuthorizationStatusO8rawValueSivg")
    __attribute__((swiftcall, weak_import));
extern void swift_release(void *object);

static uint8_t family_controls_allocate_value(uint64_t size, uint32_t flags, void **storage) {
    const uint32_t alignment_mask = flags & UINT32_C(0xff);
    const size_t alignment = (size_t)alignment_mask + 1;

    if (size == 0 || size > SIZE_MAX || (alignment_mask & (alignment_mask + 1)) != 0) {
        return 3;
    }

    if (alignment <= _Alignof(max_align_t)) {
        *storage = malloc((size_t)size);
        return *storage == NULL ? 4 : 0;
    }

    if (alignment < sizeof(void *) || (alignment & (alignment - 1)) != 0) {
        return 3;
    }

    return posix_memalign(storage, alignment, (size_t)size) == 0 ? 0 : 4;
}

uint8_t framework_family_controls_status_snapshot(int64_t *raw_value) {
    if (raw_value == NULL) {
        return 1;
    }

    if (family_controls_status_metadata == NULL || family_controls_center_metadata == NULL ||
        family_controls_center_shared == NULL ||
        family_controls_center_authorization_status == NULL ||
        family_controls_status_raw_value == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse status_response = family_controls_status_metadata(0);
    const struct SwiftMetadataResponse center_response = family_controls_center_metadata(0);
    if (status_response.metadata == NULL || center_response.metadata == NULL) {
        return 2;
    }

    const struct SwiftValueWitnessTable *const *vwt_location =
        (const struct SwiftValueWitnessTable *const *)status_response.metadata;
    const struct SwiftValueWitnessTable *vwt = vwt_location[-1];
    if (vwt == NULL || vwt->stride < vwt->size || vwt->witnesses[1] == NULL) {
        return 3;
    }

    void *status_storage = NULL;
    const uint8_t allocation_result = family_controls_allocate_value(vwt->size, vwt->flags, &status_storage);
    if (allocation_result != 0) {
        return allocation_result;
    }

    void *center = family_controls_center_shared((void *)center_response.metadata);
    if (center == NULL) {
        free(status_storage);
        return 2;
    }

    family_controls_center_authorization_status(status_storage, center);
    swift_release(center);
    const int64_t value = family_controls_status_raw_value(status_storage);
    const void *destroy = vwt->witnesses[1];
    ((void (*)(void *, const void *))destroy)(status_storage, status_response.metadata);
    free(status_storage);
    *raw_value = value;
    return 0;
}
