#include "network_listener_shim.h"

#include "network_connection_shim.h"

#include <Block.h>
#include <Network/Network.h>
#include <dispatch/dispatch.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

struct framework_listener {
    atomic_uint references;
    pthread_mutex_t lifecycle_lock;
    bool cancel_requested;
    bool cancellation_finished;
    void *context;
    framework_listener_event_callback_t event_callback;
    framework_listener_accept_callback_t accept_callback;
    nw_listener_t native_listener;
    dispatch_queue_t callback_queue;
    nw_listener_state_changed_handler_t state_handler;
    nw_listener_new_connection_handler_t accept_handler;
};

static void retain_listener(framework_listener_t *listener) {
    atomic_fetch_add_explicit(&listener->references, 1, memory_order_relaxed);
}

static void release_listener(framework_listener_t *listener) {
    if (atomic_fetch_sub_explicit(&listener->references, 1, memory_order_acq_rel) == 1) {
        pthread_mutex_destroy(&listener->lifecycle_lock);
        free(listener);
    }
}

static void listener_error_values(
    nw_error_t error,
    bool *has_error,
    int32_t *domain,
    int32_t *code) {
    *has_error = error != NULL;
    *domain = error == NULL ? 0 : (int32_t)nw_error_get_error_domain(error);
    *code = error == NULL ? 0 : (int32_t)nw_error_get_error_code(error);
}

static void release_native_resources_later(framework_listener_t *listener) {
    dispatch_async(listener->callback_queue, ^{
        pthread_mutex_lock(&listener->lifecycle_lock);
        nw_listener_t native_listener = listener->native_listener;
        dispatch_queue_t callback_queue = listener->callback_queue;
        nw_listener_state_changed_handler_t state_handler = listener->state_handler;
        nw_listener_new_connection_handler_t accept_handler = listener->accept_handler;
        listener->native_listener = NULL;
        listener->callback_queue = NULL;
        listener->state_handler = NULL;
        listener->accept_handler = NULL;
        pthread_mutex_unlock(&listener->lifecycle_lock);
        nw_listener_set_state_changed_handler(native_listener, NULL);
        nw_listener_set_new_connection_handler(native_listener, NULL);
        Block_release(state_handler);
        Block_release(accept_handler);
        nw_release(native_listener);
        dispatch_release(callback_queue);
        release_listener(listener);
    });
}

static void request_cancel_from_callback_queue(framework_listener_t *listener) {
    pthread_mutex_lock(&listener->lifecycle_lock);
    if (listener->cancellation_finished || listener->cancel_requested
        || listener->callback_queue == NULL) {
        pthread_mutex_unlock(&listener->lifecycle_lock);
        return;
    }
    listener->cancel_requested = true;
    retain_listener(listener);
    dispatch_async(listener->callback_queue, ^{
        pthread_mutex_lock(&listener->lifecycle_lock);
        nw_listener_t native_listener = listener->native_listener;
        bool should_cancel = native_listener != NULL && !listener->cancellation_finished;
        pthread_mutex_unlock(&listener->lifecycle_lock);
        if (should_cancel) {
            nw_listener_cancel(native_listener);
        }
        release_listener(listener);
    });
    pthread_mutex_unlock(&listener->lifecycle_lock);
}

