#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

struct SwiftMetadataResponse {
    const void *metadata;
    uintptr_t state;
};

typedef void (*SwiftDestroyWitness)(void *, const void *);
typedef uint32_t (*SwiftGetEnumTagWitness)(const void *, const void *);

struct SwiftValueWitnessTable {
    const void *common_prefix[1];
    SwiftDestroyWitness destroy;
    const void *common_suffix[6];
    uintptr_t size;
    uintptr_t stride;
    uint32_t flags;
    uint32_t extra_inhabitant_count;
    SwiftGetEnumTagWitness get_enum_tag;
    const void *destructive_project_enum_data;
    const void *destructive_inject_enum_tag;
};

#define SWIFT_VALUE_WITNESS_HAS_ENUM_WITNESSES UINT32_C(0x00200000)
#define SWIFT_VALUE_WITNESS_ALIGNMENT_MASK UINT32_C(0x000000ff)

_Static_assert(sizeof(void *) == 8, "AlarmKit Swift ABI bridge requires 64-bit Apple targets");
_Static_assert(sizeof(uintptr_t) == 8, "Swift metadata response state must be 64 bits");
_Static_assert(offsetof(struct SwiftMetadataResponse, state) == 8, "Swift metadata response offset changed");
_Static_assert(sizeof(struct SwiftMetadataResponse) == 16, "Swift metadata response size changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, destroy) == 8, "Swift destroy witness offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, size) == 64, "Swift value size offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, flags) == 80, "Swift value flags offset changed");
_Static_assert(offsetof(struct SwiftValueWitnessTable, get_enum_tag) == 88, "Swift enum witness offset changed");

extern struct SwiftMetadataResponse alarm_manager_metadata(uint64_t request)
    __asm__("_$s8AlarmKit0A7ManagerCMa")
    __attribute__((swiftcall, weak_import));
extern void *alarm_manager_shared(void *metadata __attribute__((swift_context)))
    __asm__("_$s8AlarmKit0A7ManagerC6sharedACvgZ")
    __attribute__((swiftcall, weak_import));
extern void alarm_manager_authorization_state(
    void *state __attribute__((swift_indirect_result)),
    void *manager __attribute__((swift_context)))
    __asm__("_$s8AlarmKit0A7ManagerC18authorizationStateAC013AuthorizationE0OvgTj")
    __attribute__((swiftcall, weak_import));
extern struct SwiftMetadataResponse alarm_authorization_state_metadata(uint64_t request)
    __asm__("_$s8AlarmKit0A7ManagerC18AuthorizationStateOMa")
    __attribute__((swiftcall, weak_import));
extern int32_t alarm_authorization_not_determined_tag
    __asm__("_$s8AlarmKit0A7ManagerC18AuthorizationStateO13notDeterminedyA2EmFWC")
    __attribute__((weak_import));
extern int32_t alarm_authorization_denied_tag
    __asm__("_$s8AlarmKit0A7ManagerC18AuthorizationStateO6deniedyA2EmFWC")
    __attribute__((weak_import));
extern int32_t alarm_authorization_authorized_tag
    __asm__("_$s8AlarmKit0A7ManagerC18AuthorizationStateO10authorizedyA2EmFWC")
    __attribute__((weak_import));

uint8_t framework_alarm_manager_create(void **manager_out) {
    if (manager_out == NULL) {
        return 3;
    }
    *manager_out = NULL;
    if (alarm_manager_metadata == NULL || alarm_manager_shared == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = alarm_manager_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }
    void *const manager = alarm_manager_shared((void *)response.metadata);
    if (manager == NULL) {
        return 2;
    }
    *manager_out = manager;
    return 0;
}

uint8_t framework_alarm_manager_authorization_state(void *manager, uint8_t *state_out) {
    if (manager == NULL || state_out == NULL) {
        return 3;
    }
    *state_out = UINT8_MAX;
    if (alarm_manager_authorization_state == NULL || alarm_authorization_state_metadata == NULL ||
        &alarm_authorization_not_determined_tag == NULL || &alarm_authorization_denied_tag == NULL ||
        &alarm_authorization_authorized_tag == NULL) {
        return 1;
    }

    const struct SwiftMetadataResponse response = alarm_authorization_state_metadata(0);
    if (response.metadata == NULL) {
        return 2;
    }
    const struct SwiftValueWitnessTable *const witnesses =
        ((const struct SwiftValueWitnessTable *const *)response.metadata)[-1];
    if (witnesses == NULL ||
        (witnesses->flags & SWIFT_VALUE_WITNESS_HAS_ENUM_WITNESSES) == 0 ||
        witnesses->size == 0 || witnesses->get_enum_tag == NULL || witnesses->destroy == NULL) {
        return 4;
    }

    const int32_t not_determined = alarm_authorization_not_determined_tag;
    const int32_t denied = alarm_authorization_denied_tag;
    const int32_t authorized = alarm_authorization_authorized_tag;
    if (not_determined < 0 || denied < 0 || authorized < 0 || not_determined == denied ||
        not_determined == authorized || denied == authorized) {
        return 4;
    }

    uintptr_t alignment =
        (uintptr_t)(witnesses->flags & SWIFT_VALUE_WITNESS_ALIGNMENT_MASK) + 1;
    if (alignment == 0 || (alignment & (alignment - 1)) != 0) {
        return 4;
    }
    if (alignment < sizeof(void *)) {
        alignment = sizeof(void *);
    }

    void *value = NULL;
    if (posix_memalign(&value, alignment, witnesses->size) != 0) {
        return 5;
    }

    alarm_manager_authorization_state(value, manager);
    const uint32_t tag = witnesses->get_enum_tag(value, response.metadata);
    witnesses->destroy(value, response.metadata);
    free(value);

    if (tag == (uint32_t)not_determined) {
        *state_out = 0;
    } else if (tag == (uint32_t)denied) {
        *state_out = 1;
    } else if (tag == (uint32_t)authorized) {
        *state_out = 2;
    } else {
        *state_out = 3;
    }
    return 0;
}
