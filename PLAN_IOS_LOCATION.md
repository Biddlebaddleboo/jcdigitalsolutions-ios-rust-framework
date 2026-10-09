# PLAN_IOS_LOCATION.md — Workstream B5: iOS Current-Location Backend

## Status

B5's `ios-location` backend and guide are integrated. Device/simulator `cargo check`, strict
all-target Clippy, and link/import probes pass; exact direct imports are `CoreLocation`,
`Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`, with Swift/Python and selected unrelated
capability imports rejected. Six host tests pass for authorization, accuracy, and native-error
mapping. Probe binaries were linked, not run; no live authorization prompt, GPS fix, cancellation
race, or device runtime is claimed. The host has Xcode 26.6 / SDK 26.5, below the Xcode 27.x plan
baseline. The follow-up's `sh platform/ios/ios-location/check-link-imports.sh` passed and checks the
locked device and Simulator Cargo trees for exactly `CLLocation`, `CLLocationManager`, and
`CLLocationManagerDelegate`, plus the direct-import allowlist. Probe binaries were inspected, not
executed. The gate emitted a rustc warning that `IPHONEOS_DEPLOYMENT_TARGET` was 9.0 while rustc
supports a minimum of 10.0; final device probe minos is 10.0 and Simulator minos is 14.0.

B87 adds `IosLocationBackend::accuracy_authorization()` as an iOS-only, synchronous, non-prompting
snapshot query; it does not change portable D4 or row 037 status. Installed
`iPhoneOS26.5.sdk`'s `CLLocationManager.h` declares readonly selector
`-[CLLocationManager accuracyAuthorization]` and `CLAccuracyAuthorization` as
`NS_ENUM(NSInteger)` with full=0 and reduced=1, available from iOS 14.0. Installed
`objc2-core-location` 0.3.2 provides the typed getter returning its transparent
`CLAccuracyAuthorization(pub NSInteger)` wrapper. The query returns `None` before iOS 14.0, maps
the two known values, and preserves any other signed raw value as `Unknown(isize)`. The existing
backend remains iOS 9.0+; this extension creates a manager and reads the property synchronously on
the backend's main thread, without assigning a delegate or starting a service. Apple's Core
Location docs give no separate queue requirement for this property read; they specify an active
creation-thread run loop for delegate callbacks, which B87 does not add. Apple also cautions that
full-accuracy status should be interpreted alongside authorization status, since it can be
reported while authorization is not determined. B87 checks passed: Rust format, device and
Simulator `cargo check`, strict Clippy, device rustdoc, `sh platform/ios/ios-location/check-link-imports.sh`,
docs-check, and `git diff --check`. No tests ran; Release probes were linked and inspected only,
not executed.

B89 audit closed with no additive API. The installed iOS 26.5 header and objc2-core-location 0.3.2
confirm public `+[CLLocationManager locationServicesEnabled]`, a typed no-instance class query
available from iOS 4.0 that returns whether device-wide Location Services are enabled. Apple
distinguishes this device setting from app authorization. However, `LocationBackend::availability()`
already calls this exact method and maps `true` to `Available` and `false` to
`TemporarilyUnavailable`; `authorization()` separately exposes app authorization. A second Rust
boolean would therefore add no state or result, only a duplicate accessor. Reopen only if a future
contract needs a distinct device-switch state that cannot be inferred from the portable
availability result. The installed header warns that subsequent Core Location APIs may cause a
system warning when services are disabled; current Apple docs say location attempts fail and
report an error. This audit did not query a live device or verify `requestLocation()` warning
behavior, so B5 makes no runtime claim here.

Row `037-sensors-connectivity-location-geofencing-significant-change-where-supported` remains
partial beyond B5/F15's one-shot current-location surface. `PLAN.md` lists the portable location
crate and iOS backend but has no scoped geofencing or significant-change task. D4 explicitly
excludes both, and the current Rust API has no region value, transition event, or event lifecycle.
Adding only Rust types would leave the shared contract undefined for region geometry,
permission scope, platform limits, registration replacement/persistence, event delivery across
app suspension or termination, and per-region cancellation. These are contract decisions, not a
bounded backend plumbing gap; do not extend B5/F15 or claim these operations until a separate plan
defines their portable semantics and iOS policy. Existing compile, feature-tree, and link/import
checks establish only the one-shot API boundary and do not validate monitoring behavior. The link
gate now also guards the B5 Rust source/example call surface against continuous, region,
significant-change, visit, heading, beacon-ranging, Always-authorization, temporary-accuracy, and
background-location APIs. This source check does not constrain calls through the borrowed native
manager escape and does not close row 037's partial status. This guard was added after the recorded
link/import run; only its shell syntax and source predicates are checked in this follow-up, not the
full gate that builds Release probes. It also confines the sole `requestWhenInUseAuthorization()`
call to the explicit authorization-request path and confirms the one-shot current-request path stays
prompt-free.

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
- Complete an explicit foreground authorization future only after Core Location reports an authorization-status change; if a request yields no change, document that the future can remain pending and may be dropped
- Contain delegate callbacks with exact-once terminal completion and safe retained ownership; handle callback reentrancy and future drop per the D4 contract
- Respect Core Location's delegate run-loop requirements; document the callback thread and main-thread/run-loop requirements
- Do not request temporary full accuracy, Always authorization, background location, or unrelated location services
- State `Info.plist` permission text requirements and native status/error mapping
- Do not add `.swift` source, private API, background modes, PushKit, or runtime/global registry

## Validation and handoff

- Run `cargo check` and Clippy for `ios-location` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add deterministic tests for pure authorization, accuracy, and native-error mappings that do not need a live permission prompt
- Inspect device and simulator imports for CoreLocation and absence of unrelated capability frameworks and Swift runtime; audit both target Cargo feature trees to require exactly `CLLocation`, `CLLocationManager`, and `CLLocationManagerDelegate` from `objc2-core-location`
- Keep the link/import script's Rust source guard aligned with the one-shot contract and verify that it does not mistake `stopUpdatingLocation()` cancellation for continuous updates
- Keep `requestWhenInUseAuthorization()` confined to `start_authorization_request`; `start_current` must stay non-prompting
- Document minimum iOS version only from installed SDK metadata, permission text, authorization-status change and pending behavior, callback/run-loop behavior, location age/accuracy semantics, cancellation, native handles, and live-device test limits
- Report changed files, commit SHA, exact checks, deviations, and unresolved assumptions
