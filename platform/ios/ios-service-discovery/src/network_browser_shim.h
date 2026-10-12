#ifndef IOS_SERVICE_DISCOVERY_NETWORK_BROWSER_SHIM_H
#define IOS_SERVICE_DISCOVERY_NETWORK_BROWSER_SHIM_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct framework_bonjour_browser framework_bonjour_browser_t;

typedef int32_t (*framework_bonjour_event_callback_t)(
    void *context,
    int32_t event_kind,
    int32_t browser_state,
    const uint8_t *service_name,
    size_t service_name_length,
    const uint8_t *service_type,
    size_t service_type_length,
    const uint8_t *service_domain,
    size_t service_domain_length,
    bool has_error,
    int32_t error_domain,
    int32_t error_code);

enum {
    FRAMEWORK_BONJOUR_EVENT_STATE = 1,
    FRAMEWORK_BONJOUR_EVENT_SERVICE_ADDED = 2,
    FRAMEWORK_BONJOUR_EVENT_SERVICE_REMOVED = 3,
    FRAMEWORK_BONJOUR_EVENT_INVALID_RESULT = 4,
};

enum {
    FRAMEWORK_BONJOUR_STATE_READY = 1,
    FRAMEWORK_BONJOUR_STATE_FAILED = 2,
    FRAMEWORK_BONJOUR_STATE_CLOSED = 3,
    FRAMEWORK_BONJOUR_STATE_WAITING = 4,
};

enum {
    FRAMEWORK_BONJOUR_CALLBACK_OK = 0,
    FRAMEWORK_BONJOUR_CALLBACK_STOP = 1,
};

enum {
    FRAMEWORK_BONJOUR_STATUS_OK = 0,
    FRAMEWORK_BONJOUR_STATUS_INVALID = 1,
    FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED = 2,
};

int32_t framework_bonjour_browser_start(
    void *context,
    framework_bonjour_event_callback_t event_callback,
    const char *service_type,
    const char *service_domain,
    framework_bonjour_browser_t **browser);
void framework_bonjour_browser_cancel(framework_bonjour_browser_t *browser);
void framework_bonjour_browser_release(framework_bonjour_browser_t *browser);

#endif
