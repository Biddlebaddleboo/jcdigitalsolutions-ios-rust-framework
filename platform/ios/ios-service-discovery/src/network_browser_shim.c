#include "network_browser_shim.h"

#include <Block.h>
#include <Network/Network.h>
#include <dispatch/dispatch.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>

#define MAX_IDENTITY_COMPONENT_BYTES 255

struct framework_bonjour_browser {
    atomic_uint references;
    pthread_mutex_t lifecycle_lock;
    bool cancel_requested;
    bool cancellation_finished;
    void *context;
    framework_bonjour_event_callback_t event_callback;
    nw_browser_t native_browser;
    dispatch_queue_t callback_queue;
    nw_browser_state_changed_handler_t state_handler;
    nw_browser_browse_results_changed_handler_t results_handler;
};

typedef struct {
    char name[MAX_IDENTITY_COMPONENT_BYTES + 1];
    size_t name_length;
    char type[MAX_IDENTITY_COMPONENT_BYTES + 1];
    size_t type_length;
    char domain[MAX_IDENTITY_COMPONENT_BYTES + 1];
    size_t domain_length;
} framework_bonjour_identity_t;

static void retain_browser(framework_bonjour_browser_t *browser) {
    atomic_fetch_add_explicit(&browser->references, 1, memory_order_relaxed);
}

static void release_browser(framework_bonjour_browser_t *browser) {
    if (atomic_fetch_sub_explicit(&browser->references, 1, memory_order_acq_rel) == 1) {
        pthread_mutex_destroy(&browser->lifecycle_lock);
        free(browser);
    }
}

static bool copy_bounded_component(
    const char *value,
    char output[MAX_IDENTITY_COMPONENT_BYTES + 1],
    size_t *length) {
    if (value == NULL) {
        return false;
    }
    size_t result = 0;
    while (result <= MAX_IDENTITY_COMPONENT_BYTES && value[result] != '\0') {
        ++result;
    }
    if (result == 0 || result > MAX_IDENTITY_COMPONENT_BYTES) {
        return false;
    }
    memcpy(output, value, result);
    output[result] = '\0';
    *length = result;
    return true;
}

static bool copy_identity(nw_browse_result_t result, framework_bonjour_identity_t *identity) {
    if (result == NULL || identity == NULL) {
        return false;
    }
    nw_endpoint_t endpoint = nw_browse_result_copy_endpoint(result);
    if (endpoint == NULL) {
        return false;
    }
    bool valid = nw_endpoint_get_type(endpoint) == nw_endpoint_type_bonjour_service;
    if (valid) {
        valid = copy_bounded_component(
                    nw_endpoint_get_bonjour_service_name(endpoint),
                    identity->name,
                    &identity->name_length)
            && copy_bounded_component(
                nw_endpoint_get_bonjour_service_type(endpoint),
                identity->type,
                &identity->type_length)
            && copy_bounded_component(
                nw_endpoint_get_bonjour_service_domain(endpoint),
                identity->domain,
                &identity->domain_length);
    }
    nw_release(endpoint);
    return valid;
}

static bool identity_equal(
    const framework_bonjour_identity_t *left,
    const framework_bonjour_identity_t *right) {
    return left->name_length == right->name_length
        && left->type_length == right->type_length
        && left->domain_length == right->domain_length
        && memcmp(left->name, right->name, left->name_length) == 0
        && memcmp(left->type, right->type, left->type_length) == 0
        && memcmp(left->domain, right->domain, left->domain_length) == 0;
}

static int32_t emit_result(
    framework_bonjour_browser_t *browser,
    int32_t event_kind,
    nw_browse_result_t result) {
    framework_bonjour_identity_t identity = {0};
    if (!copy_identity(result, &identity)) {
        return browser->event_callback(
            browser->context,
            FRAMEWORK_BONJOUR_EVENT_INVALID_RESULT,
            0,
            NULL,
            0,
            NULL,
            0,
            NULL,
            0,
            false,
            0,
            0);
    }
    return browser->event_callback(
        browser->context,
        event_kind,
        0,
        (const uint8_t *)identity.name,
        identity.name_length,
        (const uint8_t *)identity.type,
        identity.type_length,
        (const uint8_t *)identity.domain,
        identity.domain_length,
        false,
        0,
        0);
}

static void request_cancel_from_callback_queue(framework_bonjour_browser_t *browser);

static void release_native_resources_later(framework_bonjour_browser_t *browser) {
    dispatch_async(browser->callback_queue, ^{
        pthread_mutex_lock(&browser->lifecycle_lock);
        nw_browser_t native_browser = browser->native_browser;
        dispatch_queue_t callback_queue = browser->callback_queue;
        nw_browser_state_changed_handler_t state_handler = browser->state_handler;
        nw_browser_browse_results_changed_handler_t results_handler = browser->results_handler;
        browser->native_browser = NULL;
        browser->callback_queue = NULL;
        browser->state_handler = NULL;
        browser->results_handler = NULL;
        pthread_mutex_unlock(&browser->lifecycle_lock);
        Block_release(state_handler);
        Block_release(results_handler);
        nw_release(native_browser);
        dispatch_release(callback_queue);
        release_browser(browser);
    });
}

