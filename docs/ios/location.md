# iOS location

`ios-location` implements the portable one-shot and foreground update-session `framework-location` contracts with public Core Location APIs. B477 adds `Location::updates()` as a separate API from the one-shot `Location::current()` request. B496 adds an iOS-only explicit Always-authorization request and background-enabled continuous session without changing the portable foreground contract. B549 adds an opt-in iOS significant-change session alongside standard updates. B556 adds named iOS geofence monitoring as a platform extension over public `CLMonitor`; no portable geofence contract is implied. B567 adds exact authorization-change snapshots to active portable update streams while preserving downgrade errors. Start background sessions in the foreground when requesting system continuation after the app backgrounds. iOS controls actual delivery; no delivery while suspended or after termination is guaranteed

## Create and use the backend

Create the backend on the main thread and keep its futures on that thread:

~~~rust
use framework_location::{AccuracyTarget, Location, LocationRequest};
use ios_location::IosLocationBackend;
use objc2::MainThreadMarker;

async fn read_location() -> Result<framework_location::LocationFix, framework_location::LocationError> {
    let marker = MainThreadMarker::new().expect("main thread");
    let backend = IosLocationBackend::new(marker);
    let mut location = Location::new(backend);
    let request = LocationRequest::new(AccuracyTarget::new(100.0).expect("finite accuracy"));
    location.current(request).await
}
~~~

`authorization()` queries the current Core Location status without a prompt. Only polling the explicit `request_authorization()` operation can call `requestWhenInUseAuthorization`; it never requests Always access. The iOS-only `request_always_authorization()` is also explicit and lazy: polling it can call `requestAlwaysAuthorization` only while authorization is Not Determined or When In Use. Already Always, denied, restricted, and unknown states return immediately without another prompt. The future otherwise completes on the first status change and may remain pending if Core Location reports none; callers may drop it. A first grant can be provisional and later change to When In Use, so an Always result is a status snapshot rather than a promise of permanent permission. Invoke either request from a meaningful user action

`B87` adds `IosLocationBackend::accuracy_authorization()` as a separate, synchronous, non-prompting iOS extension. It returns `None` before iOS 14.0, when `CLLocationManager.accuracyAuthorization` is unavailable, and otherwise returns `FullAccuracy`, `ReducedAccuracy`, or `Unknown(isize)` with an unrecognized signed `NSInteger` preserved exactly. The query creates a manager and reads the public readonly property on the backend's main thread; it does not call an authorization request or start a location service. Apple's Core Location docs specify the manager's creation-thread run loop for delegate callbacks but give no separate queue requirement for this property read. B5 confines the read to its existing main-thread backend, and this synchronous query installs no delegate. Interpret the snapshot together with `authorization()`: Apple's SDK notes that `FullAccuracy` can be returned while authorization is still not determined, and the value can change before a later location fix. This API does not alter portable D4 semantics or the B5 permission-request behavior.

`B559` adds `IosLocationBackend::request_temporary_full_accuracy_authorization(purpose_key)` as a separate, explicit iOS 14+ request. It starts only when its returned future is first polled, requires existing foreground-capable location authorization, and never requests When In Use or Always access. The key must be nonempty, contain no NUL, and match a nonempty string under `NSLocationTemporaryUsageDescriptionDictionary` in the app's main `Info.plist`; invalid or absent configuration returns `InvalidInput`. Existing Full accuracy returns immediately without a prompt. An unrecognized accuracy value is preserved and does not prompt. Reduced accuracy calls Apple's public completion-handler API. A Core Location error means the prompt was suppressed and uses the existing native location-error mapping; a nil error means the user responded, and the result is a fresh accuracy snapshot that may still be Reduced if the user declined. Core Location controls prompt display and temporary-access expiry; background calls may be suppressed. Dropping the future detaches Rust interest but cannot retract a prompt already handed to iOS, and late callbacks are ignored. B87's `accuracy_authorization()` remains synchronous and non-prompting.

## Availability and permission configuration

The operation-scoped `CLLocationManager` and Rust delegate are created lazily on the first future poll. The backend constructor requires `MainThreadMarker`; create, poll, and drop its backend and futures on that same main thread. Core Location delivers delegate callbacks on the run loop of the thread where the manager was created, so the application's main run loop must continue to run. A normal UIKit application owns that run loop. Foreground requests and `Location::updates()` do not enable a background mode or background-location setting

