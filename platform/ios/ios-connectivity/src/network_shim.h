#ifndef FRAMEWORK_IOS_CONNECTIVITY_NETWORK_SHIM_H
#define FRAMEWORK_IOS_CONNECTIVITY_NETWORK_SHIM_H

#include <stdint.h>

typedef struct framework_path_monitor framework_path_monitor_t;
typedef void (*framework_path_update_callback_t)(void *context, int32_t status);
typedef void (*framework_path_cancel_callback_t)(void *context);

enum framework_path_status_tag {
    FRAMEWORK_PATH_STATUS_UNKNOWN = 0,
    FRAMEWORK_PATH_STATUS_SATISFIED = 1,
    FRAMEWORK_PATH_STATUS_UNSATISFIED = 2,
    FRAMEWORK_PATH_STATUS_SATISFIABLE = 3,
};

framework_path_monitor_t *framework_path_monitor_create(
    void *context,
    framework_path_update_callback_t update_callback,
    framework_path_cancel_callback_t cancel_callback);
void framework_path_monitor_cancel(framework_path_monitor_t *monitor);
void framework_path_monitor_release(framework_path_monitor_t *monitor);

#endif
