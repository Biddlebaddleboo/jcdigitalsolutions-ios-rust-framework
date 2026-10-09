# PLAN_VALIDATION_C_ABI_MAPS.md — G70 MapKit geometry C ABI

## Scope

G70 validates F13's opt-in `ios-maps` C ABI over D77/B73. The interface converts finite map
coordinates to map points, map points back to coordinates, and computes MapKit distance. It exposes
no Objective-C object or UI type.

## Gate

`sh bindings/c/check-ios-maps.sh` passed after the root lock refresh. The gate checks manifest and
header consistency, default/host feature isolation, generated-binding feature closure, locked host,
device, and Simulator builds with strict Clippy, Release archives, C11/C++17 compile/link, exported
symbols, imports, and deployment metadata.

The C probe imports MapKit and `libSystem.B.dylib`; the C++ probe additionally imports
`libc++.1.dylib`. The three native references are `MKMapPointForCoordinate`,
`MKCoordinateForMapPoint`, and `MKMetersBetweenMapPoints`. Probe minimums are iOS 12.0 on device and
iOS 14.0 on Simulator; D77's API floor remains iOS 4.0. No probe or linked consumer was executed.
Local evidence used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the plan's Xcode
27.x baseline.

## Limits

Compile, Clippy, link, and import evidence does not establish coordinate conversion or distance
parity, native runtime behavior, visual map behavior, or performance.
