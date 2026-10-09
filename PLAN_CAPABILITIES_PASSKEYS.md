# PLAN_CAPABILITIES_PASSKEYS.md — Workstream D66: Row 021 Feasibility Gate

## Status

D66 found no row-021 slice that meets the original non-prompting, non-entitled, identity-free boundary. The B78 follow-up identifies a conditional Sign in with Apple credential-state slice for a caller-supplied prior user ID; it is not passkey status and is not entitlement-free. B78 adds an isolated iOS adapter and compile/import gates for that subset; see [PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md](PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md). Root now records row 021 as `B`/partial only for this entitlement-scoped query; passkeys and general authentication remain unsupported. Do not treat the B78 package as passkey or general authentication support. B72 and G66 remain unstarted.

## Objective

Assess whether passkeys or Sign in with Apple expose one honest, non-prompting status/query operation that needs no entitlement, credential secret, user identity, or authorization UI. Exclude sign-in, registration, assertion, credential listing, and browser-app-only passkey access.

## B78 follow-up — Sign in with Apple credential state only

`ASAuthorizationAppleIDProvider.getCredentialState(forUserID:completion:)` is a bounded, non-interactive query for one opaque Sign in with Apple user identifier previously returned by a successful authorization. It is separate from passkey availability, passkey credential access, and the iOS 26.2 web-browser passkey manager. It cannot report whether Sign in with Apple is generally available, and it cannot authenticate a caller or validate a server session.

This is a conditional future partial candidate only if the row's host contract is explicitly narrowed to Sign in with Apple credential-state snapshots for a previously authenticated user and the host enables the Sign in with Apple capability. Apple documents `com.apple.developer.applesignin` (array of strings, normal value `Default`) as the entitlement that lets an app use Sign in with Apple, but does not say whether the status method alone is exempt. Therefore do not claim this is entitlement-free. The original no-entitlement boundary remains unsupported; a host without the entitlement has no documented support claim for this operation.

### Operation and value semantics

- The public Objective-C method and enum are available from iOS 13.0 in `AuthenticationServices.framework`. It accepts the caller's opaque `userID` and a completion block. It does not create an authorization request or call `ASAuthorizationController`; Apple's implementation guide demonstrates checking a saved credential at app launch and presenting sign-in UI only as a separate follow-up when appropriate.
- Preserve all four states: `Authorized`, `Revoked`, `NotFound`, and `Transferred`. Apple defines `NotFound` as no established Sign in with Apple relationship; `Transferred` means the app moved to another team and the identifier must be migrated. The SDK header notes that `NotFound` is accompanied by an `NSError`; preserve state and error separately instead of collapsing `NotFound` into an error-only result. The generated binding models the enum as a transparent `NSInteger` wrapper, so a future portable representation should use fixed-width values and preserve unknown raw cases rather than exposing the Objective-C enum layout.
- The result is only the credential state for the supplied identifier. It exposes no name, email, identity token, authorization code, or passkey state. Treat the identifier as sensitive, caller-owned user data; do not log it or include it in diagnostics.
- The reviewed method docs describe no prompt, permission dialog, privacy usage-description key, or separate per-query consent. This does not remove the host Sign in with Apple entitlement boundary. Do not infer that an `Authorized` state proves an application server session or validates any token. The API docs also do not specify whether lookup requires network access; make no offline-availability claim.

### Callback, ownership, and thread contract

- Apple's Objective-C header declares `void (^)(ASAuthorizationAppleIDProviderCredentialState, NSError * _Nullable)` and gives no callback queue or cancellation API. The Apple Swift overlay presents an async-throwing form and marks the completion closure `@Sendable`; neither source promises main-queue delivery. The sample dispatches to the main queue before UI work, so a Rust wrapper must accept callback delivery on an arbitrary queue and must not call UI directly from it.
- Upstream `objc2-authentication-services` 0.3.2 exposes `ASAuthorizationAppleIDProvider::getCredentialStateForUserID_completion` as `unsafe fn (&self, user_id: &NSString, completion: &DynBlock<dyn Fn(ASAuthorizationAppleIDProviderCredentialState, *mut NSError)>)`, gated by `block2`; provider construction is also `unsafe`, and the provider binding is `!Send` and `!Sync`. The callback uses a raw nullable error pointer. Copy the status and any selected error classification before returning from the block; do not retain `NSError` or borrowed callback data in the portable result.
- For an asynchronous API, use an escaping heap-owned block (for example, `block2::RcBlock`) with owned callback state. Do not use a stack or borrowing block for this operation. Since the queue is unspecified, the callback's captured state must be safe for arbitrary-thread invocation or be marshalled through an explicit caller-supplied scheduler. Do not move or share the `ASAuthorizationAppleIDProvider` object across threads.
- The API exposes no cancel handle. A future operation must state that dropping a caller-side receiver does not cancel AuthenticationServices work; the owned callback context must remain valid until the framework releases or invokes its block. Do not promise a timeout, cancellation, main-queue callback, or exact completion timing beyond the documented completion-handler result.
- Keep errors separate from the four credential states. The Objective-C header explicitly allows `NotFound` plus an error, while Apple documents an optional error generally. A semantic result must preserve both fields or map the error into a stable framework-owned category without exposing platform objects or localized text.

