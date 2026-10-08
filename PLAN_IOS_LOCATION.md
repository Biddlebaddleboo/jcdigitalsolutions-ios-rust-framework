# PLAN_IOS_LOCATION.md — Workstream B5: iOS Current-Location Backend

## Objective

Implement the iOS backend for the integrated D4 one-shot location contract with public Core Location APIs. Do not add continuous location or background behavior.

## Dependencies

- Foundation A and iOS runtime B are integrated
- D4 `framework-location` contract is integrated
- Inspect installed iOS SDK metadata before any availability claim

## Read first

- `PLAN_CAPABILITIES_LOCATION.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- [Apple `CLLocationManager` documentation](https://developer.apple.com/documentation/corelocation/cllocationmanager)
- [Apple `requestLocation()` documentation](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestlocation%28%29?changes=__2&language=objc)

## Write scope

- `platform/ios/ios-location/**`
- `docs/ios/location.md`

Do not edit the portable D4 crate, root workspace configuration, capability status manifest, other iOS backends, Swift ABI, C bindings, or unrelated capability families. The orchestrator adds workspace membership and updates the shared manifest after integration.

## Required implementation

- Implement `LocationBackend` with caller-owned backend state and no global service registry
- Use public Core Location Objective-C APIs through the smallest supported binding surface; keep dependency types out of the portable API
- Use the native one-shot location request path and preserve its returned accuracy/error semantics; do not promise a requested accuracy target
- Query authorization without prompting; request only the portable contract's explicit foreground authorization level
- Contain delegate callbacks with exact-once terminal completion and safe retained ownership; handle callback reentrancy and future drop per the D4 contract
- Respect Core Location's delegate run-loop requirements; document the callback thread and main-thread/run-loop requirements
- Do not request temporary full accuracy, Always authorization, background location, or unrelated location services
- State `Info.plist` permission text requirements and native status/error mapping
- Do not add `.swift` source, private API, background modes, PushKit, or runtime/global registry

## Validation and handoff

- Run `cargo check` and Clippy for `ios-location` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add deterministic tests for pure authorization, accuracy, and native-error mappings that do not need a live permission prompt
- Inspect device and simulator imports for CoreLocation and absence of unrelated capability frameworks and Swift runtime
- Document minimum iOS version only from installed SDK metadata, permission text, callback/run-loop behavior, location age/accuracy semantics, cancellation, native handles, and live-device test limits
- Report changed files, commit SHA, exact checks, deviations, and unresolved assumptions
