#include <stdint.h>
#include <stdlib.h>
#include "swift_abi_runtime.h"

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
    const struct SwiftValueWitnessTable *witnesses = NULL;
    uintptr_t value_size = 0;
    uintptr_t alignment = 0;
    if (!swift_abi_value_storage_layout_from_metadata(
            response.metadata,
            &witnesses,
            &value_size,
            &alignment) ||
        (witnesses->flags & SWIFT_ABI_VALUE_WITNESS_HAS_ENUM_WITNESSES) == 0 ||
        witnesses->get_enum_tag == NULL) {
        return 4;
    }

    const int32_t not_determined = alarm_authorization_not_determined_tag;
    const int32_t denied = alarm_authorization_denied_tag;
    const int32_t authorized = alarm_authorization_authorized_tag;
    if (not_determined < 0 || denied < 0 || authorized < 0 || not_determined == denied ||
        not_determined == authorized || denied == authorized) {
        return 4;
    }

    void *value = NULL;
    if (!swift_abi_allocate_value_storage(&value, alignment, value_size)) {
        return 5;
    }

    alarm_manager_authorization_state(value, manager);
    const uint32_t tag = witnesses->get_enum_tag(value, response.metadata);
    swift_abi_destroy_and_free_value(witnesses, value, response.metadata);

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
