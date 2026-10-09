# PLAN_IOS_NEARBY_INTERACTION.md — Workstream B39: iOS Nearby Interaction Capability Query

## Objective

Implement the D34 precise-distance capability snapshot with the public Nearby Interaction framework API on iOS 16 and later. This workstream is status-only and does not implement an interaction session.

## Dependencies

- D34 `framework-nearby` portable contract
- Installed iOS SDK metadata and Apple primary documentation
- Generated `objc2-nearby-interaction` 0.3.2 bindings

## Read first

- `PLAN_CAPABILITIES_NEARBY_INTERACTION.md`
- `docs/OBJC_INTEROP.md`
- `docs/UNSAFE.md`
- [Apple `NISession.deviceCapabilities`](https://developer.apple.com/documentation/nearbyinteraction/nisession/devicecapabilities)
- [Apple `NIDeviceCapability`](https://developer.apple.com/documentation/nearbyinteraction/nidevicecapability)
- [Apple `NISession.isSupported`](https://developer.apple.com/documentation/nearbyinteraction/nisession/issupported)
- [Apple session authorization and setup](https://developer.apple.com/documentation/nearbyinteraction/initiating-and-maintaining-a-session)
- [Apple `NSNearbyInteractionUsageDescription`](https://developer.apple.com/documentation/BundleResources/Information-Property-List/NSNearbyInteractionUsageDescription)

## Write scope

- `platform/ios/ios-nearby/**`
- `docs/ios/nearby-interaction.md`

Do not edit the portable contract, root workspace configuration, root lockfile, capability status manifest, shared docs indexes, CI, or `tools/xtask`. The integrator owns workspace and manifest reconciliation.

## Required implementation

- Implement the portable backend with the generated `NISession.deviceCapabilities` and `NIDeviceCapability.supportsPreciseDistanceMeasurement` APIs
- Pin `objc2-nearby-interaction` to 0.3.2 with default features off and only `NISession` plus `NIDeviceCapability` enabled
- Require an iOS 16 or later host deployment target; the SDK declares both APIs available from iOS 16
- Exclude Mac Catalyst from the public iOS backend module
- Keep Objective-C calls in a small private module with an exact safety comment for each generated unsafe call
- Rely on the generated NearbyInteraction framework link; add no handwritten ABI shim, raw selector, session object, or Swift source
- Do not create/run a session, request permission, exchange tokens, discover peers, or start ranging
- Do not infer authorization, peer/configuration compatibility, session readiness, operation success, measurement quality, or background execution from the returned field
- Record that this query does not prompt; `NSNearbyInteractionUsageDescription` is for session start and is outside this query's requirement set
- State that no entitlement is configured or validated by this status-only adapter; do not infer host session/background configuration

## Validation and handoff

- Run target-specific `cargo check` and strict Clippy for `ios-nearby` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run the D34 portable tests and checks
- Inspect generated signatures, selected features, framework link, device/simulator imports, and the absence of session, token, permission, and ranging APIs
- Do not claim live device, simulator runtime, permission, session, or ranging proof
- Report changed files, exact commands, API floor, import/safety audit, deviations, and unresolved host limits
