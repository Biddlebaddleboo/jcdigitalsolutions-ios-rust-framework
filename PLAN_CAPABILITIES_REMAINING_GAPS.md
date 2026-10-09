# PLAN_CAPABILITIES_REMAINING_GAPS.md — D100: Residual X-Row Candidate Audit

## Objective and scope

D100 began as a read-only review of rows marked `X` in `docs/capabilities/capability-status.json`, across canonical rows 001–113. Its snapshot had 32 X rows before B76 and 31 after B76 moved row 108 out of X. Root integrations B77, B78, and B79 then moved rows 113, 021, and 098 to scoped `B` partials, leaving 28 current `X` rows. This plan now records the disposition and evidence for those follow-ups; see the current manifest for exact scope

### Root follow-up after the D100 snapshot

B77 moved row 113 to `B`/partial for one caller-selected extension metadata read, and B78 moved row 021 to `B`/partial for an entitlement-scoped credential-state query. The canonical matrix now has 28 `X` rows. B79 has implemented and root integrated the D100 Personal VPN candidate for row 098 as a scoped `B` partial after its package gates passed. Row 033 remains `X`; no PushKit service was selected.

## Disposition

At its original audit snapshot, D100 identified three bounded follow-ups through the repo's existing `objc2`/`block2` architecture. Two are now implemented as partials:

1. **Implemented partial — row `098-extension-entitlement-capabilities-networkextension-vpn`:** B79 exposes caller-app Personal VPN profile status only, via `NEVPNManager` iOS 8.0 APIs, after preference load. The host requires the `allow-vpn` entitlement; custom providers and system-wide VPN state remain unsupported. See `PLAN_IOS_VPN_STATUS.md`
2. **Implemented partial — row `021-security-auth-passkeys-sign-in-with-apple`:** B78 exposes credential state for one prior Sign in with Apple user ID, via `ASAuthorizationAppleIDProvider.getCredentialState(forUserID:completion:)` from iOS 13.0. The host conservatively requires the Sign in with Apple entitlement; passkey support, general sign-in availability, and entitlement-free querying remain unsupported. See `PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md`
3. **Product-only candidate — row `033-notifications-background-pushkit`:** real VoIP PushKit registration/token callback service via `PKPushRegistry`, not a static status query. It can change coverage only for a concrete VoIP product with an APNs provider, launch-time registry/delegate retention, signed push entitlement, background configuration, token lifecycle, and CallKit reporting. Do not implement a token-presence Boolean as PushKit readiness

No X row is ready for a broad support claim. B78 and B79 remain partials, and each row's capability text/reason must stay limited to its exact implemented slice

## Repo-proven Rust / Objective-C route

The repo has a proven Rust-owned Objective-C route that does not need Swift source or a Swift ABI call:

- `Cargo.lock` pins `objc2` 0.6.5 and `block2` 0.6.2
- `platform/ios/ios-file-provider/src/platform.rs` calls a generated Objective-C binding with an escaping `RcBlock`, an `Arc` completion state, `autoreleasepool`, and panic containment. Its package gate compiles device and Simulator targets, runs strict Clippy/docs checks, and links an import/symbol probe
- `platform/ios/ios-cloud/src/cloudkit.rs` uses `RcBlock`, `Arc`, `Mutex`, and a future to bridge a native callback that may arrive on a background queue
- `platform/ios/ios-notification-responses/src/platform.rs` implements an Objective-C delegate with `define_class!`, `ProtocolObject`, an explicitly retained delegate handle, callback queue caveats, panic containment, and exactly-once native completion
- The local validation pattern is documented in `platform/ios/ios-file-provider/check.sh` and `platform/ios/ios-file-provider/check-link-imports.sh`; this pattern checks no Swift, Rust target compilation, framework imports, and selected symbols

This proves a reusable Rust-to-Objective-C/C method, block, delegate, and link-audit route. B78 and B79 subsequently added and gated `objc2-authentication-services` and `objc2-network-extension`; their compile/link evidence and runtime limits are in the linked workstream plans. `objc2-push-kit` remains outside the workspace and `Cargo.lock`; no PushKit binding, service contract, or product scope is selected

## Ranked candidate details