static void request_cancel_from_callback_queue(framework_bonjour_browser_t *browser) {
    pthread_mutex_lock(&browser->lifecycle_lock);
    if (browser->cancellation_finished || browser->cancel_requested
        || browser->callback_queue == NULL) {
        pthread_mutex_unlock(&browser->lifecycle_lock);
        return;
    }
    browser->cancel_requested = true;
    retain_browser(browser);
    dispatch_async(browser->callback_queue, ^{
        pthread_mutex_lock(&browser->lifecycle_lock);
        nw_browser_t native_browser = browser->native_browser;
        bool should_cancel = native_browser != NULL && !browser->cancellation_finished;
        pthread_mutex_unlock(&browser->lifecycle_lock);
        if (should_cancel) {
            nw_browser_cancel(native_browser);
        }
        release_browser(browser);
    });
    pthread_mutex_unlock(&browser->lifecycle_lock);
}

static void browser_state_changed(nw_browser_state_t state, nw_error_t error, void *context) {
    framework_bonjour_browser_t *browser = context;
    int32_t mapped_state = 0;
    switch (state) {
        case nw_browser_state_ready:
            mapped_state = FRAMEWORK_BONJOUR_STATE_READY;
            break;
        case nw_browser_state_waiting:
            mapped_state = FRAMEWORK_BONJOUR_STATE_WAITING;
            break;
        case nw_browser_state_failed:
            mapped_state = FRAMEWORK_BONJOUR_STATE_FAILED;
            break;
        case nw_browser_state_cancelled:
            mapped_state = FRAMEWORK_BONJOUR_STATE_CLOSED;
            break;
        default:
            mapped_state = FRAMEWORK_BONJOUR_STATE_FAILED;
            break;
    }
    bool has_error = error != NULL;
    int32_t error_domain = has_error ? (int32_t)nw_error_get_error_domain(error) : 0;
    int32_t error_code = has_error ? (int32_t)nw_error_get_error_code(error) : 0;
    if (state == nw_browser_state_cancelled) {
        pthread_mutex_lock(&browser->lifecycle_lock);
        bool first_cancelled_event = !browser->cancellation_finished;
        browser->cancellation_finished = true;
        browser->cancel_requested = true;
        void *callback_context = browser->context;
        browser->context = NULL;
        pthread_mutex_unlock(&browser->lifecycle_lock);
        if (first_cancelled_event) {
            browser->event_callback(
                callback_context,
                FRAMEWORK_BONJOUR_EVENT_STATE,
                FRAMEWORK_BONJOUR_STATE_CLOSED,
                NULL,
                0,
                NULL,
                0,
                NULL,
                0,
                has_error,
                error_domain,
                error_code);
            release_native_resources_later(browser);
        }
        return;
    }

    int32_t callback_status = browser->event_callback(
        browser->context,
        FRAMEWORK_BONJOUR_EVENT_STATE,
        mapped_state,
        NULL,
        0,
        NULL,
        0,
        NULL,
        0,
        has_error,
        error_domain,
        error_code);
    if (callback_status != FRAMEWORK_BONJOUR_CALLBACK_OK || state == nw_browser_state_failed
        || state == nw_browser_state_invalid) {
        request_cancel_from_callback_queue(browser);
    }
}

static void browser_results_changed(
    nw_browse_result_t old_result,
    nw_browse_result_t new_result,
    bool batch_complete,
    void *context) {
    (void)batch_complete;
    framework_bonjour_browser_t *browser = context;
    nw_browse_result_change_t changes = nw_browse_result_get_changes(old_result, new_result);
    int32_t callback_status = FRAMEWORK_BONJOUR_CALLBACK_OK;
    if (old_result == NULL && new_result != NULL) {
        callback_status = emit_result(browser, FRAMEWORK_BONJOUR_EVENT_SERVICE_ADDED, new_result);
    } else if (old_result != NULL && new_result == NULL) {
        callback_status = emit_result(browser, FRAMEWORK_BONJOUR_EVENT_SERVICE_REMOVED, old_result);
    } else if (old_result != NULL && new_result != NULL
        && ((changes & (nw_browse_result_change_result_added | nw_browse_result_change_result_removed))
            == (nw_browse_result_change_result_added | nw_browse_result_change_result_removed))) {
        framework_bonjour_identity_t old_identity = {0};
        framework_bonjour_identity_t new_identity = {0};
        if (!copy_identity(old_result, &old_identity) || !copy_identity(new_result, &new_identity)) {
            callback_status = browser->event_callback(
                browser->context,
                FRAMEWORK_BONJOUR_EVENT_INVALID_RESULT,
                0,
                NULL,
                0,
                NULL,
                0,
                NULL,
                0,
                false,
                0,
                0);
        } else if (!identity_equal(&old_identity, &new_identity)) {
            callback_status = emit_result(browser, FRAMEWORK_BONJOUR_EVENT_SERVICE_REMOVED, old_result);
            if (callback_status == FRAMEWORK_BONJOUR_CALLBACK_OK) {
                callback_status = emit_result(browser, FRAMEWORK_BONJOUR_EVENT_SERVICE_ADDED, new_result);
            }
        }
    }
    if (callback_status != FRAMEWORK_BONJOUR_CALLBACK_OK) {
        request_cancel_from_callback_queue(browser);
    }
}