### Feasibility disposition

The API path is technically Rust-callable through an upstream generated Objective-C binding and has no Swift-only ABI blocker. A useful slice can be a single asynchronous query for an existing Sign in with Apple credential with fixed-width state/error values and explicit arbitrary-queue, no-cancel semantics. It does not satisfy D66's entitlement-free or identity-free requirement. Root may promote only this explicitly entitlement-scoped subset after dependency integration and safe callback-lifecycle gates; passkey support and general Sign in with Apple login remain unsupported.

## Evidence inspected

- Xcode 26.6 build 17F113, iPhoneOS SDK 26.5 public headers under `AuthenticationServices.framework/Headers`.
- `ASAuthorizationAppleIDProvider.h` declares `ASAuthorizationAppleIDProviderCredentialState` and `getCredentialStateForUserID:completion:` from iOS 13.0. The method checks one opaque user ID previously given to the app. Apple documents that ID as the `user` value from a successful authorization credential; `NotFound` also passes an error. This is not a global Sign in with Apple availability query.
- `getCredentialStateForUserID:completion:` does not call `ASAuthorizationController` or initiate a credential request, so it is a status check rather than a sign-in prompt. Apple does not document whether this query alone requires `com.apple.developer.applesignin`; the conservative host contract must require the Sign in with Apple entitlement if this operation is ever implemented. The callback queue and cancellation behavior are unspecified, but an adapter can document arbitrary-queue delivery and no cancellation rather than assuming a queue or promising cancellation.
- Apple documents `com.apple.developer.applesignin` as the entitlement for using Sign in with Apple, but the available docs do not state whether the credential-state-only query is exempt.
- `ASAuthorizationWebBrowserPublicKeyCredentialManager.h` declares `isDeviceConfiguredForPasskeys` as a class Boolean property from iOS 26.2. The containing class is available from iOS 17.4 and is explicitly for web-browser passkey access. Apple’s browser guidance uses this manager to report/request browser access to passkeys; the browser access state is `Authorized`, `Denied`, or `NotDetermined`.
- The available Apple docs do not establish whether `isDeviceConfiguredForPasskeys` is callable without `com.apple.developer.web-browser` or `com.apple.developer.web-browser.public-key-credential`, nor whether its value means general device setup or browser credential-use eligibility. It cannot yet be reported as a general passkey-availability fact.
- `objc2-authentication-services` 0.3.2 exposes `ASAuthorizationAppleIDProvider::getCredentialStateForUserID_completion` behind `block2`; its generated signature uses `DynBlock<dyn Fn(ASAuthorizationAppleIDProviderCredentialState, *mut NSError)>` and is `unsafe`. The docs.rs source shows a transparent `NSInteger` enum with `Revoked = 0`, `Authorized = 1`, `NotFound = 2`, and `Transferred = 3`. This crate source is not present in the local Cargo registry or current repository dependency graph, so those binding facts are from the published 0.3.2 generated source and still need a package-local compile gate before implementation. Its generated `ASAuthorizationWebBrowserPublicKeyCredentialManager` API exposes `authorizationStateForPlatformCredentials`, request authorization, and credential fetch; it does not expose `isDeviceConfiguredForPasskeys`.
- Do not bridge the missing class property with a raw selector or hand-written ABI. Current generated API coverage is incomplete for that candidate.

The iOS 26.5 SDK header's completion comment says “one of 3 possible states,” but the same header enum has four declared cases including `Transferred`. The current Apple credential-state documentation defines all four. Use the four-case enum and preserve unknown raw values; do not copy the stale three-state comment into a portable contract.

## Public API boundary

- General app passkey flows need a relying-party identifier, a server challenge, and an authorization request/controller; Apple’s passkey guide also requires the `webcredentials` associated-domain service for registration/assertion. The request/controller path has user-facing authorization and callback lifecycle and is outside this status-only slice.
- `ASAuthorizationControllerRequestOptionPreferImmediatelyAvailableCredentials` (iOS 16.0) is a request option. It does not replace authorization flow with a noninteractive status query.
- The web-browser credential manager is for browser-app access to passkeys. Its authorization-state property is not a general passkey-capability query; requesting access can present system UI.
- The Apple ID credential-state call can only report a caller-supplied, previously obtained user identifier. It cannot answer whether Sign in with Apple is generally available or whether an unknown user can sign in.
- The existing `framework-auth` / `ios-auth` scope covers LocalAuthentication only and explicitly excludes passkeys and Sign in with Apple. Do not extend those packages as part of this feasibility record.

