# PLAN_CAPABILITIES_CALLKIT.md — Workstream D76: Row 083 Active-Call Snapshot

## Result

A narrow iOS-only, Rust-callable CallKit slice is implemented: a caller-requested snapshot of the active calls returned by `CXCallObserver.calls`, mapped to a framework-owned count and aggregate fixed-width state flags. Apple documents that any app may create a `CXCallObserver`; the snapshot API has no documented permission prompt, usage-description key, or entitlement requirement. The Objective-C API floor is iOS 10.0.

This is call-activity observation, not CallKit readiness, permission, call placement, incoming-call delivery, audio, caller identity, or a complete calling backend. The `calls` getter may block while the system supplies its initial state, so this package's API docs direct callers to keep it off UI-critical work. The snapshot has no callback lifecycle and exposes no CallKit object, UUID, or caller data. Root owns row 083 status and integration.

## Objective

Assess a small Rust-accessible CallKit surface for row `083-cloud-accounts-communication-callkit`, including API availability, binding support, host configuration, consent, and callback/lifecycle requirements.

## Installed SDK and API evidence

Inspected Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5.

- `CallKit.framework/Headers/CXCallObserver.h` declares `CXCallObserver` and `CXCallObserverDelegate` with `API_AVAILABLE(ios(10.0), macCatalyst(13.0), watchos(9.0))` and `API_UNAVAILABLE(macos, tvos)`.
- `CallKit.framework/Headers/CXCall.h` declares `CXCall` with the same availability. `CXCall` exposes `UUID`, `isOutgoing`, `isOnHold`, `hasConnected`, and `hasEnded`; direct `init` is unavailable.
- The binding's generated module declares `#[link(name = "CallKit", kind = "framework")]`; `CallKit.framework` is the only required Apple framework for this observer snapshot.
- `CXCallObserver.calls` is a copied `NSArray<CXCall *>` property. The header says it retrieves the current call list and may block on initial state retrieval.
- `setDelegate:queue:` is optional for a one-shot snapshot. Its header says the delegate is weakly stored; a `nil` queue means the main queue.
- Apple describes `CXCallObserver` as a programmatic interface for active calls, and says any app can create one to observe system call activity. Its `calls` property returns the active calls of the telephony provider.

The generated `objc2-call-kit` 0.3.2 binding exists on docs.rs and is now resolved through the root workspace lockfile. Its relevant generated surface is:

- `CXCallObserver` requires crate feature `CXCallObserver`; `calls` is `pub unsafe fn calls(&self) -> Retained<NSArray<CXCall>>` and is gated by feature `CXCall`.
- `CXCall` requires crate feature `CXCall`; its `UUID`, `isOutgoing`, `isOnHold`, `hasConnected`, and `hasEnded` accessors are generated as `pub unsafe fn` methods.
- `new()` is also generated as `unsafe`. The binding types are `!Send` and `!Sync`.
- Delegate setup uses `setDelegate_queue`, gated by `dispatch2`, and the binding marks the method and `CXCallObserverDelegate` trait `unsafe`. The protocol feature is not needed for a one-shot snapshot.
- The crate is `no_std`, but its default feature set enables almost all CallKit surfaces plus `block2`, `dispatch2`, `objc2-avf-audio`, and `std`. A future narrow dependency should use `default-features = false` and only `CXCallObserver` and `CXCall`, plus the required `objc2-foundation` types/features. Confirm exact feature closure with the selected lockfile and target build.

## Implemented snapshot contract

The iOS-only `active_call_snapshot()` API creates a `CXCallObserver`, reads `calls`, and copies the count and aggregate state flags into a framework-owned `CallActivitySnapshot`, then releases the Objective-C objects. The value reports a `u64` active-call count and a `u32` flag mask for whether any returned call is outgoing, connected, on hold, or ended. The flags are independent ORed facts; they do not describe or identify one call.

Do not expose `CXCall.UUID` by default. Apple documents it as the identifier used to address calls in other CallKit APIs; an observer-only slice does not need that control identifier. Do not add phone numbers, handles, contact names, audio state, histories, persistent logs, or call-control actions. The snapshot describes only what `CXCallObserver.calls` returns at query time and may be stale as soon as it returns.

The observer's `calls` property can block during its initial state read. The sync API documents that behavior and directs callers to keep it off UI-critical work. The implementation keeps `CXCallObserver` and `CXCall` on the thread where it creates and uses them; the generated binding types are not `Send` or `Sync`. It copies primitive values before those objects leave scope. No callback, observer registration, or persistent native handle is used for this one-shot form.

If a later API opts into change notifications, it must retain the delegate for the entire observation period because CallKit stores it weakly, respect the callback queue, and copy callback values before return. A `nil` queue delivers callbacks on the main queue; a supplied queue also needs an owner for its lifetime. That callback API is separate scope, not part of the one-shot snapshot.

## Consent, entitlement, and host configuration

Apple's `CXCallObserver` documentation says any app can create the observer; the inspected observer/call declarations identify no authorization request, usage-description key, or entitlement. No CallKit prompt is part of the snapshot contract. This does not assert that a calling app, Call Directory extension, or VoIP service has no separate host requirements.