For the foreground permission prompt, the consuming app must provide a nonempty `NSLocationWhenInUseUsageDescription` string in its `Info.plist`. For `request_always_authorization()`, provide nonempty `NSLocationWhenInUseUsageDescription` and `NSLocationAlwaysAndWhenInUseUsageDescription`. When the app's deployment target is iOS 10 or earlier, also include nonempty `NSLocationAlwaysUsageDescription`; the system reads the legacy key on older OS versions and the modern key on newer ones. The backend reads `MinimumOSVersion` from the main bundle to decide whether the legacy key is required; if that value is absent or malformed, it conservatively requires the legacy key. It returns `InvalidInput` when a required usage key is absent, not a string, or empty. The B559 temporary-accuracy request separately requires a matching nonempty purpose entry under `NSLocationTemporaryUsageDescriptionDictionary`; it does not substitute for app authorization. No permission request runs unless the caller explicitly polls the matching request future

The backend reports `Availability::TemporarilyUnavailable` when `+[CLLocationManager locationServicesEnabled]` is false; authorization remains a separate query. The class query itself is a non-prompting snapshot, but the installed SDK header warns that proceeding to other Core Location APIs while the device-wide switch is off may show a system warning; current Apple docs state location attempts fail and report an error. B5 has no live-device evidence for `requestLocation()` behavior in this state, so `availability()` is not a guarantee that a later operation stays prompt-free. The authorization map is: not determined, restricted, denied, authorized Always as `Background`, authorized When In Use as `Foreground`, and unknown raw statuses as `Unknown`.

## One-shot behavior, result age, and accuracy

Each current request creates a fresh manager and calls `requestLocation()` once. The portable accuracy target is assigned to `desiredAccuracy` in meters; zero uses Apple's `kCLLocationAccuracyBest`, while positive targets are passed through unchanged. Core Location treats desired accuracy as a preference, not a guarantee. A less accurate best-available fix may be returned after Core Location's timeout, and an unavailable fix may end in `kCLErrorLocationUnknown`.

For a `didUpdateLocations` callback, the backend copies the final location in the native array without refreshing or applying a maximum-age filter. It preserves the WGS-84 latitude/longitude, non-negative horizontal-accuracy estimate in meters, and native measurement timestamp converted to Unix milliseconds. The portable contract therefore makes no freshness, time-to-fix, indoor-accuracy, or target-meeting guarantee. Invalid coordinates/accuracy and timestamps outside the portable unsigned Unix-millisecond range are rejected rather than clamped.

Core Location errors map `kCLErrorLocationUnknown` and `kCLErrorNetwork` to `Unavailable`, `kCLErrorDenied` to `PermissionDenied`, and other native codes to `Platform`. A representable nonzero Core Location code is retained in `LocationError::Backend`; code zero cannot be attached because `framework-core::PlatformErrorCode` reserves zero for “no code.”

## Foreground update sessions

Use `Location::updates(request)` after the app already has `Foreground` or `Background` authorization. This synchronous call checks current authorization and returns `PermissionDenied` without prompting when authorization does not permit a foreground request. The caller can use the existing explicit authorization flow before creating a session. The session never requests Always access, enables a background mode, or changes B5's one-shot request behavior

~~~rust
use framework_location::{LocationUpdateKind, LocationUpdateSession};

let mut updates = location.updates(request)?;
loop {
    let event = updates.next_update().await;
    if event.missed_before() != 0 {
        record_skipped_updates(event.missed_before());
    }
    match event.kind() {
        LocationUpdateKind::Fix(fix) => use_location(fix),
        LocationUpdateKind::Error(error) => report_location_error(error),
        LocationUpdateKind::AuthorizationChanged(authorization) => {
            update_permission_state(authorization);
        }
        _ => {}
    }
}
~~~