## Exact evidence needed before implementation

1. If the desired row scope remains entitlement-free, Apple documentation or Apple framework guidance must confirm whether `getCredentialStateForUserID:completion:` alone is exempt from `com.apple.developer.applesignin`. Otherwise, any future implementation must explicitly require that host entitlement and must not call itself non-entitled.
2. Before code, locally resolve and compile `objc2-authentication-services` 0.3.2 with the narrow `ASAuthorizationAppleIDProvider` and `block2` features. Implement an owned escaping callback state and test callback teardown/lifetime separately; document arbitrary-queue delivery and no cancellation. Do not wait for an undocumented main-queue guarantee or invent a cancel API.
3. Apple must clarify whether `isDeviceConfiguredForPasskeys` is available without either web-browser entitlement and specify exactly what its Boolean represents. A generated `objc2-authentication-services` binding must expose the iOS 26.2 class property before Rust calls it.
4. Any future implementation must keep the iOS 13 Apple ID credential-state query separate from the iOS 26.2 browser-manager property; neither may imply general passkey support, an authenticated server session, or a valid identity token.

## Deferred work

- B72: no iOS adapter is implemented or authorized by this report; a future explicitly entitlement-scoped status slice is technically feasible but needs separate root scope approval and callback lifecycle gates.
- G66: no compile/link gate is authorized until a supported operation and exact dependency feature set are selected.
- No portable `framework-auth` contract change, entitlement claim, row-status change, workspace/lockfile edit, CI edit, or shared index edit is part of D66.

## B78 inspection record

- Installed Xcode is 26.6 build 17F113 with iPhoneOS SDK 26.5. `AuthenticationServices.framework/Headers/ASAuthorizationAppleIDProvider.h` declares the provider, enum, and method with iOS 13.0 availability; the method signature takes `NSString *` and a completion block receiving the enum and nullable `NSError *`.
- `objc2-authentication-services` 0.3.2 is available as an upstream generated binding, but its source is not present in the local Cargo registry or current repository dependency graph. The published binding uses the provider feature and `block2`; with `default-features = false`, the package should select only those plus the required Foundation features. No compile gate ran.
- Inspected Apple's current provider, method, credential-state, implementation, and entitlement documentation, plus the generated 0.3.2 binding source and `block2` 0.6.2 ownership docs. The method docs do not state a callback queue or cancellation operation. The Apple Swift overlay exposes an async-throwing method with an `@Sendable` completion; the UIKit example dispatches to main before UI work.
- No credential-state query, tests, builds, or probes ran. Only this scoped plan was edited; no code, Cargo/workspace, CI, matrix, or root index changed.

## Apple and binding references

- [ASAuthorizationAppleIDProvider](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider)
- [getCredentialState(forUserID:completion:)](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/getcredentialstate%28foruserid%3Acompletion%3A%29)
- [ASAuthorizationAppleIDProviderCredentialState](https://developer.apple.com/documentation/authenticationservices/asauthorizationappleidprovider/credentialstate?language=objc)
- [Sign in with Apple entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.applesignin)
- [Implementing user authentication with Sign in with Apple](https://developer.apple.com/documentation/authenticationservices/implementing-user-authentication-with-sign-in-with-apple)
- [ASAuthorizationWebBrowserPublicKeyCredentialManager](https://developer.apple.com/documentation/authenticationservices/asauthorizationwebbrowserpublickeycredentialmanager)
- [Passkey use in web browsers](https://developer.apple.com/documentation/authenticationservices/passkey-use-in-web-browsers)
- [isDeviceConfiguredForPasskeys](https://developer.apple.com/documentation/authenticationservices/asauthorizationwebbrowserpublickeycredentialmanager/isdeviceconfiguredforpasskeys)
- [Web-browser public-key-credential entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser.public-key-credential)
- [`ASAuthorizationAppleIDProvider` in objc2-authentication-services 0.3.2](https://docs.rs/objc2-authentication-services/0.3.2/objc2_authentication_services/struct.ASAuthorizationAppleIDProvider.html)
- [`ASAuthorizationAppleIDProvider` generated source in objc2-authentication-services 0.3.2](https://docs.rs/objc2-authentication-services/0.3.2/src/objc2_authentication_services/generated/ASAuthorizationAppleIDProvider.rs.html)
- [`ASAuthorizationWebBrowserPublicKeyCredentialManager` in objc2-authentication-services 0.3.2](https://docs.rs/objc2-authentication-services/0.3.2/objc2_authentication_services/struct.ASAuthorizationWebBrowserPublicKeyCredentialManager.html)
- [`RcBlock` in block2 0.6.2](https://docs.rs/block2/0.6.2/block2/struct.RcBlock.html)
