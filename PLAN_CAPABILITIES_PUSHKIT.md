# PLAN_CAPABILITIES_PUSHKIT.md — Workstream D70: Row 033 Feasibility Gate

## Status

D70 found no nonprompting, status-only PushKit operation that represents system or service readiness. The available token lookup only reads a token cached after successful registration; it does not report current registration, APNs reachability, delivery readiness, or app eligibility. Keep capability row `033-notifications-background-pushkit` at `X`. No implementation or matrix change is in scope.

PushKit has a real service boundary: an app must retain and configure `PKPushRegistry`, register desired push types with PushKit, receive and maintain device-token callbacks, and rely on an APNs provider server. For VoIP pushes, the host must also handle incoming payload callbacks and report calls through CallKit or LiveCommunicationKit when required. A local Boolean or status enum would misstate these requirements.

## Objective

Assess the smallest public Rust-callable PushKit slice for row 033, including whether a read-only support/status query can stand alone without a live VoIP push service.

## Installed SDK and API evidence

Inspected Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5. Relevant public headers are:

- `System/Library/Frameworks/PushKit.framework/Headers/PKPushRegistry.h`
- `System/Library/Frameworks/PushKit.framework/Headers/PKDefines.h`
- `System/Library/Frameworks/PushKit.framework/Headers/PKVoIPPushMetadata.h`

The SDK declares these API floors:

- `PKPushRegistry` and `initWithQueue:`: iOS 8.0.
- `PKPushTypeVoIP`: iOS 9.0. This is the floor for a VoIP-specific PushKit surface; it differs from the base registry floor.
- `pushRegistry:didReceiveIncomingPushWithPayload:forType:withCompletionHandler:`: iOS 11.0.
- `pushRegistry:didReceiveIncomingVoIPPushWithPayload:metadata:withCompletionHandler:` and `PKVoIPPushMetadata.mustReport`: iOS 26.4.

The iOS 13 SDK-link condition in Apple's callback docs is not an iOS deployment floor. For apps linked against the iOS 13 SDK or later, a received VoIP call must be reported to CallKit; the system terminates an app that fails to report, and repeated failures may stop future VoIP launches. The iOS 26.4 callback adds metadata: when `mustReport` is false, the app need not report that push to CallKit or LiveCommunicationKit. A `mustReport` true push still requires a report.

`PKPushRegistry` docs require a registry at every app launch, foreground or background, with its delegate assigned before `desiredPushTypes` is set. The app normally retains the registry for its runtime. Setting `desiredPushTypes` initiates asynchronous registration with PushKit; successful registration delivers `PKPushCredentials` to the delegate. `pushTokenForType:` returns the locally cached token or `nil` if none is cached. The delegate also receives token invalidation and incoming-push callbacks. The installed delegate protocol does not expose a named registration-failure callback, although the registry's overview describes asynchronous success/failure reporting; a future implementation must resolve that documentation/API detail rather than invent an error callback.

The docs.rs listing for `objc2-push-kit` 0.3.2 exposes generated bindings for `PKPushRegistry`, `PKPushRegistryDelegate`, `PKPushCredentials`, `PKPushPayload`, and `PKPushTypeVoIP`. Binding availability alone does not provide a status-only API: a Rust backend still needs a retained Objective-C registry/delegate, callback ownership, token handling, host launch integration, and completion-handler behavior. A future implementation must inspect the exact selected crate features and compile against its chosen SDK before it claims a supported backend.

## Status-query feasibility

No public static PushKit support, authorization, or registration-status API appears in the inspected SDK or Apple API docs.

- Setting `desiredPushTypes` is not a query. It configures the registry and starts server registration.
- Reading `desiredPushTypes` only reports the configured set on that registry; it does not report successful registration or service readiness.
- `pushTokenForType:` is a token lookup on a configured registry. A cached token proves only that PushKit previously delivered a token for that type. `nil` does not distinguish unsupported type, no prior registration, registration failure, invalidation, or other unavailable state.
- `UIApplication.isRegisteredForRemoteNotifications` belongs to the APNs/UserNotifications registration surface; it is not a PushKit VoIP status query and must not be substituted.

Therefore a status-only capability slice is not viable without the real PushKit registration/service contract. A token-presence API could be a later, explicitly named cache snapshot within that contract, but it must not be labeled PushKit availability, permission, registration, or live-delivery status.

## Host, entitlement, and APNs obligations

Apple documents the APS Environment entitlement as required for PushKit and UserNotifications. Its exact signed entitlement key is `com.apple.developer.aps-environment`, with `development` or `production` values. Xcode adds it when the host enables the Push Notifications capability. The inspected Apple docs do not identify a separate PushKit-only or VoIP-only app entitlement; do not invent one.

A VoIP host also declares the `voip` value in its `UIBackgroundModes` array when it provides Voice over IP background service. This is host Info.plist configuration, not an entitlement. Apple lists Voice over IP as a background execution mode, and CallKit documents missing `voip` in `UIBackgroundModes` as a common cause of an unentitled transaction error. Applicability must be validated for the concrete host and CallKit product; this plan does not claim the value alone grants background execution.

