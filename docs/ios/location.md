# iOS location

`ios-location` implements the portable one-shot `framework-location` contract with public Core Location APIs. It does not run continuous updates, track geofence regions or significant-change events, or enable background location. The portable contract defines no region value, transition event, or event lifecycle; authorization status and the one-shot native-manager escape do not add these operations.

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

`authorization()` queries the current Core Location status without a prompt. Only polling an explicit `request_authorization()` operation can call `requestWhenInUseAuthorization`; it never requests Always access. The request future completes only after Core Location reports a raw authorization status different from the status sampled before the request. An already-determined status is not returned as an immediate request result; if the call reports no status change, the future can remain pending and callers may drop it. Use `authorization()` to read the current status without waiting for a change. The application must call the explicit authorization operation in response to a meaningful user action if it wants Core Location's prompt.

## Availability and permission configuration

The operation-scoped `CLLocationManager` and Rust delegate are created lazily on the first future poll. The backend constructor requires `MainThreadMarker`; create, poll, and drop its backend and futures on that same main thread. Core Location delivers delegate callbacks on the run loop of the thread where the manager was created, so the application's main run loop must continue to run. A normal UIKit application owns that run loop. No background mode or background-location setting is enabled.

For a permission prompt, the consuming app must provide a meaningful `NSLocationWhenInUseUsageDescription` string in its `Info.plist` and call the request while the app is in use. The authorization future completes only after Core Location reports an authorization-status change. If the request yields no status change, the future can remain pending; the app may drop it to abandon the result. Without the key or an active app, Core Location may provide no status-change callback, so the future can remain pending. This backend does not require or request `NSLocationAlwaysAndWhenInUseUsageDescription`, Always authorization, temporary full accuracy, or background location. Make one-shot requests while the app is foregrounded; no background mode is enabled.

The backend reports `Availability::TemporarilyUnavailable` when `+[CLLocationManager locationServicesEnabled]` is false; authorization remains a separate query. The authorization map is: not determined, restricted, denied, authorized Always as `Background`, authorized When In Use as `Foreground`, and unknown raw statuses as `Unknown`.

## One-shot behavior, result age, and accuracy

Each current request creates a fresh manager and calls `requestLocation()` once. The portable accuracy target is assigned to `desiredAccuracy` in meters; zero uses Apple's `kCLLocationAccuracyBest`, while positive targets are passed through unchanged. Core Location treats desired accuracy as a preference, not a guarantee. A less accurate best-available fix may be returned after Core Location's timeout, and an unavailable fix may end in `kCLErrorLocationUnknown`.

For a `didUpdateLocations` callback, the backend copies the final location in the native array without refreshing or applying a maximum-age filter. It preserves the WGS-84 latitude/longitude, non-negative horizontal-accuracy estimate in meters, and native measurement timestamp converted to Unix milliseconds. The portable contract therefore makes no freshness, time-to-fix, indoor-accuracy, or target-meeting guarantee. Invalid coordinates/accuracy and timestamps outside the portable unsigned Unix-millisecond range are rejected rather than clamped.

Core Location errors map `kCLErrorLocationUnknown` and `kCLErrorNetwork` to `Unavailable`, `kCLErrorDenied` to `PermissionDenied`, and other native codes to `Platform`. A representable nonzero Core Location code is retained in `LocationError::Backend`; code zero cannot be attached because `framework-core::PlatformErrorCode` reserves zero for “no code.”

## Cancellation, ownership, and native escape

The mutable backend borrow prevents two simultaneous requests through this backend. Each operation owns its `CLLocationManager` and strongly retains its Rust delegate because Core Location's delegate property is weak. The delegate contains only the operation's exactly-once completion state; native values are copied into framework-owned scalar values before the callback returns. With an unwinding Rust panic strategy, callback panics are caught and mapped to an internal error; with `panic=abort`, a panic aborts the process.

Dropping a pending current-location future detaches its result and calls `stopUpdatingLocation()` on its operation-scoped manager. Any racing or late delegate completion is discarded. Dropping an authorization future detaches the result but cannot promise to dismiss a prompt already shown; releasing the future releases its delegate, and Core Location's weak delegate reference does not point into freed Rust state. `native_location_manager()` exposes a borrowed manager only after the future starts; direct calls can alter the active operation. It remains valid only while the future is alive.

## SDK floor and validation limits

The installed Xcode 26.6 / iPhoneOS 26.5 SDK headers mark `requestLocation()` available starting in iOS 9.0; this is the backend's minimum iOS API floor. `requestWhenInUseAuthorization()` is available from iOS 8.0. The backend uses the deprecated class-level `authorizationStatus` query to retain compatibility with iOS versions before the iOS 14 instance property. The old and new authorization delegate callbacks are both implemented.

Pure tests cover authorization, accuracy, fix-value, and native-error conversion. `sh platform/ios/ios-location/check-link-imports.sh` checks both targets' `objc2-core-location` feature trees and links probes for direct-import inspection; target checks and probe import inspection establish compile/linkage scope only. They do not exercise live permission UI, physical GPS/Wi-Fi positioning, cached-fix age, timeout behavior, or cancellation races on a running device.

Apple API references: [`CLLocationManager`](https://developer.apple.com/documentation/corelocation/cllocationmanager), [`requestLocation()`](https://developer.apple.com/documentation/corelocation/cllocationmanager/requestlocation%28%29?language=objc).

## Binding dependency

The backend uses `objc2-core-location` 0.3.2 with only the `CLLocation`, `CLLocationManager`, and `CLLocationManagerDelegate` features enabled. The device and Simulator link/import gate checks each target's Cargo feature tree for exactly these Core Location binding features; a separate source guard rejects Rust calls to continuous, region, significant-change, visit, heading, beacon-ranging, Always-authorization, temporary-accuracy, and background-location APIs. It also confines the sole `requestWhenInUseAuthorization()` call to the explicit authorization-request path and keeps the one-shot current-request path prompt-free. These checks describe backend code only. Calls made by a host through the borrowed native-manager escape remain outside this contract and can alter the active operation. The portable crate exposes none of these dependency types. Conversion and delegate code are isolated under `ios-location`, so a future binding replacement remains local to this backend.