int32_t framework_bonjour_browser_start(
    void *context,
    framework_bonjour_event_callback_t event_callback,
    const char *service_type,
    const char *service_domain,
    framework_bonjour_browser_t **out_browser) {
    if (context == NULL || event_callback == NULL || service_type == NULL || out_browser == NULL) {
        return FRAMEWORK_BONJOUR_STATUS_INVALID;
    }
    *out_browser = NULL;
    nw_browse_descriptor_t descriptor =
        nw_browse_descriptor_create_bonjour_service(service_type, service_domain);
    if (descriptor == NULL) {
        return FRAMEWORK_BONJOUR_STATUS_INVALID;
    }
    nw_browse_descriptor_set_include_txt_record(descriptor, false);
    nw_browser_t native_browser = nw_browser_create(descriptor, NULL);
    nw_release(descriptor);
    if (native_browser == NULL) {
        return FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED;
    }

    framework_bonjour_browser_t *browser = calloc(1, sizeof(*browser));
    if (browser == NULL) {
        nw_release(native_browser);
        return FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED;
    }
    if (pthread_mutex_init(&browser->lifecycle_lock, NULL) != 0) {
        nw_release(native_browser);
        free(browser);
        return FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED;
    }
    browser->native_browser = native_browser;
    browser->callback_queue = dispatch_queue_create(
        "org.jcdigitalsolutions.ios-service-discovery.bonjour",
        DISPATCH_QUEUE_SERIAL);
    if (browser->callback_queue == NULL) {
        nw_release(native_browser);
        pthread_mutex_destroy(&browser->lifecycle_lock);
        free(browser);
        return FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED;
    }

    atomic_init(&browser->references, 1);
    browser->context = context;
    browser->event_callback = event_callback;
    browser->state_handler = Block_copy(^(nw_browser_state_t state, nw_error_t error) {
        browser_state_changed(state, error, browser);
    });
    browser->results_handler = Block_copy(^(nw_browse_result_t old_result,
                                            nw_browse_result_t new_result,
                                            bool batch_complete) {
        browser_results_changed(old_result, new_result, batch_complete, browser);
    });
    if (browser->state_handler == NULL || browser->results_handler == NULL) {
        if (browser->state_handler != NULL) {
            Block_release(browser->state_handler);
        }
        if (browser->results_handler != NULL) {
            Block_release(browser->results_handler);
        }
        dispatch_release(browser->callback_queue);
        nw_release(native_browser);
        pthread_mutex_destroy(&browser->lifecycle_lock);
        free(browser);
        return FRAMEWORK_BONJOUR_STATUS_RESOURCE_EXHAUSTED;
    }

    retain_browser(browser);
    nw_browser_set_queue(native_browser, browser->callback_queue);
    nw_browser_set_state_changed_handler(native_browser, browser->state_handler);
    nw_browser_set_browse_results_changed_handler(native_browser, browser->results_handler);
    nw_browser_start(native_browser);
    *out_browser = browser;
    return FRAMEWORK_BONJOUR_STATUS_OK;
}

void framework_bonjour_browser_cancel(framework_bonjour_browser_t *browser) {
    if (browser == NULL) {
        return;
    }
    pthread_mutex_lock(&browser->lifecycle_lock);
    if (browser->cancellation_finished || browser->cancel_requested
        || browser->callback_queue == NULL) {
        pthread_mutex_unlock(&browser->lifecycle_lock);
        return;
    }
    browser->cancel_requested = true;
    retain_browser(browser);
    dispatch_async(browser->callback_queue, ^{
        pthread_mutex_lock(&browser->lifecycle_lock);
        nw_browser_t native_browser = browser->native_browser;
        bool should_cancel = native_browser != NULL && !browser->cancellation_finished;
        pthread_mutex_unlock(&browser->lifecycle_lock);
        if (should_cancel) {
            nw_browser_cancel(native_browser);
        }
        release_browser(browser);
    });
    pthread_mutex_unlock(&browser->lifecycle_lock);
}

void framework_bonjour_browser_release(framework_bonjour_browser_t *browser) {
    if (browser != NULL) {
        release_browser(browser);
    }
}
