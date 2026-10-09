#ifndef IOS_CONNECTION_NETWORK_CONNECTION_SHIM_H
#define IOS_CONNECTION_NETWORK_CONNECTION_SHIM_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct framework_connection framework_connection_t;

typedef void (*framework_connection_event_callback_t)(
    void *context,
    int32_t event_kind,
    uint64_t operation_id,
    int32_t connection_state,
    const uint8_t *bytes,
    size_t bytes_length,
    bool is_complete,
    bool is_final_context,
    bool has_error,
    bool resource_exhausted,
    int32_t error_domain,
    int32_t error_code);

enum {
    FRAMEWORK_CONNECTION_EVENT_STATE = 1,
    FRAMEWORK_CONNECTION_EVENT_SEND = 2,
    FRAMEWORK_CONNECTION_EVENT_RECEIVE = 3,
    FRAMEWORK_CONNECTION_EVENT_CLOSED = 4,
};

enum {
    FRAMEWORK_CONNECTION_STATE_INVALID = 0,
    FRAMEWORK_CONNECTION_STATE_WAITING = 1,
    FRAMEWORK_CONNECTION_STATE_PREPARING = 2,
    FRAMEWORK_CONNECTION_STATE_READY = 3,
    FRAMEWORK_CONNECTION_STATE_FAILED = 4,
};

enum {
    FRAMEWORK_CONNECTION_STATUS_OK = 0,
    FRAMEWORK_CONNECTION_STATUS_INVALID = 1,
    FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED = 2,
};

int32_t framework_connection_start(
    void *context,
    framework_connection_event_callback_t event_callback,
    const char *hostname,
    uint16_t port,
    framework_connection_t **connection);
int32_t framework_connection_send(
    framework_connection_t *connection,
    uint64_t operation_id,
    const uint8_t *bytes,
    size_t bytes_length);
int32_t framework_connection_receive(
    framework_connection_t *connection,
    uint64_t operation_id,
    uint32_t maximum_length);
void framework_connection_cancel(framework_connection_t *connection);
void framework_connection_release(framework_connection_t *connection);

#endif
