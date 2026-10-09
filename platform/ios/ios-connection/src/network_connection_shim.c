#include "network_connection_shim.h"

#include <Block.h>
#include <Network/Network.h>
#include <dispatch/dispatch.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct framework_connection {
    atomic_uint references;
    pthread_mutex_t lifecycle_lock;
    bool cancel_requested;
    bool cancellation_finished;
    void *context;
    framework_connection_event_callback_t event_callback;
    nw_connection_t native_connection;
    dispatch_queue_t callback_queue;
    nw_connection_state_changed_handler_t state_handler;
};

struct framework_operation {
    framework_connection_t *connection;
    uint64_t operation_id;
    dispatch_data_t send_data;
    nw_connection_send_completion_t send_handler;
    nw_connection_receive_completion_t receive_handler;
};

static void retain_connection(framework_connection_t *connection) {
    atomic_fetch_add_explicit(&connection->references, 1, memory_order_relaxed);
}

static void release_connection(framework_connection_t *connection) {
    if (atomic_fetch_sub_explicit(&connection->references, 1, memory_order_acq_rel) == 1) {
        pthread_mutex_destroy(&connection->lifecycle_lock);
        free(connection);
    }
}

static void native_error_values(nw_error_t error, bool *has_error, int32_t *domain, int32_t *code) {
    *has_error = error != NULL;
    *domain = error == NULL ? 0 : (int32_t)nw_error_get_error_domain(error);
    *code = error == NULL ? 0 : (int32_t)nw_error_get_error_code(error);
}

static int32_t portable_state(nw_connection_state_t state) {
    switch (state) {
    case nw_connection_state_waiting:
        return FRAMEWORK_CONNECTION_STATE_WAITING;
    case nw_connection_state_preparing:
        return FRAMEWORK_CONNECTION_STATE_PREPARING;
    case nw_connection_state_ready:
        return FRAMEWORK_CONNECTION_STATE_READY;
    case nw_connection_state_failed:
        return FRAMEWORK_CONNECTION_STATE_FAILED;
    case nw_connection_state_invalid:
    case nw_connection_state_cancelled:
    default:
        return FRAMEWORK_CONNECTION_STATE_INVALID;
    }
}

static void release_native_resources_later(framework_connection_t *connection) {
    dispatch_async(connection->callback_queue, ^{
        pthread_mutex_lock(&connection->lifecycle_lock);
        nw_connection_t native_connection = connection->native_connection;
        dispatch_queue_t callback_queue = connection->callback_queue;
        nw_connection_state_changed_handler_t state_handler = connection->state_handler;
        connection->native_connection = NULL;
        connection->callback_queue = NULL;
        connection->state_handler = NULL;
        pthread_mutex_unlock(&connection->lifecycle_lock);
        nw_connection_set_state_changed_handler(native_connection, NULL);
        Block_release(state_handler);
        nw_release(native_connection);
        dispatch_release(callback_queue);
        release_connection(connection);
    });
}

static void request_cancel_from_callback_queue(framework_connection_t *connection) {
    pthread_mutex_lock(&connection->lifecycle_lock);
    if (connection->cancellation_finished || connection->cancel_requested
        || connection->callback_queue == NULL) {
        pthread_mutex_unlock(&connection->lifecycle_lock);
        return;
    }
    connection->cancel_requested = true;
    retain_connection(connection);
    dispatch_async(connection->callback_queue, ^{
        pthread_mutex_lock(&connection->lifecycle_lock);
        nw_connection_t native_connection = connection->native_connection;
        bool should_cancel = native_connection != NULL && !connection->cancellation_finished;
        pthread_mutex_unlock(&connection->lifecycle_lock);
        if (should_cancel) {
            nw_connection_cancel(native_connection);
        }
        release_connection(connection);
    });
    pthread_mutex_unlock(&connection->lifecycle_lock);
}

static void release_operation_later(struct framework_operation *operation) {
    framework_connection_t *connection = operation->connection;
    dispatch_async(connection->callback_queue, ^{
        if (operation->send_handler != NULL) {
            Block_release(operation->send_handler);
        }
        if (operation->receive_handler != NULL) {
            Block_release(operation->receive_handler);
        }
        if (operation->send_data != NULL) {
            dispatch_release(operation->send_data);
        }
        release_connection(connection);
        free(operation);
    });
}

