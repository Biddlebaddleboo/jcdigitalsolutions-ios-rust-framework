# PLAN_BINDINGS_F21.md — F21: Sign in with Apple Credential-State C ABI

## Objective

Expose only B78's asynchronous query for one caller-provided, prior Sign in with Apple user ID. Preserve its state and independent NSError-presence bit. Do not add a portable authentication contract, sign-in flow, passkey operation, or cancellation claim.

## Status

F21 is integrated in the root C ABI feature graph, public exports, ABI manifest, Cargo.lock, macOS CI, aggregate plan, and docs index. The static gate passed Rust format, shell syntax, C11/C++17 header syntax, source/header symbol parity, and whitespace checks. The native gate passed arm64 device/Simulator Release archives, C11/C++17 links, exact imports, Objective-C class/selector/block symbols, the sole C export, forbidden Swift/authentication-flow/passkey symbols, and minos 13.0/14.0. No tests, consumer execution, runtime query, or linked-probe execution occurred.

## B78 surface and host assumptions

B78 provides `ios_sign_in_with_apple_status::get_credential_state(user_id, completion)` for `ASAuthorizationAppleIDProvider.getCredentialStateForUserID:completion:`. It copies the borrowed `&str` into `NSString` before the call returns, retains an escaping `RcBlock`, maps the four Apple cases, preserves unknown values as `Unknown(i64)`, and reports `NSError` presence separately as `had_error`. Its Rust completion is one-shot, `Send + 'static`, may run on any queue, may run before the initiating call returns, and cannot cancel AuthenticationServices work.

The native API floor is iOS 13.0 in `AuthenticationServices.framework`. The host must conservatively enable the Sign in with Apple capability and the `com.apple.developer.applesignin` entitlement (`Default` value). Apple does not state whether this query alone is exempt. No separate usage-description key appears in the reviewed query docs. This query accepts only an opaque user ID from a prior successful authorization; it is not a general readiness, authentication, server-session, or passkey check.

## Exact C contract

- Header: `framework_ios_sign_in_with_apple_status.h`
- Optional root Cargo feature to add: `ios-sign-in-with-apple-status`; target-iOS optional path dependency: `ios-sign-in-with-apple-status`
- Sole export: `FrameworkStatus framework_ios_sign_in_with_apple_credential_state_start(FrameworkStr user_id, FrameworkIosSignInWithAppleCredentialStateCompletion completion, void *context)`
- State ABI: signed `int64_t`; known values are `REVOKED=0`, `AUTHORIZED=1`, `NOT_FOUND=2`, `TRANSFERRED=3`; unknown raw signed values pass through unchanged
- Error ABI: `uint8_t had_error`, always 0 or 1. It is separate from state because `NotFound` can arrive with an `NSError`; the C layer does not expose NSError domain, code, or localized text
- Input: non-empty valid UTF-8 `FrameworkStr`; bytes are borrowed and immutable only through the synchronous start call; B78 copies the ID to `NSString`. Do not normalize, log, or return the identifier
- Callback: one non-null C function pointer and caller context. On accepted iOS start, callback may run inline before start returns or later on Apple's unspecified queue; no main-queue guarantee. Context must remain valid until callback return, and callback must not unwind across C
- Start statuses: malformed/empty UTF-8 or null callback is `INVALID_ARGUMENT`; valid non-iOS call is `UNSUPPORTED`; accepted iOS call returns `OK`; caught synchronous Rust panic is `PANIC`. Rejected start calls no callback. A native `NSError` is carried only as `had_error` with the state
- Ownership and lifecycle: B78's one-shot closure captures the C function pointer plus caller context address. No native object or C pointer is returned or retained by a C-visible handle. There is no result poll, destroy, timeout, or cancel export. A host that stops caring still must preserve context until callback return
- Limits: no general Sign in with Apple readiness, passkey status, sign-in, registration, UI, identity/token/session validation, entitlement exemption, query network/offline promise, or native error detail

## F21-owned files

- `bindings/c/src/ios_sign_in_with_apple_status.rs`
- `bindings/c/include/framework_ios_sign_in_with_apple_status.h`
- `bindings/c/check-ios-sign-in-with-apple-status.sh`
- `bindings/c/check-ios-sign-in-with-apple-status-link.sh`
- `docs/bindings/ios-sign-in-with-apple-status.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, `PLAN_BINDINGS.md`, capability matrix, and documentation indexes. No B78 backend file changes are in F21.

## Static gate and root integration

`sh bindings/c/check-ios-sign-in-with-apple-status.sh` runs only Rust formatting, shell syntax, header C11/C++17 syntax checks, source/header export-name parity, and F21 whitespace checks. `sh bindings/c/check-ios-sign-in-with-apple-status-link.sh` builds the feature-enabled `framework-c-api` release archive for arm64 iOS device and Simulator targets, then links C11 and C++17 consumers without running them. It checks exact `AuthenticationServices`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` imports, with `libc++.1.dylib` allowed only for C++; the C export, Objective-C class/selector imports, block runtime imports, forbidden Swift/authentication-flow/passkey symbols, and final Mach-O minos (device 13.0, Simulator 14.0) are also checked. The gate uses the integrated root Cargo feature, dependency, ABI module/export, manifest, and lockfile. The static syntax and native link/import gates passed after root integration. Consumers and probes were linked/inspected only, never executed.

Root integration state and remaining steps:

1. The current root checkout already has the optional target-iOS dependency and feature, cfg module and public re-export, ABI manifest entry, and refreshed root Cargo.lock.
2. Root added both F21 static and native link/import scripts to the macOS C ABI job.
3. Root recorded the passing device and Simulator link/import/minos evidence in `PLAN_BINDINGS.md`. Keep the separate iOS API floor at 13.0 and C Simulator link minimum at 14.0.

Do not promote canonical row 021 to imply passkey or full Sign in with Apple implementation. Any partial row status must name only this entitlement-scoped query for a previously authenticated user.
