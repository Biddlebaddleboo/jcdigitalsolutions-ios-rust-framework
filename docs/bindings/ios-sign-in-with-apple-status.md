# iOS Sign in with Apple credential-state C ABI

This opt-in C API starts one query for a previously obtained Sign in with Apple user ID. It wraps B78's `ios-sign-in-with-apple-status::get_credential_state` only. It does not start sign-in or registration, show authorization UI, validate a server session, expose a token, or query passkey readiness.

The native API floor is iOS 13.0 in `AuthenticationServices.framework`. The host must enable Sign in with Apple and include the `com.apple.developer.applesignin` entitlement with the normal `Default` value. Apple does not state whether the credential-state query alone is exempt, so this C guide keeps the conservative entitlement requirement. The reviewed API docs name no separate usage-description key.

## Call and callback

`framework_ios_sign_in_with_apple_credential_state_start` accepts a non-empty UTF-8 `FrameworkStr`, one non-null C completion, and a context pointer. The caller's user-ID bytes need remain readable only through the start call; B78 copies them to `NSString` before return. The ID is sensitive caller data. Do not log it or put it in the context's diagnostics.

On valid iOS input the function returns `FRAMEWORK_STATUS_OK`; the completion then receives a fixed-width signed raw state and a separate `had_error` byte. Known states are `REVOKED` (0), `AUTHORIZED` (1), `NOT_FOUND` (2), and `TRANSFERRED` (3). Any future raw value passes through unchanged. `NOT_FOUND` may coexist with `had_error=1`. The C ABI preserves only error presence, not `NSError` domain, code, or text.

The completion may run inline before start returns or later on Apple's unspecified queue. It is not a main-queue callback. The host must prepare context before start, keep it valid until callback return, and must not unwind across C. Start rejection (`INVALID_ARGUMENT`, `UNSUPPORTED`, or synchronous `PANIC`) calls no completion. The callback fires once for the accepted query if AuthenticationServices supplies a result. No handle, poll, destroy, timeout, or cancellation API exists; abandoning caller work cannot cancel the native query.

The status is a point-in-time answer for one prior identifier only. It does not establish identity, prove an app server session, report general service readiness, or imply passkey support. B78 and F21 do not supply a portable authentication contract.

## Integration boundary

`framework_ios_sign_in_with_apple_status.h` remains opt-in. Root must add the target-iOS optional dependency and Cargo feature, gate this source module and its public re-exports, add one ABI manifest entry, refresh the shared lock, add the named static/build gate to macOS CI, and link this guide from the root binding index. Those root-owned edits are out of F21 scope.

See [B78's package guide](../../platform/ios/ios-sign-in-with-apple-status/README.md), [B78's evidence plan](../../PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md), Apple's [credential-state query](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/getcredentialstate%28foruserid%3Acompletion%3A), [state values](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/credentialstate?language=objc), and [Sign in with Apple entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.applesignin).
