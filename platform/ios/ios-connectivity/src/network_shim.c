#include "network_shim.h"

#include <Network/Network.h>
#include <dispatch/dispatch.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdlib.h>

struct framework_path_monitor {
    atomic_uint references;
    atomic_bool cancel_requested;
    atomic_bool update_delivered;
    nw_path_monitor_t native_monitor;
    dispatch_queue_t callback_queue;
    void *context;
    framework_path_update_callback_t update_callback;
    framework_path_cancel_callback_t cancel_callback;
};

static void release_reference(framework_path_monitor_t *monitor) {
    if (atomic_fetch_sub_explicit(&monitor->references, 1, memory_order_acq_rel) != 1) {
        return;
    }

    nw_release(monitor->native_monitor);
    dispatch_release(monitor->callback_queue);
    free(monitor);
}

void framework_path_monitor_cancel(framework_path_monitor_t *monitor) {
    if (monitor != NULL
        && !atomic_exchange_explicit(&monitor->cancel_requested, true, memory_order_acq_rel)) {
        nw_path_monitor_cancel(monitor->native_monitor);
    }
}

static int32_t portable_status(nw_path_t path) {
    switch (nw_path_get_status(path)) {
    case nw_path_status_satisfied:
        return FRAMEWORK_PATH_STATUS_SATISFIED;
    case nw_path_status_unsatisfied:
        return FRAMEWORK_PATH_STATUS_UNSATISFIED;
    case nw_path_status_satisfiable:
        return FRAMEWORK_PATH_STATUS_SATISFIABLE;
    case nw_path_status_invalid:
    default:
        return FRAMEWORK_PATH_STATUS_UNKNOWN;
    }
}

framework_path_monitor_t *framework_path_monitor_create(
    void *context,
    framework_path_update_callback_t update_callback,
    framework_path_cancel_callback_t cancel_callback) {
    if (context == NULL || update_callback == NULL || cancel_callback == NULL) {
        return NULL;
    }

    framework_path_monitor_t *monitor = calloc(1, sizeof(*monitor));
    if (monitor == NULL) {
        return NULL;
    }

    monitor->native_monitor = nw_path_monitor_create();
    monitor->callback_queue = dispatch_queue_create(
        "org.jcdigitalsolutions.ios-connectivity.path-monitor",
        DISPATCH_QUEUE_SERIAL);
    if (monitor->native_monitor == NULL || monitor->callback_queue == NULL) {
        if (monitor->native_monitor != NULL) {
            nw_release(monitor->native_monitor);
        }
        if (monitor->callback_queue != NULL) {
            dispatch_release(monitor->callback_queue);
        }
        free(monitor);
        return NULL;
    }

    atomic_init(&monitor->references, 2);
    atomic_init(&monitor->cancel_requested, false);
    atomic_init(&monitor->update_delivered, false);
    monitor->context = context;
    monitor->update_callback = update_callback;
    monitor->cancel_callback = cancel_callback;

    nw_path_monitor_set_queue(monitor->native_monitor, monitor->callback_queue);
    nw_path_monitor_set_cancel_handler(monitor->native_monitor, ^{
        monitor->cancel_callback(monitor->context);
        monitor->context = NULL;
        release_reference(monitor);
    });
    nw_path_monitor_set_update_handler(monitor->native_monitor, ^(nw_path_t path) {
        if (path == NULL
            || atomic_exchange_explicit(&monitor->update_delivered, true, memory_order_acq_rel)) {
            return;
        }
        monitor->update_callback(monitor->context, portable_status(path));
        framework_path_monitor_cancel(monitor);
    });
    nw_path_monitor_start(monitor->native_monitor);
    return monitor;
}

void framework_path_monitor_release(framework_path_monitor_t *monitor) {
    if (monitor != NULL) {
        release_reference(monitor);
    }
}
