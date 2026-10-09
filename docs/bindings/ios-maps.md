# Optional iOS MapKit geometry C API

The opt-in `framework-c-api` Cargo feature `ios-maps` enables
`bindings/c/include/framework_ios_maps.h`. It exposes only D77's three synchronous MapKit geometry
calls: geographic coordinate to map point, map point to geographic coordinate, and distance between
map points.

## Values and outputs

Latitude is finite and within `-90..=90` degrees. Longitude is finite and within `-180..=180`
degrees. Map-point `x` and `y` must be finite; they use MapKit projection units, not meters or
screen points, and need not lie inside a checked world rectangle. Distance is MapKit's surface
distance in meters, not Euclidean distance over projected `x` and `y`.

Each required output is zeroed before validation or platform work. Null outputs, aliased paired
outputs, non-finite inputs, and out-of-range geographic coordinates return
`FRAMEWORK_STATUS_INVALID_ARGUMENT`. A successful MapKit result returns `FRAMEWORK_STATUS_OK` and
writes the output. On iOS, `FRAMEWORK_STATUS_UNAVAILABLE` means the runtime API floor is not met or
the native result fails finite/range validation; outputs stay zero. On non-iOS, valid inputs return
`FRAMEWORK_STATUS_UNSUPPORTED` with outputs zero. A caught Rust panic returns
`FRAMEWORK_STATUS_PANIC` with outputs zero.

The three public C functions are available from iOS 4.0. F13 adds no main-thread rule, native
object, handle, callback, location-service access, permission request, map UI, map-data/network
service, search, geocoding, or route behavior. It makes no MapKit parity or performance claim.

The build-only 64-bit C link probe imports `MapKit` and `libSystem.B.dylib`; C++ also imports
`libc++.1.dylib`. The link has no ARKit, CoreLocation, Foundation, or Objective-C runtime import,
even though the shared `ios-maps` package includes a separate ARKit support query. This C feature
does not expose or call that query.

The F13 gate compiles and links C11/C++17 consumers for host, device, and Simulator, then inspects
symbols, imports, and deployment metadata. It never executes a consumer binary. See
[`PLAN_BINDINGS_MAPS.md`](../../PLAN_BINDINGS_MAPS.md),
[`docs/ios/mapkit.md`](../ios/mapkit.md), and
[`D77 MapKit geometry`](../../PLAN_CAPABILITIES_MAPKIT.md).
