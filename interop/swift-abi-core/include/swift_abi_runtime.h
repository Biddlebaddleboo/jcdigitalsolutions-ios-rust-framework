#ifndef SWIFT_ABI_RUNTIME_H
#define SWIFT_ABI_RUNTIME_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

struct SwiftMetadataResponse {
    void *metadata;
    uintptr_t state;
};

typedef void (*SwiftDestroyWitness)(void *, const void *);
typedef uint32_t (*SwiftGetEnumTagWitness)(const void *, const void *);

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
    SwiftGetEnumTagWitness get_enum_tag;
    const void *destructive_project_enum_data;
    const void *destructive_inject_enum_tag;
};

#define SWIFT_ABI_VALUE_WITNESS_HAS_ENUM_WITNESSES UINT32_C(0x00200000)
#define SWIFT_ABI_VALUE_WITNESS_ALIGNMENT_MASK UINT32_C(0x000000ff)

static inline bool swift_abi_value_storage_alignment(
    const struct SwiftValueWitnessTable *witnesses,
    uintptr_t *alignment_out) {
    if (witnesses == NULL || alignment_out == NULL) {
        return false;
    }
    uintptr_t alignment =
        (uintptr_t)(witnesses->flags & SWIFT_ABI_VALUE_WITNESS_ALIGNMENT_MASK) + 1;
    if (alignment == 0 || (alignment & (alignment - 1)) != 0) {
        return false;
    }
    if (alignment < sizeof(void *)) {
        alignment = sizeof(void *);
    }
    *alignment_out = alignment;
    return true;
}

/* `metadata` must be compiler-proven Swift type metadata whose preceding word holds a valid
 * `SwiftValueWitnessTable` pointer; the caller ensures both reads are valid
 * On success, `witnesses_out` borrows that table and size is nonzero; alignment is read from
 * witness flags, validated as a power of two, and raised to at least `sizeof(void *)`
 * With all output pointers valid, failure leaves the witness pointer null and size/alignment zero
 * The caller owns allocated storage and calls `swift_abi_destroy_and_free_value` exactly once
 * after a successful Swift value init; this helper does not alloc, init, copy, or destroy values
 */
static inline bool swift_abi_value_storage_layout_from_metadata(
    const void *metadata,
    const struct SwiftValueWitnessTable **witnesses_out,
    uintptr_t *size_out,
    uintptr_t *alignment_out) {
    if (witnesses_out == NULL || size_out == NULL || alignment_out == NULL) {
        return false;
    }
    *witnesses_out = NULL;
    *size_out = 0;
    *alignment_out = 0;
    if (metadata == NULL) {
        return false;
    }

    const struct SwiftValueWitnessTable *const *const witnesses_location =
        (const struct SwiftValueWitnessTable *const *)metadata;
    const struct SwiftValueWitnessTable *const witnesses = witnesses_location[-1];
    uintptr_t alignment = 0;
    if (witnesses == NULL || witnesses->size == 0 || witnesses->destroy == NULL ||
        !swift_abi_value_storage_alignment(witnesses, &alignment)) {
        return false;
    }

    *witnesses_out = witnesses;
    *size_out = witnesses->size;
    *alignment_out = alignment;
    return true;
}

static inline bool swift_abi_allocate_value_storage(
    void **value_out,
    uintptr_t alignment,
    uintptr_t size) {
    if (value_out == NULL) {
        return false;
    }
    *value_out = NULL;
    if (size == 0 || alignment < sizeof(void *) || (alignment & (alignment - 1)) != 0) {
        return false;
    }
    return posix_memalign(value_out, alignment, size) == 0;
}

static inline void swift_abi_destroy_and_free_value(
    const struct SwiftValueWitnessTable *witnesses,
    void *value,
    const void *metadata) {
    witnesses->destroy(value, metadata);
    free(value);
}

_Static_assert(sizeof(void *) == 8, "Swift ABI bridges require 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, destroy) == 8, "Swift destroy witness offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, size) == 64, "Swift value size offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, flags) == 80, "Swift value flags offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, get_enum_tag) == 88, "Swift enum witness offset changed");
_Static_assert(sizeof(struct SwiftValueWitnessTable) == 112, "Swift value witness table size changed");
_Static_assert(sizeof(int64_t) == sizeof(intptr_t), "Swift Int snapshot needs 64-bit Apple targets");

#endif
