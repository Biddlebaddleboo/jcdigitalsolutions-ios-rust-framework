#include <assert.h>
#include <stdint.h>
#include "swift_abi_runtime.h"

static unsigned destroy_count;
static const void *expected_metadata;

static void destroy_value(void *value, const void *metadata) {
    assert(value != NULL);
    assert(metadata == expected_metadata);
    destroy_count += 1;
}

static void assert_layout_failure(const void *metadata) {
    const struct SwiftValueWitnessTable *witnesses =
        (const struct SwiftValueWitnessTable *)(uintptr_t)1;
    uintptr_t size = 99;
    uintptr_t alignment = 99;

    assert(!swift_abi_value_storage_layout_from_metadata(
        metadata, &witnesses, &size, &alignment));
    assert(witnesses == NULL);
    assert(size == 0);
    assert(alignment == 0);
}

static void test_metadata_layout(void) {
    struct SwiftValueWitnessTable witnesses = {0};
    const struct SwiftValueWitnessTable *metadata_words[2] = {&witnesses, NULL};
    const void *metadata = &metadata_words[1];
    const struct SwiftValueWitnessTable *witnesses_out = NULL;
    uintptr_t size_out = 0;
    uintptr_t alignment_out = 0;

    witnesses.destroy = destroy_value;
    witnesses.size = 24;
    witnesses.flags = 15;
    assert(swift_abi_value_storage_layout_from_metadata(
        metadata, &witnesses_out, &size_out, &alignment_out));
    assert(witnesses_out == &witnesses);
    assert(size_out == 24);
    assert(alignment_out == 16);

    assert(!swift_abi_value_storage_layout_from_metadata(
        metadata, NULL, &size_out, &alignment_out));
    assert(!swift_abi_value_storage_layout_from_metadata(
        metadata, &witnesses_out, NULL, &alignment_out));
    assert(!swift_abi_value_storage_layout_from_metadata(
        metadata, &witnesses_out, &size_out, NULL));
    assert_layout_failure(NULL);

    metadata_words[0] = NULL;
    assert_layout_failure(metadata);
    metadata_words[0] = &witnesses;
    witnesses.size = 0;
    assert_layout_failure(metadata);
    witnesses.size = 24;
    witnesses.destroy = NULL;
    assert_layout_failure(metadata);
    witnesses.destroy = destroy_value;
    witnesses.flags = 2;
    assert_layout_failure(metadata);
}

int main(void) {
    struct SwiftValueWitnessTable witnesses = {0};
    uintptr_t alignment = 99;

    test_metadata_layout();

    witnesses.flags = 0;
    assert(swift_abi_value_storage_alignment(&witnesses, &alignment));
    assert(alignment == sizeof(void *));
    witnesses.flags = 15;
    assert(swift_abi_value_storage_alignment(&witnesses, &alignment));
    assert(alignment == 16);
    witnesses.flags = 2;
    assert(!swift_abi_value_storage_alignment(&witnesses, &alignment));
    assert(alignment == 16);
    assert(!swift_abi_value_storage_alignment(NULL, &alignment));
    assert(!swift_abi_value_storage_alignment(&witnesses, NULL));

    void *value = (void *)(uintptr_t)1;
    assert(!swift_abi_allocate_value_storage(&value, 16, 0));
    assert(value == NULL);
    assert(!swift_abi_allocate_value_storage(&value, 3, 32));
    assert(value == NULL);
    assert(!swift_abi_allocate_value_storage(NULL, 16, 32));

    assert(swift_abi_allocate_value_storage(&value, 16, 32));
    assert(((uintptr_t)value & 15) == 0);
    witnesses.destroy = destroy_value;
    expected_metadata = &witnesses;
    swift_abi_destroy_and_free_value(&witnesses, value, expected_metadata);
    assert(destroy_count == 1);
    return 0;
}
