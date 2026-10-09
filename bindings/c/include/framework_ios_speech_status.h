#ifndef FRAMEWORK_IOS_SPEECH_STATUS_H
#define FRAMEWORK_IOS_SPEECH_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef int64_t FrameworkIosSpeechAuthorizationStatus;
#define FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_NOT_DETERMINED INT64_C(0)
#define FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_DENIED INT64_C(1)
#define FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_RESTRICTED INT64_C(2)
#define FRAMEWORK_IOS_SPEECH_AUTHORIZATION_STATUS_AUTHORIZED INT64_C(3)

/*
 * Opt in with framework-c-api's `ios-speech-status` Cargo feature. On iOS this reads
 * +[SFSpeechRecognizer authorizationStatus], whose API floor is iOS 10.0. The F28 link probes
 * use device minos 10.0 and Simulator minos 14.0; these are target link settings, not a claim
 * about authorization or service availability.
 *
 * Known raw values are 0 NOT_DETERMINED, 1 DENIED, 2 RESTRICTED, and 3 AUTHORIZED. Unknown native
 * values are returned unchanged as signed int64_t values. The query does not call requestAuthorization, show
 * a prompt, create a recognizer, accept audio, or start recognition. A successful result reports
 * saved authorization only; it does not report service availability or guarantee recognition
 * success.
 *
 * `out_raw_status` is required and is set to zero before platform handling. Read it only when
 * FRAMEWORK_STATUS_OK is returned. On iOS below 10.0 the function returns
 * FRAMEWORK_STATUS_UNAVAILABLE with zero output. A valid non-iOS call returns
 * FRAMEWORK_STATUS_UNSUPPORTED with zero output. A null output returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC with zero output. The pointer is caller-owned and not retained; the API
 * checks only nullness, so the caller must provide valid aligned writable storage and prevent
 * unsynchronized concurrent access.
 *
 * Calls are synchronous on the caller's thread. Apple's header states no main-thread rule for the
 * class method; this API makes no broader thread-safety promise. Host privacy metadata for other
 * Speech use remains the app's responsibility.
 */
FrameworkStatus framework_ios_speech_status_authorization_status(
    FrameworkIosSpeechAuthorizationStatus *out_raw_status);

#ifdef __cplusplus
}
#endif

#endif