int32_t framework_listener_start(
    void *context,
    framework_listener_event_callback_t event_callback,
    framework_listener_accept_callback_t accept_callback,
    uint16_t port,
    framework_listener_t **out_listener) {
    if (context == NULL || event_callback == NULL || accept_callback == NULL
        || out_listener == NULL) {
        return FRAMEWORK_LISTENER_STATUS_INVALID;
    }
    *out_listener = NULL;

    char port_text[6];
    (void)snprintf(port_text, sizeof(port_text), "%u", (unsigned int)port);
    nw_parameters_t parameters = nw_parameters_create();
    nw_protocol_stack_t stack = parameters == NULL
        ? NULL : nw_parameters_copy_default_protocol_stack(parameters);
    nw_protocol_options_t tcp_options = nw_tcp_create_options();
    if (parameters == NULL || stack == NULL || tcp_options == NULL) {
        if (tcp_options != NULL) {
            nw_release(tcp_options);
        }
        if (stack != NULL) {
            nw_release(stack);
        }
        if (parameters != NULL) {
            nw_release(parameters);
        }
        return FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED;
    }
    nw_protocol_stack_set_transport_protocol(stack, tcp_options);
    nw_release(tcp_options);
    nw_release(stack);

    nw_listener_t native_listener = nw_listener_create_with_port(port_text, parameters);
    nw_release(parameters);
    if (native_listener == NULL) {
        return FRAMEWORK_LISTENER_STATUS_INVALID;
    }

    framework_listener_t *listener = calloc(1, sizeof(*listener));
    if (listener == NULL) {
        nw_release(native_listener);
        return FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED;
    }
    if (pthread_mutex_init(&listener->lifecycle_lock, NULL) != 0) {
        nw_release(native_listener);
        free(listener);
        return FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED;
    }
    listener->native_listener = native_listener;
    listener->callback_queue = dispatch_queue_create(
        "org.jcdigitalsolutions.ios-connection.listener",
        DISPATCH_QUEUE_SERIAL);
    if (listener->callback_queue == NULL) {
        nw_release(native_listener);
        pthread_mutex_destroy(&listener->lifecycle_lock);
        free(listener);
        return FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED;
    }

    atomic_init(&listener->references, 1);
    listener->context = context;
    listener->event_callback = event_callback;
    listener->accept_callback = accept_callback;
    listener->state_handler = Block_copy(^(nw_listener_state_t state, nw_error_t error) {
        bool has_error;
        int32_t domain;
        int32_t code;
        listener_error_values(error, &has_error, &domain, &code);
        if (state == nw_listener_state_cancelled) {
            pthread_mutex_lock(&listener->lifecycle_lock);
            bool first_cancelled_event = !listener->cancellation_finished;
            listener->cancellation_finished = true;
            listener->cancel_requested = true;
            void *callback_context = listener->context;
            listener->context = NULL;
            pthread_mutex_unlock(&listener->lifecycle_lock);
            if (first_cancelled_event) {
                listener->event_callback(
                    callback_context,
                    FRAMEWORK_LISTENER_EVENT_CLOSED,
                    0,
                    has_error,
                    domain,
                    code);
                release_native_resources_later(listener);
            }
            return;
        }
        if (state == nw_listener_state_ready) {
            uint16_t port = nw_listener_get_port(listener->native_listener);
            if (port == 0) {
                listener->event_callback(
                    listener->context,
                    FRAMEWORK_LISTENER_EVENT_FAILED,
                    0,
                    false,
                    0,
                    0);
                request_cancel_from_callback_queue(listener);
                return;
            }
            listener->event_callback(
                listener->context,
                FRAMEWORK_LISTENER_EVENT_READY,
                port,
                false,
                0,
                0);
            return;
        }
        if (state == nw_listener_state_failed || state == nw_listener_state_invalid) {
            listener->event_callback(
                listener->context,
                FRAMEWORK_LISTENER_EVENT_FAILED,
                0,
                has_error,
                domain,
                code);
            request_cancel_from_callback_queue(listener);
        }
    });
    listener->accept_handler = Block_copy(^(nw_connection_t incoming) {
        int32_t status = listener->accept_callback(listener->context, (void *)incoming);
        if (status != FRAMEWORK_LISTENER_STATUS_OK) {
            nw_connection_cancel(incoming);
        }
    });
    if (listener->state_handler == NULL || listener->accept_handler == NULL) {
        if (listener->state_handler != NULL) {
            Block_release(listener->state_handler);
        }
        if (listener->accept_handler != NULL) {
            Block_release(listener->accept_handler);
        }
        dispatch_release(listener->callback_queue);
        nw_release(listener->native_listener);
        pthread_mutex_destroy(&listener->lifecycle_lock);
        free(listener);
        return FRAMEWORK_LISTENER_STATUS_RESOURCE_EXHAUSTED;
    }

    retain_listener(listener);
    nw_listener_set_queue(listener->native_listener, listener->callback_queue);
    nw_listener_set_state_changed_handler(listener->native_listener, listener->state_handler);
    nw_listener_set_new_connection_handler(listener->native_listener, listener->accept_handler);
    nw_listener_start(listener->native_listener);
    *out_listener = listener;
    return FRAMEWORK_LISTENER_STATUS_OK;
}

void framework_listener_cancel(framework_listener_t *listener) {
    if (listener == NULL) {
        return;
    }
    pthread_mutex_lock(&listener->lifecycle_lock);
    if (listener->cancellation_finished || listener->cancel_requested
        || listener->callback_queue == NULL) {
        pthread_mutex_unlock(&listener->lifecycle_lock);
        return;
    }
    listener->cancel_requested = true;
    retain_listener(listener);
    dispatch_async(listener->callback_queue, ^{
        pthread_mutex_lock(&listener->lifecycle_lock);
        nw_listener_t native_listener = listener->native_listener;
        bool should_cancel = native_listener != NULL && !listener->cancellation_finished;
        pthread_mutex_unlock(&listener->lifecycle_lock);
        if (should_cancel) {
            nw_listener_cancel(native_listener);
        }
        release_listener(listener);
    });
    pthread_mutex_unlock(&listener->lifecycle_lock);
}

void framework_listener_release(framework_listener_t *listener) {
    if (listener != NULL) {
        release_listener(listener);
    }
}