PushKit tokens are opaque device tokens for a push type. Apple requires the host to send a token to its provider server, protect it, and update server state when credentials change or a token is invalidated. A real VoIP provider sends APNs requests with `apns-push-type: voip` and the app bundle identifier plus `.voip` as `apns-topic`. Token-based APNs provider auth is supported; if using certificate auth, the provider needs a VoIP Services certificate. PushKit registration without a provider server cannot deliver an app's service pushes.

There is no PushKit usage-description or user authorization prompt key in the inspected API. Registering is nonprompting but is not side-effect-free: setting desired types asks PushKit/APNs to register the app and produces a device token for service use.

## Callback and UI boundary

PushKit's `voIP` type is only for initiating live voice calls. Apple's public guidance says to use UserNotifications instead when the app does not present the system call interface. For iOS 13 SDK-linked apps, the incoming VoIP path must report a call promptly through CallKit. The newer iOS 26.4 metadata callback permits omission only when `mustReport` is false; it does not remove the host's call/service lifecycle requirements for calls that must be reported.

A Rust-callable backend would need a safe, retained delegate that can receive callbacks on the configured queue, copy or otherwise bound the lifetime of token/payload data, handle credential replacement and invalidation, invoke the OS completion block exactly once, and hand reportable calls to a live CallKit/LiveCommunicationKit provider. The host must create/configure the registry on every launch, including background launch. None of these duties fit a portable scalar availability API.

## Feasibility result and next evidence

Do not implement a portable contract or iOS status backend as a proxy for PushKit readiness. Keep row 033 at `X` until a product requires actual remote VoIP push delivery and the host/service owner accepts the full lifecycle.

Before a future PushKit implementation:

1. Define a real VoIP call product and its CallKit or LiveCommunicationKit call lifecycle; if no system call UI is intended, use UserNotifications instead of PushKit.
2. Confirm the app's signed `com.apple.developer.aps-environment`, provisioning profile, and `UIBackgroundModes` `voip` configuration for its product and signing setup.
3. Provide an APNs provider/server contract, token storage and revocation path, `.voip` topic, and selected provider authentication mode.
4. Select a minimum iOS version separately for `PKPushRegistry`, `PKPushTypeVoIP`, completion callbacks, and optional iOS 26.4 metadata handling.
5. Audit the selected `objc2-push-kit` version/features, Objective-C delegate ownership, callback queue, completion exactly-once behavior, host launch integration, and CallKit/LiveCommunicationKit handoff on device and Simulator.
6. Resolve registration-failure observability: the SDK overview describes asynchronous failure, but the inspected delegate protocol has no named failure method.
7. Validate runtime registration, APNs delivery, background launch, CallKit reporting, and token invalidation only in a real app/service environment; compile/link checks alone cannot establish delivery behavior.

## Deferred work

- No `framework-notifications` changes, portable status contract, PushKit backend, APNs provider, token storage, CallKit/LiveCommunicationKit integration, host entitlements, `UIBackgroundModes`, or background lifecycle code.
- No Swift, C ABI, live push registration, APNs request, call presentation, device probe, or tests.
- No canonical capability manifest, Cargo manifest/lockfile, CI, aggregate plan, or shared-index edit.

## Apple and binding references

