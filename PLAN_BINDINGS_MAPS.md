# PLAN_BINDINGS_MAPS.md — Workstream F13: MapKit Geometry C ABI

## Objective

Expose the completed D77 MapKit geometry API through an opt-in C feature. Keep the C API limited
to finite coordinate/map-point conversion and MapKit surface distance. Use fixed-width C scalar
types and never expose Rust or Objective-C layout.

## Stable source contract

- `framework-maps::MapCoordinate` accepts finite latitude `-90..=90` and longitude `-180..=180`
  degrees.
- `framework-maps::MapPoint` accepts finite projected `x`/`y` values and does not assert world
  rectangle membership. Map points are not meters or screen points.
- `framework-maps::MapDistanceMeters` accepts finite, non-negative meters.
- `ios-maps::map_point_for_coordinate`, `coordinate_for_map_point`, and
  `meters_between_map_points` call `MKMapPointForCoordinate`, `MKCoordinateForMapPoint`, and
  `MKMetersBetweenMapPoints`; they return `None` below the checked iOS 4.0 floor or when a native
  output fails finite/range validation.
- Non-iOS `ios-maps` functions return `None`. F13 validates the scalar input first, then returns
  `FRAMEWORK_STATUS_UNSUPPORTED` for valid host input.
- MapKit distance is surface distance in meters, not Euclidean distance over projected points.
- The `ios-maps` crate also contains the separate ARKit support query. F13 exports none of that
  query; its C exports and retained link imports stay limited to MapKit geometry. The feature tree
  still compiles the combined `ios-maps` package, while the C consumer link retains only MapKit and
  `libSystem.B.dylib` (plus `libc++.1.dylib` for C++); it has no ARKit, CoreLocation, Foundation,
  or Objective-C runtime import.

## C contract

- Cargo feature `ios-maps` is off by default; it enables `framework-maps` and the optional iOS
  target dependency `ios-maps` only.
- Header: `framework_ios_maps.h`.
- Exports:
  - `framework_ios_maps_map_point_for_coordinate`
  - `framework_ios_maps_coordinate_for_map_point`
  - `framework_ios_maps_meters_between_map_points`
- Inputs are scalar `double` values. Geographic coordinate inputs use the finite degree ranges
  above. Map-point inputs must be finite.
- Required output pointers are initialized to zero before validation or platform work. Paired
  outputs must be distinct and non-overlapping. Invalid inputs or outputs return
  `FRAMEWORK_STATUS_INVALID_ARGUMENT`.
- Success returns `FRAMEWORK_STATUS_OK`; a missing runtime API or rejected native result returns
  `FRAMEWORK_STATUS_UNAVAILABLE`; valid non-iOS input returns `FRAMEWORK_STATUS_UNSUPPORTED`.
  Outputs remain zero unless status is OK. Rust panics return `FRAMEWORK_STATUS_PANIC` and do not
  unwind across C.
- The wrapped MapKit geometry API floor is iOS 4.0. Link probes use device iOS 12.0 and Simulator
  iOS 14.0 and do not raise that API floor.
- There are no objects, handles, callbacks, borrowed buffers, location-service calls, permissions,
  map UI, or retained state. The wrapper adds no main-thread rule.

## Scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs` and `bindings/c/src/ios_maps.rs`
- `bindings/c/include/framework_ios_maps.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-maps.sh`
- `docs/bindings/ios-maps.md`
- `PLAN_BINDINGS_MAPS.md`

Root owns `Cargo.lock`, CI, root `PLAN_BINDINGS.md`, aggregate plans, shared docs indexes, workspace
membership, and capability counts. F13 does not edit those files or D77 backend files.

## Validation gate

`sh bindings/c/check-ios-maps.sh` must check manifest JSON and shell syntax, format the C ABI crate,
compare header symbols to the ABI manifest, verify default and non-iOS feature isolation and the
limited `objc2-map-kit` geometry feature closure, run locked host/device/Simulator checks and strict
Clippy, build Release archives, compile/link C11/C++17 consumers without execution, inspect exact
F13 exports/imports and required MapKit geometry symbols, reject out-of-scope MapKit service/UI
symbols, and verify deployment metadata. It must not add or run tests or execute linked consumers.

