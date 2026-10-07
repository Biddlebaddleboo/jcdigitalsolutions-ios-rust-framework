#include "framework_ios_secure_storage.h"

#include <assert.h>

static FrameworkStr as_framework_str(const char *value) {
    FrameworkStr result = {(const uint8_t *)value, 0};
    while (value[result.length] != '\0') {
        ++result.length;
    }
    return result;
}

int main(void) {
    FrameworkStr service = as_framework_str("com.example.account");
    FrameworkStr item = as_framework_str("access-token");
    const FrameworkSlice empty_secret = {0, 0};
    FrameworkOwnedBuffer secret = {0};
    uint8_t found = 7;
    uint8_t removed = 7;
    uint32_t effective_policy = 99;
    int32_t native_status = -99;

    assert(framework_ios_secure_storage_read(service, item, &found, &secret, &native_status) == FRAMEWORK_STATUS_UNSUPPORTED);
    assert(found == 0 && secret.data == 0 && secret.length == 0 && secret.capacity == 0);
    assert(native_status == 0);

    effective_policy = 99;
    native_status = -99;
    assert(framework_ios_secure_storage_store(service, item, empty_secret, 0, &effective_policy, &native_status) == FRAMEWORK_STATUS_UNSUPPORTED);
    assert(effective_policy == 0 && native_status == 0);

    removed = 7;
    native_status = -99;
    assert(framework_ios_secure_storage_remove(service, item, &removed, &native_status) == FRAMEWORK_STATUS_UNSUPPORTED);
    assert(removed == 0 && native_status == 0);

    effective_policy = 99;
    native_status = -99;
    assert(framework_ios_secure_storage_store(service, item, empty_secret, UINT32_C(4), &effective_policy, &native_status) == FRAMEWORK_STATUS_INVALID_ARGUMENT);
    assert(effective_policy == 0 && native_status == 0);

    native_status = -99;
    assert(framework_ios_secure_storage_read(service, item, 0, &secret, &native_status) == FRAMEWORK_STATUS_INVALID_ARGUMENT);
    assert(native_status == 0);
    return 0;
}
