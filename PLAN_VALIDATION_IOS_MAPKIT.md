# PLAN_VALIDATION_IOS_MAPKIT.md — G68 MapKit geometry

## Scope

G68 validates D77/B73's finite geographic coordinates, projected map points, non-negative meter
distances, and iOS MapKit conversion/distance calls. It does not cover map UI, user location,
permissions, network services, search, directions, or full MapKit parity.

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