int32_t framework_connection_start(
    void *context,
    framework_connection_event_callback_t event_callback,
    const char *hostname,
    uint16_t port,
    framework_connection_t **out_connection) {
    if (context == NULL || event_callback == NULL || hostname == NULL || hostname[0] == '\0'
        || port == 0 || out_connection == NULL) {
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }
    *out_connection = NULL;

    framework_connection_t *connection = calloc(1, sizeof(*connection));
    if (connection == NULL) {
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }
    if (pthread_mutex_init(&connection->lifecycle_lock, NULL) != 0) {
        free(connection);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    char port_text[6];
    (void)snprintf(port_text, sizeof(port_text), "%u", (unsigned int)port);
    nw_endpoint_t endpoint = nw_endpoint_create_host(hostname, port_text);
    nw_parameters_t parameters = nw_parameters_create_secure_tcp(
        NW_PARAMETERS_DEFAULT_CONFIGURATION,
        NW_PARAMETERS_DEFAULT_CONFIGURATION);
    connection->callback_queue = dispatch_queue_create(
        "org.jcdigitalsolutions.ios-connection.stream",
        DISPATCH_QUEUE_SERIAL);
    if (endpoint == NULL || parameters == NULL || connection->callback_queue == NULL) {
        if (endpoint != NULL) {
            nw_release(endpoint);
        }
        if (parameters != NULL) {
            nw_release(parameters);
        }
        if (connection->callback_queue != NULL) {
            dispatch_release(connection->callback_queue);
        }
        pthread_mutex_destroy(&connection->lifecycle_lock);
        free(connection);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    connection->native_connection = nw_connection_create(endpoint, parameters);
    nw_release(endpoint);
    nw_release(parameters);
    if (connection->native_connection == NULL) {
        dispatch_release(connection->callback_queue);
        pthread_mutex_destroy(&connection->lifecycle_lock);
        free(connection);
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }

    atomic_init(&connection->references, 1);
    connection->context = context;
    connection->event_callback = event_callback;
    connection->state_handler = Block_copy(^(nw_connection_state_t state, nw_error_t error) {
        bool has_error;
        int32_t domain;
        int32_t code;
        native_error_values(error, &has_error, &domain, &code);
        if (state == nw_connection_state_cancelled) {
            pthread_mutex_lock(&connection->lifecycle_lock);
            bool first_cancelled_event = !connection->cancellation_finished;
            connection->cancellation_finished = true;
            connection->cancel_requested = true;
            void *context = connection->context;
            connection->context = NULL;
            pthread_mutex_unlock(&connection->lifecycle_lock);
            if (first_cancelled_event) {
                connection->event_callback(
                    context,
                    FRAMEWORK_CONNECTION_EVENT_CLOSED,
                    0,
                    FRAMEWORK_CONNECTION_STATE_INVALID,
                    NULL,
                    0,
                    false,
                    false,
                    has_error,
                    false,
                    domain,
                    code);
                release_native_resources_later(connection);
            }
            return;
        }

        connection->event_callback(
            connection->context,
            FRAMEWORK_CONNECTION_EVENT_STATE,
            0,
            portable_state(state),
            NULL,
            0,
            false,
            false,
            has_error,
            false,
            domain,
            code);
        if (state == nw_connection_state_failed) {
            request_cancel_from_callback_queue(connection);
        }
    });
    if (connection->state_handler == NULL) {
        nw_release(connection->native_connection);
        dispatch_release(connection->callback_queue);
        pthread_mutex_destroy(&connection->lifecycle_lock);
        free(connection);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    retain_connection(connection);
    nw_connection_set_queue(connection->native_connection, connection->callback_queue);
    nw_connection_set_state_changed_handler(
        connection->native_connection,
        connection->state_handler);
    nw_connection_start(connection->native_connection);
    *out_connection = connection;
    return FRAMEWORK_CONNECTION_STATUS_OK;
}

static void send_completed(struct framework_operation *operation, nw_error_t error) {
    bool has_error;
    int32_t domain;
    int32_t code;
    native_error_values(error, &has_error, &domain, &code);
    framework_connection_t *connection = operation->connection;
    uint64_t operation_id = operation->operation_id;
    release_operation_later(operation);
    connection->event_callback(
        connection->context,
        FRAMEWORK_CONNECTION_EVENT_SEND,
        operation_id,
        FRAMEWORK_CONNECTION_STATE_INVALID,
        NULL,
        0,
        false,
        false,
        has_error,
        false,
        domain,
        code);
}

int32_t framework_connection_send(
    framework_connection_t *connection,
    uint64_t operation_id,
    const uint8_t *bytes,
    size_t bytes_length) {
    if (connection == NULL || bytes == NULL || bytes_length == 0) {
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }

    uint8_t *copy = malloc(bytes_length);
    if (copy == NULL) {
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }
    memcpy(copy, bytes, bytes_length);
    dispatch_data_t data = dispatch_data_create(
        copy,
        bytes_length,
        NULL,
        DISPATCH_DATA_DESTRUCTOR_FREE);
    if (data == NULL) {
        free(copy);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    struct framework_operation *operation = calloc(1, sizeof(*operation));
    if (operation == NULL) {
        dispatch_release(data);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }
    operation->connection = connection;
    operation->operation_id = operation_id;
    operation->send_data = data;
    operation->send_handler = Block_copy(^(nw_error_t error) {
        send_completed(operation, error);
    });
    if (operation->send_handler == NULL) {
        dispatch_release(data);
        free(operation);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    pthread_mutex_lock(&connection->lifecycle_lock);
    if (connection->native_connection == NULL || connection->cancel_requested
        || connection->cancellation_finished) {
        pthread_mutex_unlock(&connection->lifecycle_lock);
        Block_release(operation->send_handler);
        dispatch_release(data);
        free(operation);
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }
    retain_connection(connection);
    nw_connection_send(
        connection->native_connection,
        operation->send_data,
        NW_CONNECTION_DEFAULT_MESSAGE_CONTEXT,
        true,
        operation->send_handler);
    pthread_mutex_unlock(&connection->lifecycle_lock);
    return FRAMEWORK_CONNECTION_STATUS_OK;
}

static void receive_completed(
    struct framework_operation *operation,
    dispatch_data_t content,
    nw_content_context_t content_context,
    bool is_complete,
    nw_error_t error) {
    dispatch_data_t mapped = NULL;
    const void *mapped_bytes = NULL;
    size_t mapped_length = 0;
    if (content != NULL) {
        mapped = dispatch_data_create_map(content, &mapped_bytes, &mapped_length);
    }

    bool has_error;
    bool resource_exhausted = content != NULL && mapped == NULL;
    int32_t domain;
    int32_t code;
    native_error_values(error, &has_error, &domain, &code);
    framework_connection_t *connection = operation->connection;
    connection->event_callback(
        connection->context,
        FRAMEWORK_CONNECTION_EVENT_RECEIVE,
        operation->operation_id,
        FRAMEWORK_CONNECTION_STATE_INVALID,
        mapped_bytes,
        mapped_length,
        is_complete,
        content_context != NULL && nw_content_context_get_is_final(content_context),
        has_error,
        resource_exhausted,
        domain,
        code);
    if (mapped != NULL) {
        dispatch_release(mapped);
    }
    release_operation_later(operation);
}

int32_t framework_connection_receive(
    framework_connection_t *connection,
    uint64_t operation_id,
    uint32_t maximum_length) {
    if (connection == NULL || maximum_length == 0) {
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }

    struct framework_operation *operation = calloc(1, sizeof(*operation));
    if (operation == NULL) {
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }
    operation->connection = connection;
    operation->operation_id = operation_id;
    operation->receive_handler = Block_copy(^(
        dispatch_data_t content,
        nw_content_context_t content_context,
        bool is_complete,
        nw_error_t error) {
        receive_completed(operation, content, content_context, is_complete, error);
    });
    if (operation->receive_handler == NULL) {
        free(operation);
        return FRAMEWORK_CONNECTION_STATUS_RESOURCE_EXHAUSTED;
    }

    pthread_mutex_lock(&connection->lifecycle_lock);
    if (connection->native_connection == NULL || connection->cancel_requested
        || connection->cancellation_finished) {
        pthread_mutex_unlock(&connection->lifecycle_lock);
        Block_release(operation->receive_handler);
        free(operation);
        return FRAMEWORK_CONNECTION_STATUS_INVALID;
    }
    retain_connection(connection);
    nw_connection_receive(
        connection->native_connection,
        1,
        maximum_length,
        operation->receive_handler);
    pthread_mutex_unlock(&connection->lifecycle_lock);
    return FRAMEWORK_CONNECTION_STATUS_OK;
}

void framework_connection_cancel(framework_connection_t *connection) {
    if (connection == NULL) {
        return;
    }
    pthread_mutex_lock(&connection->lifecycle_lock);
    if (connection->cancellation_finished || connection->cancel_requested
        || connection->callback_queue == NULL) {
        pthread_mutex_unlock(&connection->lifecycle_lock);
        return;
    }
    connection->cancel_requested = true;
    retain_connection(connection);
    dispatch_async(connection->callback_queue, ^{
        pthread_mutex_lock(&connection->lifecycle_lock);
        nw_connection_t native_connection = connection->native_connection;
        bool should_cancel = native_connection != NULL && !connection->cancellation_finished;
        pthread_mutex_unlock(&connection->lifecycle_lock);
        if (should_cancel) {
            nw_connection_cancel(native_connection);
        }
        release_connection(connection);
    });
    pthread_mutex_unlock(&connection->lifecycle_lock);
}

void framework_connection_release(framework_connection_t *connection) {
    if (connection != NULL) {
        release_connection(connection);
    }
}
