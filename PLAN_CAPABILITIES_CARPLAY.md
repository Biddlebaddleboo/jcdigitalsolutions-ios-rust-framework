# PLAN_CAPABILITIES_CARPLAY.md — Workstream D84: Row 085 Feasibility Gate

## Status

D84 found no honest CarPlay-wide status or scalar query that can stand alone without a CarPlay-entitled host and its scene/session lifecycle. The public value APIs describe a connected CarPlay system, not device support, entitlement approval, or whether a CarPlay session is active. Keep row `085-cloud-accounts-communication-carplay` at `X`; no implementation or matrix change is part of D84.

## Objective

Assess whether row 085 has a narrow public Rust-callable support/status/value operation that does not require a CarPlay-authorized app category, CarPlay scene configuration, or live CarPlay session lifecycle.

## Installed SDK and generated binding evidence

- Inspected Xcode 26.6 build `17F113` and the iPhoneOS SDK 26.5 at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/CarPlay.framework/Headers`.
- The public headers expose no `isAvailable`, `isSupported`, `isConnected`, or CarPlay-device support query. The only plausible scalar-value surface is `CPSessionConfiguration`: `initWithDelegate:` and `limitedUserInterfaces` are available from iOS 12.0; `contentStyle` and its delegate change callback are available from iOS 13.0; `supportsVideoPlayback` is available from iOS 26.4. The latter says only that the connected CarPlay system supports video playback, not that CarPlay is generally present or usable by this host.
- `CPTemplateApplicationScene` and `CPTemplateApplicationSceneDelegate` are available from iOS 13.0. The SDK describes the scene as CarPlay-managed and requires the host to supply a scene configuration. The scene provides `CPInterfaceController` on connection; navigation apps use a separate connection callback with `CPWindow`. A connection callback is evidence of a live scene, not a preflight capability query.
- The installed public scene manifest role is `CPTemplateApplicationSceneSessionRoleApplication`. A host uses it in `UIApplicationSceneManifest` / `UISceneConfigurations`, with `UISceneClassName` set to `CPTemplateApplicationScene` and its own `UISceneDelegateClassName`.
- `objc2-car-play` 0.3.2 was inspected from its generated source after metadata fetch with `cargo info objc2-car-play@0.3.2`. `CPSessionConfiguration` is feature-gated by `CPSessionConfiguration` and binds `initWithDelegate(this: Allocated<Self>, delegate: &ProtocolObject<dyn CPSessionConfigurationDelegate>) -> Retained<Self>`, `limitedUserInterfaces(&self) -> CPLimitableUserInterface`, and `contentStyle(&self) -> CPContentStyle` as unsafe Objective-C calls. Its delegate protocol is `MainThreadOnly` and has optional change callbacks. `CPTemplateApplicationScene` is feature-gated by `CPTemplateApplicationScene` plus `objc2-ui-kit`; it binds scene properties and delegate callbacks, not an availability query. The generated API feature set does not add a host entitlement, scene manifest, or CarPlay session.
- The CarPlay framework umbrella header imports UIKit and the CarPlay template, controller, session configuration, and scene APIs. No standalone C/Objective-C query was found that reports CarPlay device support or active connection without entering the scene/session model.

## Entitlement, host, and privacy boundary

- Apple requires a CarPlay-enabled app to request the entitlement for its app category through CarPlay Contact Us, agree to the CarPlay Entitlement Addendum, and receive Apple's review/managed capability before adding the entitlement to the App ID, provisioning profile, and signed target.
- Apple's current entitlement table names `com.apple.developer.carplay-audio`, `com.apple.developer.carplay-communication`, `com.apple.developer.carplay-charging`, `com.apple.developer.carplay-maps`, `com.apple.developer.carplay-parking`, and `com.apple.developer.carplay-quick-ordering`. These category-specific grants are host signing configuration, not a runtime user permission and not facts a Rust query can infer.
- The host must add the appropriate CarPlay scene role and delegate configuration to its `Info.plist` scene manifest. CarPlay creates and disconnects the scene as the user and system interact with the vehicle. The host must implement the scene lifecycle and use the interface controller/templates; navigation apps also receive a managed window for map drawing.
- No CarPlay privacy usage-description key or user authorization prompt was found in the inspected headers/docs. This does not remove the entitlement approval, signed-host configuration, system-driven connection, and app-review requirements.
- `CPSessionConfiguration.limitedUserInterfaces`, `contentStyle`, and `supportsVideoPlayback` report properties of the connected vehicle environment. They do not establish CarPlay hardware support in general, an entitlement grant, host eligibility, scene launch permission, or CarPlay connection readiness. A default or absent session value cannot be mapped to a truthful `available`/`unavailable` status without a documented no-session contract.

## Feasibility result and next evidence

Do not add a portable contract, iOS status backend, or stand-alone CarPlay support Boolean for row 085. The candidate session values have meaningful semantics only as part of a connected CarPlay environment, and obtaining that state depends on Apple's category entitlement and CarPlay-managed host scene/session lifecycle. Exporting a constructed `CPSessionConfiguration` value without that integration would imply a supported runtime state that the API does not promise.

If a future product explicitly requires CarPlay integration, scope it as host UI/session support rather than a generic availability query. Before implementation:

1. Name the eligible CarPlay app category and obtain/verify its exact managed entitlement and provisioning profile.
2. Define the supported scene role, `Info.plist` manifest, delegate ownership, connect/disconnect behavior, main-thread requirements, and host launch lifecycle.
3. Choose one session value such as `limitedUserInterfaces` or `contentStyle`; preserve unknown native option bits and do not relabel the value as device support or authorization.
4. Audit the selected `objc2-car-play` features and verify the deployment floor separately for each chosen symbol; do not claim the iOS 26.4 video property is part of a lower-baseline API.
5. Validate actual vehicle/CarPlay Simulator scene connection and category entitlement in an eligible signed host. Compile/link evidence alone cannot validate entitlement approval or live-session values.

## Deferred work

- No `framework-ui` changes, portable CarPlay contract, iOS backend, template UI, navigation, audio/communication, dashboard, instrument cluster, `CPSessionConfiguration` wrapper, or Swift/C ABI work.
- No entitlements, provisioning profile, host scene manifest, CarPlay usage-description key, live vehicle session, runtime probe, tests, or builds.
- No canonical capability manifest, Cargo/workspace/lockfile, CI, aggregate plan, or shared-index edit.

## Apple and binding references

- [Requesting CarPlay Entitlements](https://developer.apple.com/documentation/carplay/requesting-carplay-entitlements)
- [Displaying Content in CarPlay](https://developer.apple.com/documentation/carplay/displaying-content-in-carplay)
- [CPTemplateApplicationScene](https://developer.apple.com/documentation/carplay/cptemplateapplicationscene)
- [CPTemplateApplicationSceneDelegate](https://developer.apple.com/documentation/carplay/cptemplateapplicationscenedelegate)
- [CPSessionConfiguration](https://developer.apple.com/documentation/carplay/cpsessionconfiguration)
- [CPSessionConfigurationDelegate](https://developer.apple.com/documentation/carplay/cpsessionconfigurationdelegate)
- [CPSessionConfiguration.supportsVideoPlayback](https://developer.apple.com/documentation/carplay/cpsessionconfiguration/supportsvideoplayback)
- [`objc2-car-play` 0.3.2](https://docs.rs/objc2-car-play/0.3.2/objc2_car_play/)
- [Generated `CPSessionConfiguration` binding](https://docs.rs/objc2-car-play/0.3.2/src/objc2_car_play/generated/CPSessionConfiguration.rs.html)
- [Generated `CPTemplateApplicationScene` binding](https://docs.rs/objc2-car-play/0.3.2/src/objc2_car_play/generated/CPTemplateApplicationScene.rs.html)
