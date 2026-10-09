#ifndef FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_STATUS_H
#define FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef int64_t FrameworkIosSignInWithAppleCredentialState;
#define FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED INT64_C(0)
#define FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED INT64_C(1)
#define FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND INT64_C(2)
#define FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED INT64_C(3)

typedef void (*FrameworkIosSignInWithAppleCredentialStateCompletion)(
    void *context,
    FrameworkIosSignInWithAppleCredentialState credential_state,
    uint8_t had_error);

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-sign-in-with-apple-status`. It wraps only the iOS 13.0+
 * ASAuthorizationAppleIDProvider.getCredentialStateForUserID:completion:
 * query in AuthenticationServices.framework. The host must enable Sign in
 * with Apple and include com.apple.developer.applesignin with its normal
 * `Default` value. Apple does not say whether this query alone is exempt.
 *
 * `user_id` must be a non-empty UTF-8 opaque ID received from a prior
 * successful Sign in with Apple authorization. The input is borrowed and
 * copied to NSString before this function returns. It is not normalized,
 * retained by the C ABI, returned, or logged here. This does not start an
 * authorization request, show UI, validate a server session, or report
 * general Sign in with Apple or passkey readiness.
 *
 * A non-null completion is required. On an accepted iOS call, the function
 * returns FRAMEWORK_STATUS_OK and the completion runs once with the raw
 * signed credential-state value and a separate 0/1 NSError-presence bit.
 * NotFound may arrive with an error; preserve both arguments. Known values
 * are REVOKED=0, AUTHORIZED=1, NOT_FOUND=2, and TRANSFERRED=3. Other values
 * are passed through unchanged. Error domain, code, and localized text are
 * not exposed. A native error does not erase the returned state.
 *
 * The callback may run before this function returns or later on Apple's
 * unspecified callback queue. Do not assume main-queue delivery. The callback
 * must not unwind across C; keep its context valid through callback return.
 * The C ABI has no operation handle, result poll, destroy, timeout, or cancel
 * API. A successful start cannot be cancelled; keep context valid until the
 * callback returns. On a non-iOS target a valid request returns
 * FRAMEWORK_STATUS_UNSUPPORTED and does not call the completion. Invalid
 * input or a null completion returns FRAMEWORK_STATUS_INVALID_ARGUMENT and
 * does not call it. A caught synchronous Rust panic returns
 * FRAMEWORK_STATUS_PANIC.
 */
FrameworkStatus framework_ios_sign_in_with_apple_credential_state_start(
    FrameworkStr user_id,
    FrameworkIosSignInWithAppleCredentialStateCompletion completion,
    void *context);

#ifdef __cplusplus
}
#endif

#endif