| D100 candidate / row | Narrow API and floor | Integrated outcome and exact limits |
| --- | --- | --- |
| Implemented — `098-extension-entitlement-capabilities-networkextension-vpn` | `NetworkExtension.framework`: `NEVPNManager.sharedManager()`, `loadFromPreferencesWithCompletionHandler:`, `connection.status`, `NEVPNStatus`; iOS 8.0 | B79 preserves six status cases, unknown values, and preference-load errors; the host requires the `allow-vpn` entitlement. Device/Simulator compile, Clippy, rustdoc, feature, and link/import gates passed; probes were inspected, not executed. No VPN query ran. No mutation, tunnel control, provider extension, routing, reachability, or all-device VPN claim. See [B79](PLAN_IOS_VPN_STATUS.md) |
| Implemented — `021-security-auth-passkeys-sign-in-with-apple` | `AuthenticationServices.framework`: `ASAuthorizationAppleIDProvider.getCredentialState(forUserID:completion:)`; iOS 13.0 | B78 accepts one opaque prior user ID, preserves known/unknown status and error presence, and permits any callback queue. The host conservatively requires the Sign in with Apple entitlement. Device/Simulator compile, Clippy, rustdoc, feature, and import gates passed; no query ran. No sign-in flow, passkey support, global readiness, or identity proof. See [B78](PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md) |
| Product-only — `033-notifications-background-pushkit` | `PushKit.framework`: `PKPushRegistry.init(queue:)`, `delegate`, `desiredPushTypes`, token and callback methods; base registry iOS 8.0, `PKPushTypeVoIP` iOS 9.0, incoming-push callback iOS 11.0 | This is a real VoIP service, not a status query. It requires an accepted product scope, APNs provider, launch-time registry/delegate retention, signed push entitlement, background configuration, token lifecycle, and CallKit reporting. No source or package is integrated; see D70 and Apple's PushKit references above |

## D82 follow-up implemented by B79

B79 uses `NEVPNManager.sharedManager()`, loads preferences asynchronously, and reads `connection.status` only after a successful callback. The Rust surface is a main-thread-bound future; dropping it detaches Rust interest but does not cancel NetworkExtension work. The exact generated `objc2-network-extension` 0.3.2 APIs and callback/thread constraints compiled under Rust 1.94.1, and the focused device/Simulator compile, strict Clippy, rustdoc, feature-tree, and link/import gates passed. Device minos is 10.0 and arm64 Simulator minos is 14.0; the API floor remains iOS 8.0. Probes were inspected, not executed. See [B79](PLAN_IOS_VPN_STATUS.md) for implementation details and imports

B78 implements the row-021 credential-state subset using `objc2-authentication-services` 0.3.2 and `block2`; its entitlement-scoped device/Simulator compile, strict Clippy, rustdoc, and import gates passed, with no live query. See [B78](PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md)

Row 033 remains a real PushKit service candidate only if a VoIP product scope is selected. A static token-presence Boolean would not establish registration or readiness. No PushKit dependency, source, package, or implementation gate is integrated

## Screened X-row set and exclusions
The current audited `X` row set has 28 rows. Rows 021, 098, and 113 moved to `B` for the exact B78, B79, and B77 partials, and row 108 moved to `B` in B76:

