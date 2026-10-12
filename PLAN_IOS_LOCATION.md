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

B477 adds a distinct foreground continuous-location session without changing B5's one-shot
`Location::current()` contract. `Location::updates()` requires current foreground-capable
authorization and never prompts; its statically dispatched session owns a Core Location manager
and retained delegate until drop. The native callback array is copied oldest-to-newest into a fixed
16-event queue; overflow evicts oldest events and reports a saturating missed count on the next
event. Delegate errors remain nonterminal, dropping one wait preserves the session, and dropping
the session detaches the delegate and stops updates. Host tests cover authorization/no-prompt,
error-then-fix delivery, queue order, overflow accounting, wait cancellation, and teardown. Device
and Simulator compile/link checks establish static target evidence only; live authorization,
location delivery, and suspended-app behavior remain unverified

Row `037-sensors-connectivity-location-geofencing-significant-change-where-supported` remains
partial after B549 because geofencing and background delivery across suspension or termination
are not implemented. B549 adds the opt-in `IosLocationBackend::start_significant_change_updates()`
extension, which checks Core Location's device-support query and current foreground-capable
authorization without prompting, then uses the existing bounded event queue and retained delegate.
Its session drop detaches the delegate and calls `stopMonitoringSignificantLocationChanges()`.
The API defines no fixed threshold or cadence, makes no background-delivery or relaunch promise,
and does not persist a manager or configure standard background updates. Host regressions cover
unsupported devices and authorization states; device/Simulator compilation and link/import checks
establish static evidence only, not service delivery or power behavior. Region monitoring remains
unimplemented pending its own bounded geometry, permission, replacement, persistence, and
cancellation contract. The registered B477 profile's source guard now permits the significant
selector only in the dedicated operation path, keeps `requestLocation()` in B5's current-request
path, and keeps the authorization prompts in their explicit request paths. Negative fixtures
check selector scope, device-support preflight, authorization scope, matching stop behavior, and
existing prohibited APIs. The guard constrains repository Rust source only; calls through the B5
borrowed native manager escape remain outside the contract

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

## B496 — Always authorization and background update sessions

B496 extends `ios-location` without changing the portable foreground contract. The explicit,
lazy `request_always_authorization()` path requests Always only from Not Determined or When In Use;
already determined, denied, restricted, and unknown states do not trigger another prompt. It checks
the required usage-description values first. The iOS-only `start_background_updates()` requires
current Always authorization, checks that the host `Info.plist` has `UIBackgroundModes` containing
`location` before enabling `allowsBackgroundLocationUpdates`, and accepts typed activity, automatic
pausing, and optional iOS 11+ indicator hints. Authorization is refreshed before accepting every
location batch; a downgrade emits a nonterminal permission event and suppresses fixes until Always
returns. Session drop retains the existing bounded queue, delegate detachment, and stop behavior.

Host regressions cover plist-value parsing, authorization transitions, downgrade handling, and
`InvalidInput` mapping. Registered negative selector/configuration and availability guards are
extended. The registered `ios-location-updates` profile passed 40 checks with 0 FAIL, 0 BLOCKED,
and 2 SKIPPED at snapshot
`ff1baf94cc4e1fbbe4c11cea92330365ded7e847a14a0baf8c094e249b0670c3`; the skips are Simulator
execution and physical-device proof. Host tests, format, host/device/Simulator compilation and
Clippy, device Rustdoc, shell syntax, link/import, documentation, and source guards passed on
Xcode 26.6 / iOS SDK 26.5. Static checks do not prove live prompts, background callbacks,
suspension/resumption, or physical-device behavior

## B549 — Significant-change location session

B549 adds the iOS-only `IosLocationBackend::start_significant_change_updates()` entry point as an
opt-in alternative to standard updates. It requires existing foreground-capable authorization,
checks `significantLocationChangeMonitoringAvailable()`, never prompts, and reports `Unsupported`
when the device lacks service support. It uses the existing bounded update queue and serialized
main-run-loop delegate lifecycle; its session drop detaches the delegate and calls the matching
`stopMonitoringSignificantLocationChanges()` selector. Standard accuracy and distance-filter
settings do not define this service's events. The API makes no fixed-threshold, cadence, background
delivery, app-relaunch, or runtime claim. Host preflight tests and negative source regressions cover
support, permission, selector scope, and cancellation mapping; registered device and Simulator
gates provide compile/link evidence only

