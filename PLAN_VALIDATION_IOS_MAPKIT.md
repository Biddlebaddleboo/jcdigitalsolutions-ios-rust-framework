# PLAN_VALIDATION_IOS_MAPKIT.md — G68 MapKit geometry

## Scope

G68 validates D77/B73's finite geographic coordinates, projected map points, non-negative meter
distances, and iOS MapKit conversion/distance calls. The `ios-map-view` profile also validates
B577's native map region API, B581's point-marker snapshot contract, B583's route overlay, B586's
caller-controlled user tracking, B601's bounded polygon overlay, and B602's native UIKit/MapKit
host surface. It does not cover user
location, permissions, network services, search, directions, or full MapKit parity.

## Gate

`sh platform/ios/ios-maps/check.sh` passed after root integration. The package gate includes format,
portable and host checks, strict Clippy, rustdoc, device/Simulator checks, feature isolation, and
link/import inspection.

Device and Simulator probes include MapKit geometry symbols and import MapKit, CoreLocation, ARKit,
Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. Probe minimums are iOS 12.0 on device and
iOS 14.0 on Simulator; the declared API floor remains iOS 4.0. Probes were not executed. Local
evidence used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the plan's Xcode 27.x
baseline.

## Limits

The gates do not establish native runtime conversion or distance parity, visual map behavior,
location access, request success, or performance.

## B577 registered native-view gate

`ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-map-view` passed with the locally selected validator `0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d` (binary SHA-256 `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`). Post-rebase snapshot `47ba54e43b115eb284eb5fc27d216c95f808ac22bb5cff444c32456a135437c8` covered 12,471,951 bytes and reported 28 passed, 0 failed, 0 blocked, and 2 skipped

The registered host test gate ran the invalid-span negative regression and valid-boundary tests. Device and Simulator compile, Clippy, Rustdoc, and Release link/import checks passed with the profile floors of iOS 12.0 and iOS 14.0. The link probes checked MapKit/UIKit/CoreLocation imports, Objective-C class/message references, forbidden location/search/annotation symbols, and deployment metadata

Simulator execution and physical-device proof remain skipped. No runtime map rendering, basemap request, tile fetch, or content availability is claimed

## B581 registered annotation gate

The `ios-map-view` profile now requires the portable marker value, duplicate-id negative regression,
snapshot replacement API, and native remove path. Its Release device and Simulator link probes
exercise the public `MKPointAnnotation` class and `MKMapView` add/remove selectors, while retaining
the existing imports and deployment-probe floors. Host tests establish input value preservation,
stable-id ordering, and duplicate rejection before native mutation. None of these checks prove
rendered pin style, title callout visibility, touch interaction, or runtime behavior; those remain
unverified without Simulator or device execution.

## B601 registered polygon overlay gate

The existing `ios-map-view` profile selects the crate's complete host test suite and
`check-link-imports.sh`. B601 extends the selected link probe with negative host-test source guards
and device/Simulator symbol-string requirements for public `MKPolygon`, `MKPolygonRenderer`,
`polygonWithCoordinates:count:`, `initWithPolygon:`, `setFillColor:`, and `setStrokeColor:` use. The
existing profile source directories already cover `polygon.rs`, `platform.rs`, the example, and the
probe script; no new profile or schema fields are required. Test and gate results are recorded in
`PLAN.md` after the implementation run. Static probes do not establish overlay render pixels or
runtime coexistence.

## B602 registered native UIKit/MapKit host surface gate

The existing `ios-map-view` profile requires guards for retained host-delegate lifecycle, the typed
`UIView` borrow, caller-selected overlay levels, iOS `UIEdgeInsets`, native annotation selection,
and delegate replacement cleanup. Its host test gate runs negative delegate-slot and active-overlay
policy tests. The Release device and Simulator link probe uses two distinct `MKPolyline` objects,
native renderer width/dash selectors, region/map-rect/camera selectors, annotation selection,
delegate region/selection callbacks, and UIKit add/frame/bounds/layout/hit-test/remove methods.

Both `aarch64-apple-ios` and `aarch64-apple-ios-sim` links retain the existing iOS 12.0 and 14.0
probe floors. The import scan keeps MapKit, UIKit, CoreLocation, Foundation, CoreFoundation,
`libSystem.B.dylib`, and `libobjc.A.dylib` as the complete dependency set. These probes are linked,
not run; delegate delivery, simultaneous display, native touches, rotation, and physical-device
behavior remain unverified.

The registered command `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-map-view` passed 28 checks with 0 FAIL, 0 BLOCKED, and 2 SKIPPED at snapshot `8969148fa6e63b259be93b1ee7d56fa382593026bb31305ae4d5b2061f62da4a`. The installed validator identity is `0.1.0+b6d422528c80671d828a4b0d40e9fc739652409a`, binary SHA-256 `4835271071ca3b7292f09a50d6d6c682f0c58f7d219045dd05e556553a451262`. The skips are Simulator execution and physical-device proof; no MapKit runtime behavior is claimed.