Each `didUpdateLocations` callback contributes its array entries in the callback's oldest-to-newest order. `LocationUpdateKind::Error` reports a transient backend/delegate error and does not end the session; a later fix can follow it. B567 also reports each distinct normalized authorization state received by either public Core Location authorization delegate callback. Duplicate callbacks for the same state are suppressed. A transition below the session's required authorization queues the existing nonterminal `PermissionDenied` error before `AuthorizationChanged`; a restoration event reports the new state even before the next location fix. These events do not prompt or guarantee service delivery. The queue holds at most 16 events. On overflow it evicts the oldest queued event and adds to a saturating `u64` count reported as `missed_before` on the next returned event, so overflow is observable

The active session exclusively borrows the backend and owns a `CLLocationManager` plus a strongly retained Rust delegate. Create, poll, and drop it on the backend's main thread while its run loop is active. Dropping one `next_update()` future cancels only that wait and preserves queued events; dropping the session detaches the delegate queue and calls `stopUpdatingLocation()`. If authorization is revoked during a session, the delegate emits a nonterminal permission error; drop the session before using the backend's authorization flow again

## Significant-change update sessions

Call `IosLocationBackend::start_significant_change_updates()` to explicitly opt into Apple's significant-change service. It uses the same bounded event queue, delegate run loop, authorization-change handling, and caller-owned session lifetime as standard updates. Core Location's `significantLocationChangeMonitoringAvailable()` must report device support; otherwise the method returns a `LocationError` whose kind is `Unsupported`. It requires an already-authorized `Foreground` or `Background` status and never requests permission. The app must provide the applicable usage-description key for the authorization it requested

Apple describes this as a low-power service and says the first callback may contain a cached location; the system controls whether and when later changes are significant. This API defines no distance threshold or update cadence, ignores standard desired-accuracy and distance-filter settings, and does not promise background delivery, relaunch after termination, or any runtime result. It does not enable `allowsBackgroundLocationUpdates` or alter the separate standard background-update path. Dropping the session detaches its delegate and calls `stopMonitoringSignificantLocationChanges()`; the session itself is not persisted across process termination

## Always authorization and background updates

Use `IosLocationBackend::request_always_authorization()` from an explicit user action when the app needs Always access. For a background session, first obtain `LocationAuthorization::Background`, then call `start_background_updates(request, IosBackgroundLocationOptions::new())`. This iOS extension does not change `Location::updates()`; that portable API remains foreground-capable, never prompts, and works with either When In Use or Always authorization

Before enabling background delivery, the backend verifies that the main app `Info.plist` has `UIBackgroundModes` as an array containing the exact string `location`. Missing, malformed, or unrelated values return `InvalidInput` before the `allowsBackgroundLocationUpdates` setter is called. This preflight prevents the documented fatal termination caused by enabling the setter without that background mode. The host app must also enable Background Modes > Location updates in its target capabilities. Always authorization alone does not configure the background mode, and the background mode does not grant permission

`IosBackgroundLocationOptions` provides public activity, automatic-pausing, and optional indicator hints. Defaults are `Other`, automatic pausing enabled, and no explicit background indicator. `setShowsBackgroundLocationIndicator` is used only when requested and is runtime-guarded to iOS 11+; requesting it on an older system returns `Unsupported`. Background sessions use `kCLDistanceFilterNone`, the requested accuracy preference, and standard `startUpdatingLocation()`. An authorization transition below Always emits the existing nonterminal `PermissionDenied` event followed by `AuthorizationChanged` with the exact new normalized state, then suppresses location fixes until Always authorization returns. Restoration is reported as a state event; the session does not prompt again. Dropping the session detaches its delegate and stops updates as for foreground sessions

Required app configuration:

- `NSLocationWhenInUseUsageDescription`: nonempty localized purpose text
- `NSLocationAlwaysAndWhenInUseUsageDescription`: nonempty localized purpose text for Always access on iOS 11 and later
- `NSLocationAlwaysUsageDescription`: nonempty localized purpose text when the app's deployment target is iOS 10 or earlier; required conservatively if `MinimumOSVersion` cannot be read
- `UIBackgroundModes`: array with `location`; enable the target's Background Modes > Location updates capability