- [Supporting PushKit Notifications in Your App](https://developer.apple.com/documentation/pushkit/supporting-pushkit-notifications-in-your-app)
- [PKPushRegistry](https://developer.apple.com/documentation/pushkit/pkpushregistry)
- [PKPushRegistryDelegate](https://developer.apple.com/documentation/pushkit/pkpushregistrydelegate)
- [PKPushType.voIP](https://developer.apple.com/documentation/pushkit/pkpushtype/voip)
- [Responding to VoIP Notifications from PushKit](https://developer.apple.com/documentation/pushkit/responding-to-voip-notifications-from-pushkit)
- [PKVoIPPushMetadata](https://developer.apple.com/documentation/pushkit/pkvoippushmetadata)
- [APS Environment entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/aps-environment)
- [UIBackgroundModes](https://developer.apple.com/documentation/bundleresources/information-property-list/uibackgroundmodes)
- [Configuring background execution modes](https://developer.apple.com/documentation/xcode/configuring-background-execution-modes)
- [CallKit unentitled transaction error](https://developer.apple.com/documentation/callkit/cxerrorcoderequesttransactionerror/code/unentitled)
- [Sending notification requests to APNs](https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns)
- [Certificate-based APNs connections](https://developer.apple.com/documentation/usernotifications/establishing-a-certificate-based-connection-to-apns)
- [`objc2-push-kit` 0.3.2](https://docs.rs/objc2-push-kit/0.3.2/objc2_push_kit/)

## B212 follow-up: iOS 26.4 VoIP `mustReport` is callback-scoped

Rechecked the installed iOS 26.5 SDK and published `objc2-push-kit` 0.3.2 for a narrow Rust-callable capability beyond the existing status-query audit. The SDK adds `PKVoIPPushMetadata.mustReport` as a readonly `BOOL` on iOS 26.4 and adds `pushRegistry:didReceiveIncomingVoIPPushWithPayload:metadata:withCompletionHandler:` at the same floor. The property answers only whether the app must report a call or live conversation for that received VoIP push; it is not a PushKit registration, support, authorization, or delivery-status query.

The exact public declarations are in
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/PushKit.framework/Headers/PKVoIPPushMetadata.h`
and `PKPushRegistry.h`. The callback supplies both `PKVoIPPushMetadata` and a completion handler. The SDK documents that failure to report when `mustReport == YES` can terminate the app and repeated failures may stop later VoIP delivery; `mustReport == NO` permits omitting a report for that callback. This meaning exists only within incoming-push handling and cannot be safely represented as a standalone or persistent scalar capability.

The published [`objc2-push-kit` 0.3.2 API listing](https://docs.rs/objc2-push-kit/0.3.2/objc2_push_kit/) exposes `PKPushRegistry`, `PKPushRegistryDelegate`, `PKPushCredentials`, `PKPushPayload`, and VoIP push-type declarations, but not `PKVoIPPushMetadata` or the iOS 26.4 metadata callback. That binding gap is additional friction, not the primary no-go: even a hand-authored typed binding would still require the actual PushKit registry/delegate lifecycle, callback-lifetime metadata access, exactly-once completion, APNs/provider state, and CallKit or LiveCommunicationKit handoff when required. No VoIP product/service scope is selected here, so adding a facade for `mustReport` alone would not provide a truthful Rust capability.

Decision: no implementation or dependency change. Keep row `033-notifications-background-pushkit` at `X`. Revisit only with an explicit VoIP product that owns the registry/delegate lifecycle and can consume the metadata inside the incoming callback; do not expose `mustReport` as current readiness or authorization.

Evidence: installed iOS 26.5 SDK headers `PKVoIPPushMetadata.h` and `PKPushRegistry.h`; [Apple `PKVoIPPushMetadata.mustReport`](https://developer.apple.com/documentation/pushkit/pkvoippushmetadata/mustreport?language=objc), [Apple `PKPushRegistryDelegate`](https://developer.apple.com/documentation/pushkit/pkpushregistrydelegate), [Apple `PKPushRegistry`](https://developer.apple.com/documentation/pushkit/pkpushregistry), and the [`objc2-push-kit` 0.3.2 API listing](https://docs.rs/objc2-push-kit/0.3.2/objc2_push_kit/).

No source, dependency, build, link probe, test, app launch, Simulator run, device query, registry creation, push registration, callback, or live manager call was performed for B212

## B285 follow-up: `desiredPushTypes` is local registration intent only

Audited one distinct PushKit candidate beyond D70's status/token review and B212's callback-only
`PKVoIPPushMetadata.mustReport`: reading `PKPushRegistry.desiredPushTypes` from an already-owned
registry. The installed iOS 26.5 header declares a nullable read/write `NSSet<PKPushType> *`
instance property. It is not a static API and does not expose a server registration result

Apple documents the setter semantics: assigning `desiredPushTypes` asks the PushKit server to
register those types asynchronously, and the success/failure result arrives through the registry's
delegate. Apple also requires the app to set a valid delegate before modifying this property and
normally to create and retain a registry at every app launch. Therefore a getter can report only
the local set requested on that particular registry. It cannot establish successful PushKit
registration, a current token, APNs reachability, delivery readiness, entitlement approval, or a
working VoIP service. Creating a new registry solely to read the property does not recover another
registry's state and introduces the launch/delegate lifecycle

Decision: no B285 Rust API, dependency, or Cargo.lock change. Keep row
`033-notifications-background-pushkit` at `X`. A Rust wrapper named for configured push types would
still require a host-owned registry handle and would expose configuration rather than a useful
system capability or readiness result. This is distinct from B212's per-incoming-push `mustReport`
value and does not revise D70's status/token boundary

Evidence: Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5,
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/PushKit.framework/Headers/PKPushRegistry.h`;
[Apple `desiredPushTypes`](https://developer.apple.com/documentation/pushkit/pkpushregistry/desiredpushtypes?changes=_5_8&language=objc),
[Apple `PKPushRegistry`](https://developer.apple.com/documentation/pushkit/pkpushregistry),
[Apple PushKit registration guidance](https://developer.apple.com/documentation/pushkit/supporting-pushkit-notifications-in-your-app),
and the existing [D70/B212 audit](PLAN_CAPABILITIES_PUSHKIT.md). The upstream `objc2-push-kit`
0.3.2 API listing is documented at <https://docs.rs/objc2-push-kit/0.3.2/objc2_push_kit/>; this
audit adds no crate dependency. Evidence is limited to Xcode 26.6 / SDK 26.5 and retains the
repository's Xcode 27.x baseline caveat

No source, dependency, build, link probe, test, app launch, Simulator run, device query, registry
creation, push registration, or live manager call was performed for B285
