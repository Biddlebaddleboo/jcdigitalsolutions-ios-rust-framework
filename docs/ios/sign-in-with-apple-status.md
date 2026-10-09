# Sign in with Apple credential status

`ios-sign-in-with-apple-status` queries `ASAuthorizationAppleIDProvider.getCredentialStateForUserID:completion:` for one opaque user ID from a prior successful Sign in with Apple authorization. It maps Apple's four documented states, preserves unknown integer states, and reports `NSError` presence separately. It does not return the user ID or native objects.

This is not passkey support, an authorization or registration flow, UI, token validation, server-session validation, or general Sign in with Apple readiness. The host must conservatively enable Sign in with Apple and include `com.apple.developer.applesignin` with its normal `Default` value; Apple does not document whether the query alone is exempt. The callback may arrive on any queue or inline, and the API exposes no cancellation operation.

The API floor is iOS 13.0 in `AuthenticationServices.framework`. Device and arm64 Simulator compile, strict Clippy, rustdoc, and Mach-O import checks passed with minos 13.0/14.0; the linked probe and credential-state query were not executed, and no tests were run. See [B78](../../PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md), [D66](../../PLAN_CAPABILITIES_PASSKEYS.md), and the [package guide](../../platform/ios/ios-sign-in-with-apple-status/README.md).