`033-notifications-background-pushkit`, `036-notifications-background-backgroundassets-where-applicable`, `042-sensors-connectivity-sensorkit`, `044-sensors-connectivity-accessorysetupkit`, `045-sensors-connectivity-thread-network-system-pieces`, `067-ml-vision-language-foundation-models-residual-through-c-where-available`, `072-personal-data-system-stores-homekit`, `074-personal-data-system-stores-screentime-familycontrols-base-authorization`, `075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists`, `084-cloud-accounts-communication-pushtotalk`, `085-cloud-accounts-communication-carplay`, `089-commerce-services-adattributionkit-adservices-as-applicable`, `091-commerce-services-weatherkit-through-rest-native-http-where-appropriate`, `092-commerce-services-tipkit-only-if-system-tipkit-behavior-is-specifically-requested-otherwise-framework-owned-tip-logic-may-be-portable`, `095-maps-ar-spatial-realitykit-native-swift-residual-coverage-that-is-realistically-supportable`, `097-maps-ar-spatial-dockkit`, `100-extension-entitlement-capabilities-extensionkit-foundation-where-feasible`, `101-extension-entitlement-capabilities-browserenginekit-for-entitled-apps`, `102-extension-entitlement-capabilities-contactprovider`, `103-extension-entitlement-capabilities-managedapp-distribution`, `104-extension-entitlement-capabilities-marketplacekit`, `105-extension-entitlement-capabilities-mattersupport`, `106-extension-entitlement-capabilities-secureelementcredential`, `107-extension-entitlement-capabilities-carkey`, `109-extension-entitlement-capabilities-lockedcameracapture`, `110-compiler-build-host-capabilities-app-intents-use-result-of-c-stage-0-1-if-unsupported-expose-no-fake-runtime-implementation`, `111-compiler-build-host-capabilities-widgetkit-support-management-data-logic-available-through-proven-interfaces-do-not-implement-a-swiftui-clone-merely-to-claim-full-rendering-support`, `112-compiler-build-host-capabilities-activitykit-support-if-layer-2-abi-work-is-proven`

The other X rows at the D100 snapshot were not elevated to implementation candidates in D100; this is historical screening context:

- **Swift-only / no proven Rust ABI route:** `067`, `089`, `091`, `092`, `095`, `097`, `100`, `102`, `103`, `104`, `105`, `106`, `107`, `110`, `111`, `112`. The audited APIs are Swift-only, use Swift async/protocol/view surfaces, or have no truthful small Rust status subset. The relevant focused reports include `PLAN_CAPABILITIES_FOUNDATION_MODELS.md`, `PLAN_CAPABILITIES_WEATHERKIT.md`, `PLAN_CAPABILITIES_TIPKIT.md`, `PLAN_CAPABILITIES_REALITYKIT.md`, `PLAN_CAPABILITIES_DOCKKIT.md`, `PLAN_CAPABILITIES_EXTENSIONKIT.md`, `PLAN_CAPABILITIES_CONTACTPROVIDER.md`, `PLAN_CAPABILITIES_MANAGEDAPP.md`, `PLAN_CAPABILITIES_MARKETPLACEKIT.md`, `PLAN_CAPABILITIES_MATTERSUPPORT.md`, `PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md`, `PLAN_CAPABILITIES_CARKEY.md`, `PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md`, `PLAN_CAPABILITIES_APP_INTENTS.md`, `PLAN_CAPABILITIES_WIDGETKIT.md`, and `PLAN_SWIFT_ABI.md`
- **Direct-looking query lacks honest scope or has narrow status with special gates:** `042`, `044`, `045`, `072`, `074`, `075`, `085`, `109`. These audits found a research-only entitlement, user-facing setup, an entitlement-gated Thread query that does not establish local Thread capability, a HomeKit manager that can prompt, an unverified Family Controls status, an undocumented DeviceActivity Boolean, no global CarPlay readiness query, or no LockedCameraCapture readiness query. See `PLAN_CAPABILITIES_SENSORKIT.md`, `PLAN_CAPABILITIES_ACCESSORY.md`, `PLAN_CAPABILITIES_THREAD.md`, the D33 HomeKit result in `PLAN_CAPABILITIES.md`, `PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md`, `PLAN_CAPABILITIES_DEVICE_ACTIVITY.md`, `PLAN_CAPABILITIES_CARPLAY.md`, and `PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md`
- **Host/service/extension lifecycle dominates the slice:** `036`, `084`, `101`, `113`. BackgroundAssets needs an actual asset lifecycle; PushToTalk needs channel, audio, push, and entitlement setup; BrowserEngineKit needs an entitled browser/extension process model; extension metadata helpers need a documented Xcode mechanism and real host target contract

## Root integration status

B77, B78, and B79 are integrated as scoped `B` partials for rows 113, 021, and 098. The canonical matrix has 28 `X` rows; row 033 remains `X` because no PushKit product scope was selected. No further source, matrix, CI, aggregate-plan, or index change is needed for D100. The exact current unsupported set and per-row reason live in `docs/capabilities/capability-status.json`
