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

int main(void) {
    struct SwiftValueWitnessTable witnesses = {0};
    uintptr_t alignment = 99;

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