## Status and validation record

F13 source, header, ABI manifest, guide, and gate are implemented. Root refreshes `Cargo.lock` with:

```sh
cargo +1.94.1 check --offline -p framework-c-api --no-default-features \
  --features ios-maps --target aarch64-apple-ios
```

On Rust 1.94.1 (`e408947bf`, 2026-03-25), Xcode 26.6 (Build 17F113), and iOS SDKs 26.5,
`sh bindings/c/check-ios-maps.sh` passed. It validated manifest JSON, shell syntax, Rust formatting,
header/manifest symbol parity, default and host feature isolation, the generated binding feature
closure, locked host/device/Simulator checks and strict Clippy, Release archives, and C11/C++17
host/device/Simulator compile/link consumers. The sole F13 exports were
`framework_ios_maps_coordinate_for_map_point`, `framework_ios_maps_map_point_for_coordinate`, and
`framework_ios_maps_meters_between_map_points`; the linked consumers referenced
`MKMapPointForCoordinate`, `MKCoordinateForMapPoint`, and `MKMetersBetweenMapPoints`.

The exact 64-bit device imports were `MapKit` and `libSystem.B.dylib` for C, with
`libc++.1.dylib` added for C++; Simulator imports matched. No ARKit, CoreLocation, Foundation, or
Objective-C runtime import was present, even though the combined `ios-maps` feature tree compiles
its separately scoped ARKit support binding. Device link consumers report `minos 12.0` and
Simulator consumers report `minos 14.0`; these validation floors do not change the MapKit API floor
of iOS 4.0. Consumers were linked and inspected, not run. No tests or MapKit service behavior were
run or claimed.

Xcode 27 run `38049963372` on head `af6607bd7195ccd8b5a707f02c8dd2c6bbc1feba` passed the
preferences C ABI step 241, MediaPlayer step 242, SpriteKit step 243, and CallKit step 244, then
failed MapKit C ABI step 245 in Xcode-27 job `114206929758`. The device C11 probe compiled and
linked with minos 12.0; the following C++17 compile of `framework-c-ios-maps-cpp.cpp` entered the
Xcode 27 SDK libc++ `stddef.h` shim and promoted `availability.h:204` warning `The selected
platform is no longer supported by libc++` to an error under `-Werror`. Since the fixtures use only
C ABI declarations, `check-ios-maps.sh` now passes `-nostdinc++` to its C++ host/device/Simulator
compile commands while retaining C++17, the `libc++.1.dylib` runtime import assertion, all MapKit
framework/symbol/API checks, device 12.0 and Simulator 14.0 minos checks, and the iOS 4.0 API-floor
record. Xcode 27 MapKit requalification is pending; no consumer or probe execution/runtime behavior
is claimed

macOS 15 run `38051886481` on head `59555590b4fec231ea2f3686faff43df9e7220fb`, job
`114212506152`, passed C ABI steps 241–244 and failed MapKit step 245. The hosted log shows the
host Release archive build completing before `check-ios-maps.sh` exits with code 1 and no diagnostic;
the script suppresses stderr for its Rust-archive symbol inspections, so the exact failing command
is not confirmed by the log. This failure is consistent with the recurring Xcode 16.4 Apple `nm`
incompatibility on Rust LLVM 21 archives, but that attribution remains inferred. The gate now uses
Rust's sysroot-matched `llvm-nm` for host, device, and Simulator Rust archive inspections, while
retaining Apple `nm` for linked Mach-O probes and preserving all export/import, framework, and
deployment assertions. macOS 15 requalification is pending. These are compile/link and metadata
checks only; no tests, consumer execution, or runtime behavior are claimed

## Root integration steps

After the root lock refresh, add `sh bindings/c/check-ios-maps.sh` beside the other C ABI gates in
the macOS CI workflow. Root adds the F13 plan and guide links to aggregate plans and documentation
indexes separately.
