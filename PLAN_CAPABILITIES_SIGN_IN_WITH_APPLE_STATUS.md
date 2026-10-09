# B78: Sign in with Apple credential-state query

## Disposition

The narrow query is implementable through the published Rust binding and local iOS toolchain. The package at `platform/ios/ios-sign-in-with-apple-status/` calls only `ASAuthorizationAppleIDProvider.getCredentialStateForUserID:completion:`. Its device and Simulator compile, strict Clippy, rustdoc, and Mach-O import gates pass. This does not support passkeys, sign-in, registration, authorization UI, or general authentication readiness.

Root integrated this as a row 021 `B` partial for the credential-state query only. Apple does not say whether the query alone is exempt from the Sign in with Apple entitlement, so the package and canonical matrix take a conservative host requirement: enable Sign in with Apple and include `com.apple.developer.applesignin` with the normal `Default` value. This is a conditional entitlement-scoped capability, not the original entitlement-free row scope.

## API boundary

- Apple declares `ASAuthorizationAppleIDProvider`, the credential-state enum, and `getCredentialStateForUserID:completion:` from iOS 13.0 in `AuthenticationServices.framework`.
- The method checks one opaque user ID from a prior successful Sign in with Apple authorization. It does not begin an authorization request and cannot establish a new user, validate a token, check a server session, or report general Sign in with Apple availability.
- The generated binding is `objc2-authentication-services = 0.3.2`. Its method signature is `pub unsafe fn getCredentialStateForUserID_completion(&self, user_id: &NSString, completion: &DynBlock<dyn Fn(ASAuthorizationAppleIDProviderCredentialState, *mut NSError)>)`, gated by `block2`. The Swift overlay marks the completion `@escaping @Sendable`; no queue or cancellation guarantee is documented.
- The package maps `Revoked`, `Authorized`, `NotFound`, and `Transferred` to Rust values, and preserves other raw `NSInteger` cases as `Unknown(i64)`. It carries `NSError` presence separately as `had_error`; the package does not copy error domain, code, or localized text. `NotFound` may coexist with an error.
- The callback is `FnOnce + Send + 'static`, may run on any queue, and may run before the call returns. `RcBlock` owns the escaping block; its capture uses `Arc<Mutex<Option<Completion>>>` to accept arbitrary-queue delivery and ignore repeat callbacks. Rust panics do not unwind through the Objective-C callback. The framework exposes no cancel operation; dropping downstream work does not cancel the query.
- The input user ID is copied to `NSString` for the call and is not returned or logged by this package. The output exposes no Objective-C object, identity token, authorization code, email, or user ID.
- No separate usage-description key appears in the Apple docs checked for this query. The lack of a cited key is not a claim that the entitlement is optional.

## Binding and package boundary

`platform/ios/ios-sign-in-with-apple-status/Cargo.toml` selects these direct dependencies and features:

- `objc2 = 0.6.5`, `std`
- `block2 = 0.6.2`, `alloc`
- `objc2-authentication-services = 0.3.2`, `ASAuthorizationAppleIDProvider`, `block2`
- `objc2-foundation = 0.3.2`, `NSError`, `NSString`

The package is an iOS adapter only. It defines no portable `framework-auth` contract and uses no Swift or raw selector/ABI call. The direct `platform/ios/*` workspace glob picks up the package. Root owns the workspace lock refresh and any root feature, facade, matrix, CI, or index wiring.

## Validation

`platform/ios/ios-sign-in-with-apple-status/check.sh` passed with exit code 0. Root added it to macOS CI; it runs these non-test gates:

- `cargo check --locked --offline -p ios-sign-in-with-apple-status --target aarch64-apple-ios`
- `cargo clippy --locked --offline -p ios-sign-in-with-apple-status --target aarch64-apple-ios -- -D warnings`
- The same `cargo check` and strict Clippy commands for `aarch64-apple-ios-sim`
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline -p ios-sign-in-with-apple-status --target aarch64-apple-ios --no-deps`
- `platform/ios/ios-sign-in-with-apple-status/check-link-imports.sh`

The link/import gate builds but does not execute `examples/credential_state_link_probe.rs`. Both final Mach-O images import exactly `AuthenticationServices`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; both contain the provider class string, `getCredentialStateForUserID:completion:` selector, `__Block_copy`, `__Block_release`, `_objc_getClass`, and `_objc_msgSend`. The gate rejects Swift, authorization-controller/request, and passkey imports.

`vtool -show-build` reports `minos 13.0` for `aarch64-apple-ios` and `minos 14.0` for `aarch64-apple-ios-sim`. The API floor remains iOS 13.0; the arm64 Simulator target/link floor observed with Xcode 26.6 build 17F113 and iOS SDK 26.5 is 14.0. Setting `IPHONEOS_DEPLOYMENT_TARGET=13.0` did not lower the Simulator Mach-O minos.

`cargo metadata --manifest-path Cargo.toml --no-deps --format-version 1` succeeded and listed the package as a root workspace member. No tests, credential-state query, or linked-probe execution took place. No shared lock, root manifest, CI, matrix, or root index was edited by this workstream.

## Root integration

The package is selected by the root `platform/ios/*` workspace glob, and root refreshed the shared lock after package integration. The canonical row now records a platform-exclusive `B` partial, AuthenticationServices, iOS 13.0, and the conservative Sign in with Apple entitlement. No root portable `framework-auth` API exists. Passkeys and general Sign in with Apple authentication remain unsupported; the query-only entitlement exemption remains undocumented.

## Apple and binding sources

- [ASAuthorizationAppleIDProvider](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider)
- [getCredentialState(forUserID:completion:)](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/getcredentialstate%28foruserid%3Acompletion%3A%29)
- [ASAuthorizationAppleIDProviderCredentialState](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/credentialstate?language=objc)
- [Sign in with Apple entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.applesignin)
- [Implementing user authentication with Sign in with Apple](https://developer.apple.com/documentation/authenticationservices/implementing-user-authentication-with-sign-in-with-apple)
- [`ASAuthorizationAppleIDProvider` in objc2-authentication-services 0.3.2](https://docs.rs/objc2-authentication-services/0.3.2/objc2_authentication_services/struct.ASAuthorizationAppleIDProvider.html)
- [Generated provider binding source](https://docs.rs/objc2-authentication-services/0.3.2/src/objc2_authentication_services/generated/ASAuthorizationAppleIDProvider.rs.html)
- [`RcBlock` in block2 0.6.2](https://docs.rs/block2/0.6.2/block2/struct.RcBlock.html)