Apple documents the required [`NSLocationWhenInUseUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationwheninuseusagedescription), [`NSLocationAlwaysAndWhenInUseUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationalwaysandwheninuseusagedescription), and pre-iOS-11 [`NSLocationAlwaysUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationalwaysusagedescription) keys, and the [`requestAlwaysAuthorization()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestalwaysauthorization%28%29) request. [`UIBackgroundModes`](https://developer.apple.com/documentation/bundleresources/information-property-list/uibackgroundmodes) must include `location` before [`allowsBackgroundLocationUpdates`](https://developer.apple.com/documentation/corelocation/cllocationmanager/allowsbackgroundlocationupdates) is enabled. The public availability floors verified from the installed SDK are iOS 8 for Always authorization, iOS 9 for background updates, and iOS 11 for the background indicator. These checks and target compilation do not verify live prompt behavior, background callback delivery, suspension/resumption, or termination behavior

## Cancellation, ownership, and native escape

The mutable backend borrow prevents two simultaneous operations through this backend. Each one-shot operation owns its `CLLocationManager` and strongly retains its Rust delegate because Core Location's delegate property is weak. The delegate contains the operation's exactly-once completion state; native values are copied into framework-owned scalar values before the callback returns. Update sessions own a separate manager and queue-backed delegate. With an unwinding Rust panic strategy, callback panics are caught and mapped to an internal error event or result; with `panic=abort`, a panic aborts the process.

Dropping a pending current-location future detaches its result and calls `stopUpdatingLocation()` on its operation-scoped manager. Any racing or late delegate completion is discarded. Dropping an update session closes its queue, clears the manager's delegate, then calls `stopUpdatingLocation()`; a callback racing with teardown cannot publish another event. Dropping an authorization future detaches the result but cannot promise to dismiss a prompt already shown; releasing the future releases its delegate, and Core Location's weak delegate reference does not point into freed Rust state. `native_location_manager()` exposes a borrowed manager only after the one-shot future starts; direct calls can alter that active operation. It remains valid only while the future is alive.

## Persistent iOS geofence monitors

`IosLocationBackend::open_geofence_monitor(name)` is an iOS-only extension and requires iOS 17.0 or newer at runtime. It returns `Unsupported` on older systems. The call does not prompt: current authorization must already map to `LocationAuthorization::Background` (Always), and the app bundle must contain the required nonempty When In Use and Always usage-description strings, including the legacy Always key when the deployment-target rule requires it. It returns `PermissionDenied` or `InvalidConfiguration` before creating a native monitor when those preconditions fail

~~~rust
use ios_location::{IosGeofence, IosLocationBackend};

async fn watch_home(backend: &mut IosLocationBackend) -> Result<(), ios_location::IosGeofenceError> {
    let open = backend.open_geofence_monitor("HomeMonitor")?;
    let mut monitor = open.await?;
    let home = IosGeofence::new("home", 43.6532, -79.3832, 150.0)?;
    monitor.add_or_replace(&home)?;
    let persisted = monitor.list()?;
    for region in &persisted {
        record_region(region.identifier());
    }
    while let Some(event) = monitor.next_event().await {
        if event.missed_before() != 0 {
            record_evicted_geofence_events(event.missed_before());
        }
        handle_geofence_event(event.identifier(), event.state(), event.issues());
    }
    Ok(())
}
~~~

`IosGeofence::new` requires a nonempty identifier, a finite WGS-84 latitude/longitude, and a finite positive radius in meters. There is no invented maximum radius; Core Location can report unsupported conditions or its condition-count limit through typed event issues. `add_or_replace`, `remove`, and `list` are synchronous operations serialized on the monitor's configured serial queue. `list` returns circular conditions; a persisted condition of another native type is reported as `NativeFailure` rather than silently converted

Monitor names must be nonempty ASCII letters and digits and must not begin with `_`. Core Location permits only one live monitor per name; this crate narrows the process contract further and rejects any second pending/open monitor with `MonitorAlreadyOpen`, even if its name differs. Reserve the chosen name exclusively for this backend, since another direct `CLMonitor` client is outside the crate's lease. Conditions remain persisted until explicitly removed. Dropping the Rust session releases the native monitor on its configured queue but does not remove its regions. Reopen the same name on each app launch to receive events; persistence does not guarantee background relaunch, event delivery while suspended, or any delivery cadence

Events expose an optional identifier, `Unknown`/`Satisfied`/`Unsatisfied`/`Unmonitored` state, an optional representable Unix-millisecond timestamp, and typed issue flags. The issue flags for authorization, support, persistence, and condition limits are available starting in iOS 18; on iOS 17 the issue slice may be empty. A failed native callback copy is emitted as a monitor-level event with no identifier and `IosGeofenceIssue::CallbackFailure`. The queue retains at most 16 events. `missed_before()` counts only earlier queued events evicted by overflow; a callback-copy failure has its own issue and does not itself increase that count. Dropping a pending `next_event()` future cancels only that wait; dropping the monitor detaches event delivery and leaves persisted conditions intact

## SDK floor and validation limits

The installed Xcode 26.6 / iPhoneOS 26.5 SDK headers mark `requestLocation()` available starting in iOS 9.0; this remains the backend's base API floor. The significant-change availability query and start/stop selectors are available from iOS 4.0, below that floor. `requestWhenInUseAuthorization()` and `requestAlwaysAuthorization()` are available from iOS 8.0, `allowsBackgroundLocationUpdates` from iOS 9.0, and the background indicator from iOS 11.0. The backend uses the deprecated class-level `authorizationStatus` query to retain compatibility with iOS versions before the iOS 14 instance property. The old and new authorization delegate callbacks are both implemented. The separate `accuracy_authorization()` query is runtime-gated to iOS 14.0 and newer. Registered device and Simulator gates compile and link with their configured minimum deployment targets; they do not prove callback delivery on a running OS or hardware

Pure tests cover authorization, accuracy, fix-value, and native-error conversion. `sh platform/ios/ios-location/check-link-imports.sh` checks both targets' `objc2-core-location` feature trees and links probes for direct-import inspection; target checks and probe import inspection establish compile/linkage scope only. They do not exercise live permission UI, physical GPS/Wi-Fi positioning, cached-fix age, timeout behavior, or cancellation races on a running device.

Apple API references: [`CLLocationManager`](https://developer.apple.com/documentation/corelocation/cllocationmanager), [`locationServicesEnabled()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/locationservicesenabled%28%29?language=objc), [`accuracyAuthorization`](https://developer.apple.com/documentation/corelocation/cllocationmanager/accuracyauthorization), [`CLAccuracyAuthorization`](https://developer.apple.com/documentation/corelocation/claccuracyauthorization), [`requestLocation()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestlocation%28%29?language=objc), [`startUpdatingLocation()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/startupdatinglocation%28%29), [`significantLocationChangeMonitoringAvailable()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/significantlocationchangemonitoringavailable%28%29), [`startMonitoringSignificantLocationChanges()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/startmonitoringsignificantlocationchanges%28%29), [`stopMonitoringSignificantLocationChanges()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/stopmonitoringsignificantlocationchanges%28%29), and [`locationManager(_:didUpdateLocations:)`](https://developer.apple.com/documentation/corelocation/cllocationmanagerdelegate/locationmanager%28_%3Adidupdatelocations%3A%29)

## Binding dependency

The backend uses `objc2-core-location` 0.3.2 with only the `CLLocation`, `CLLocationManager`, `CLLocationManagerDelegate`, `CLMonitor`, `CLMonitorConfiguration`, `CLMonitoringEvent`, `CLMonitoringRecord`, `CLCondition`, `CLCircularGeographicCondition`, `block2`, and `dispatch2` features enabled. The device and Simulator link/import gate checks each target's Cargo feature tree for exactly these Core Location binding features. Its executable source guard confines `requestWhenInUseAuthorization()`, `requestAlwaysAuthorization()`, `requestTemporaryFullAccuracyAuthorizationWithPurposeKey:completion:`, `requestLocation()`, `startUpdatingLocation()`, and `startMonitoringSignificantLocationChanges()` to their explicit operation paths; negative fixtures verify selector scope, prompt eligibility, Always/background configuration ordering, significant-change device support and teardown, geofence availability/preflight, `CLMonitor` queue use, ownership transfer, persistent conditions, overflow semantics, caller-option mapping, downgrade event/suppression behavior, indicator availability, and prohibited APIs. It rejects deprecated region, visit, heading, and beacon-ranging APIs. These source and link checks do not prove runtime behavior. Calls made by a host through the borrowed one-shot native-manager escape remain outside this contract and can alter the active operation. The portable crate exposes none of the binding types. Conversion, monitor, and delegate code are isolated under `ios-location`, so a future binding replacement remains local to this backend