## B556 — Named iOS geofence monitoring

B556 adds `IosLocationBackend::open_geofence_monitor(name)` as an iOS 17+ extension over public
`CLMonitor` and `CLCircularGeographicCondition`; the portable `framework-location` contract remains
unchanged. The API requires existing Always authorization plus valid app usage-description strings
and never prompts. A process-wide lease rejects duplicate pending/open monitors, all native access
uses the monitor's configured serial queue, and drop releases the native monitor without removing
persisted conditions. Regions have validated WGS-84 centers, nonempty identifiers, and finite
positive radii with no invented maximum. The bounded 16-event queue reports evictions separately
from callback-copy issues. Host regressions and registered static device/Simulator checks do not
prove live state mapping, event delivery, background relaunch, permission UI, or Apple parity

## B559 — explicit temporary full-accuracy authorization

B559 extends only `platform/ios/ios-location` and its iOS documentation. The portable
`framework-location` contract and B87's synchronous, non-prompting
`accuracy_authorization()` behavior remain unchanged. Add the explicit lazy
`request_temporary_full_accuracy_authorization(purpose_key)` future over Core Location's public
iOS 14 completion-handler API. Reject a pre-iOS-14 runtime, empty/NUL purpose keys, missing or
empty exact-key `NSLocationTemporaryUsageDescriptionDictionary` values, and callers without
existing foreground-capable authorization. Never request foreground/Always permission
implicitly. Before calling Core Location, return Full or preserve Unknown snapshots without a
prompt; only Reduced accuracy reaches the request. Map a callback NSError through the existing
Core Location error mapping; after nil NSError, query a fresh snapshot, which can remain Reduced
when the user declines. Dropping the future detaches Rust state but cannot cancel the OS prompt;
late callbacks must remain safe.

The implementation uses the existing registered `ios-location-updates` profile and source-contract
negative fixtures for invalid purpose keys/configuration, preconditions, explicit selector scope,
and B87's non-prompting getter. The device and Simulator link probe must retain the exact public
selector at the profile's existing deployment floors. Host unit tests cover key validation.
Compilation/linking and source guards are static evidence only; live prompt, user choice, temporary
authorization expiry, and device behavior remain runtime-unverified

## B567 — typed location authorization-change events

Extend the portable update event enum with `AuthorizationChanged(LocationAuthorization)`. In the iOS backend, route both public Core Location authorization delegate callbacks and the location-batch status refresh through one deduplicating transition path. Emit one event for each distinct normalized status change, including restoration, without prompting. Preserve the existing nonterminal `PermissionDenied` event before the typed event when the transition falls below the session's authorization requirement; continue to suppress fixes until authorization again permits the session. Initial status is a baseline, not a change event. Keep the portable contract allocation-free for these scalar events, preserve the bounded queue and session drop behavior, and retain all deployment floors.

Acceptance requires deterministic host tests for exact transition values, duplicate suppression, downgrade classification, and queue order; source-contract negative fixtures must reject removal of either callback route, the typed event, downgrade error, or fix suppression. The existing `ios-location-updates` validator must cover the portable API, host tests, device/Simulator compile and strict Clippy, Rustdoc, source guards, link/import checks, docs, and deployment floors. Static validation does not prove live permission UI, Core Location callback delivery, background location, restoration timing, or physical-device behavior.

The host suites pass: `framework-location` 7/7, `ios-location` unit tests 33/33, and `ios-location` integration tests 2/2. The registered `ios-location-updates` profile passes its source and negative guards, format, host/device/Simulator builds and Clippy, Rustdoc, link/import, documentation, and policy gates. Simulator execution and physical-device proof are skipped; live authorization changes, restoration timing, and background delivery remain unverified.
