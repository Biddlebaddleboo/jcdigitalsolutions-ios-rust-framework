#ifndef IOS_CONNECTION_NETWORK_LISTENER_SHIM_H
#define IOS_CONNECTION_NETWORK_LISTENER_SHIM_H

#include <stdbool.h>
#include <stdint.h>

typedef struct framework_listener framework_listener_t;

typedef void (*framework_listener_event_callback_t)(
    void *context,
    int32_t event_kind,
    uint16_t port,
    bool has_error,
    int32_t error_domain,
    int32_t error_code);

typedef int32_t (*framework_listener_accept_callback_t)(void *context, void *native_connection);

enum {
    FRAMEWORK_LISTENER_EVENT_READY = 1,
    FRAMEWORK_LISTENER_EVENT_FAILED = 2,
    FRAMEWORK_LISTENER_EVENT_CLOSED = 3,
};

enum {
    FRAMEWORK_LISTENER_STATUS_OK = 0,
    FRAMEWORK_LISTENER_STATUS_INVALID = 1,
    FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED = 2,
};

int32_t framework_listener_start(
    void *context,
    framework_listener_event_callback_t event_callback,
    framework_listener_accept_callback_t accept_callback,
    uint16_t port,
    const char *advertisement_name,
    const char *advertisement_service_type,
    const char *advertisement_domain,
    framework_listener_t **listener);
int32_t framework_listener_set_advertisement(
    framework_listener_t *listener,
    const char *advertisement_name,
    const char *advertisement_service_type,
    const char *advertisement_domain);
void framework_listener_cancel(framework_listener_t *listener);
void framework_listener_release(framework_listener_t *listener);

#endif
