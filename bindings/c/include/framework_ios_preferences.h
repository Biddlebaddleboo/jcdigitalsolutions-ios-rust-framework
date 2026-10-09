#ifndef FRAMEWORK_IOS_PREFERENCES_H
#define FRAMEWORK_IOS_PREFERENCES_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosPreferences FrameworkIosPreferences;

typedef uint32_t FrameworkIosPreferencesAvailability;
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef uint32_t FrameworkIosPreferencesUpdateRequirement;
#define FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC UINT32_C(0)
#define FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC UINT32_C(1)

typedef uint32_t FrameworkIosPreferencesUpdateAtomicity;
#define FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_ATOMIC UINT32_C(1)
#define FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED UINT32_C(2)

/*
 * This header is opt-in through the framework-c-api Cargo feature `ios-preferences`.
 * On iOS, calls use the app's standard NSUserDefaults domain and are synchronous.
 * They do not require a permission prompt or entitlement. The app must declare its
 * required NSUserDefaults usage reason in PrivacyInfo.xcprivacy. Values are non-secure
 * settings, not Keychain data, and persistence is asynchronous.
 *
 * Calls on one handle must be serialized. Serialized calls may use different threads;
 * do not call a function with a handle at the same time as another function or destroy.
 * On non-iOS targets create and all operations return FRAMEWORK_STATUS_UNSUPPORTED;
 * output slots and input shapes are still checked where the operation has them.
 * Non-null pointers must satisfy each function's readability, writability, alignment,
 * lifetime, and non-aliasing preconditions; those properties cannot be checked from C.
 */
/* Initializes *out_preferences to NULL before work. The slot must be writable, aligned, and must
 * not contain a live preferences handle. On success it receives one unique opaque handle. */
FrameworkStatus framework_ios_preferences_create(
    FrameworkIosPreferences **out_preferences);

/* NULL is a no-op. Otherwise pass the original valid, aligned, writable slot once; destroy
 * clears it first. */
void framework_ios_preferences_destroy(
    FrameworkIosPreferences **preferences);

/* out_availability must be writable, aligned, and distinct from the handle. Returns AVAILABLE on
 * iOS, without a promise that any later call will succeed. */
FrameworkStatus framework_ios_preferences_availability(
    const FrameworkIosPreferences *preferences,
    FrameworkIosPreferencesAvailability *out_availability);

/*
 * Keys are exact UTF-8, non-empty, case-sensitive, and contain no NUL byte. The key
 * span must be readable and unchanged through the call. Required outputs must be
 * writable, aligned, and distinct from one another, the input, and the handle. Each
 * non-null required output is set to its empty value before validation. out_value must
 * not hold a live buffer on entry.
 *
 * out_found distinguishes absence (0) from a present empty value (1 with an empty
 * buffer). When out_found is 1, release out_value exactly once with
 * framework_owned_buffer_destroy, including for an empty value. When out_found is 0,
 * out_value is the default empty descriptor.
 */
FrameworkStatus framework_ios_preferences_get(
    FrameworkIosPreferences *preferences,
    FrameworkStr key,
    uint8_t *out_found,
    FrameworkOwnedBuffer *out_value);

/*
 * value is opaque bytes borrowed through this call; its zero-length form is {NULL, 0}.
 * required_atomicity is ALLOW_NON_ATOMIC or REQUIRE_ATOMIC. The current B1 backend
 * returns NOT_GUARANTEED on success and returns UNSUPPORTED before mutation for
 * REQUIRE_ATOMIC. out_atomicity must be writable, aligned, and distinct from the
 * handle and input spans. It is initialized to UNKNOWN before validation and stays
 * unknown on errors.
 */
FrameworkStatus framework_ios_preferences_set(
    FrameworkIosPreferences *preferences,
    FrameworkStr key,
    FrameworkSlice value,
    FrameworkIosPreferencesUpdateRequirement required_atomicity,
    FrameworkIosPreferencesUpdateAtomicity *out_atomicity);

/*
 * out_removed must be writable, aligned, and distinct from the handle and key span.
 * It is zeroed before validation. On iOS it reports whether a value was
 * visible in the NSUserDefaults search list before the call. Removal affects the app's
 * domain; a registered or global fallback may be visible on a later get.
 */
FrameworkStatus framework_ios_preferences_remove(
    FrameworkIosPreferences *preferences,
    FrameworkStr key,
    uint8_t *out_removed);

#ifdef __cplusplus
}
#endif

#endif