The `com.apple.developer.calling-app` entitlement is for an app that elects to support the default calling-app setting on iOS and iPadOS 18.2 or later. Apple's default-calling-app setup also requires `UIBackgroundModes` to contain `voip` for that product and submission path. Neither requirement applies to the observer-only snapshot.

A full VoIP calling app is out of scope. Apple's call guide describes a retained `CXProvider`, a provider delegate for system actions and audio-session changes, `CXCallController` transactions, service signaling, and `AVAudioSession` coordination. Incoming calls may depend on PushKit notification callbacks; the related PushKit/VoIP host and APNs obligations are separate from `CXCallObserver` and are not inferred from this slice.

The returned data still reveals active call activity. A host should query only for an explicit product need and avoid logging or persistence by default. The observer does not identify callers through `CXCall`; no permission prompt is not a privacy-policy claim.

## Implementation and validation

The crate at `platform/ios/callkit/ios-call-observer` calls the generated `CXCallObserver` and `CXCall` bindings. It computes the active-call count and ORs four state flags across the returned list. It never reads `CXCall.UUID`, creates a delegate, or retains a CallKit object after the function returns. The crate is an explicit root workspace member and uses the root `Cargo.lock`; `objc2-call-kit` 0.3.2 has default features disabled and only `CXCallObserver` and `CXCall` enabled. The root workspace manifest and lockfile are integration-owned.

The focused `check.sh` runs host and iOS device/Simulator `cargo check`, device/Simulator strict Clippy, iOS rustdoc, format, dependency feature-isolation checks, and the link-import probe below. The device/Simulator release probes are separate target binaries and are inspected but never executed.

The link-import gate is `sh platform/ios/callkit/ios-call-observer/check-link-imports.sh`. It builds only `ios_call_observer_link_probe` in release mode for `aarch64-apple-ios` with `IPHONEOS_DEPLOYMENT_TARGET=10.0` and `aarch64-apple-ios-sim` with `IPHONEOS_DEPLOYMENT_TARGET=14.0`, then inspects each final Mach-O with `otool -L`, `nm -u`, `strings`, and `vtool -show-build`. Both builds passed. Both final import lists exactly match `platform/ios/callkit/ios-call-observer/link-imports-expected.txt`: `CallKit`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`. Both import `_objc_getClass` and `_objc_msgSend`; both contain the `CXCallObserver`, `calls`, `isOutgoing`, `hasConnected`, `isOnHold`, and `hasEnded` class/selector strings, and the gate rejects Swift runtime, call-control, caller-identity, audio, PushKit, and UI symbols/strings. `vtool` reports final `minos 10.0` for the device binary and `minos 14.0` for the Simulator binary. The iOS 10.0 API floor is separate from the 14.0 Simulator link target.

No tests or runtime probes were run, and neither link-probe binary was executed. This validates framework load commands and selected Objective-C imports, not live call visibility or behavior on a real telephony device. The root matrix, workspace manifest/lockfile, CI, and shared docs indexes are integration-owned; they are not changed by this focused package gate.

The package implements an isolated iOS-only partial slice. Root may classify row 083 as `B` partial after review and central integration. It does not claim all CallKit APIs or a full calling service.

## Deferred work

- No portable contract, provider or controller, delegate, callback, PushKit integration, APNs service, audio session, call action, Call Directory extension, default-calling-app support, or device runtime probe.
- No canonical capability manifest, CI, aggregate plan, shared docs index, or matrix edit. Root integration owns workspace membership and the shared lockfile.
- No tests, linked binary execution, or runtime probes. The only release binaries built are the two link/import probes described above.

## Apple and binding references

- [CallKit overview](https://developer.apple.com/documentation/callkit)
- [CXCallObserver](https://developer.apple.com/documentation/callkit/cxcallobserver)
- [CXCallObserver.calls](https://developer.apple.com/documentation/callkit/cxcallobserver/calls)
- [CXCall](https://developer.apple.com/documentation/callkit/cxcall?language=objc)
- [CXCallObserver.setDelegate(_:queue:)](https://developer.apple.com/documentation/callkit/cxcallobserver/setdelegate%28_%3Aqueue%3A%29)
- [Making and receiving VoIP calls](https://developer.apple.com/documentation/callkit/making-and-receiving-voip-calls)
- [Preparing your app to be the default calling app](https://developer.apple.com/documentation/callkit/preparing-your-app-to-be-the-default-calling-app)
- [Default Calling App entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.calling-app)
- [Supporting PushKit notifications](https://developer.apple.com/documentation/pushkit/supporting-pushkit-notifications-in-your-app)
- [`objc2-call-kit` 0.3.2 crate](https://docs.rs/objc2-call-kit/0.3.2/objc2_call_kit/)
- [`CXCallObserver` binding](https://docs.rs/objc2-call-kit/0.3.2/objc2_call_kit/struct.CXCallObserver.html)
- [`CXCall` binding](https://docs.rs/objc2-call-kit/0.3.2/objc2_call_kit/struct.CXCall.html)
- [`CXCallObserverDelegate` binding](https://docs.rs/objc2-call-kit/0.3.2/objc2_call_kit/trait.CXCallObserverDelegate.html)
