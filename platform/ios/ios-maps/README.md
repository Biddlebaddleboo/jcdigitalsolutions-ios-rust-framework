# ios-maps

`ios-maps` implements an ARKit support snapshot and a MapKit geometry-only slice. The ARKit
`world_tracking_support()` query sends inherited `ARConfiguration.isSupported` to
`ARWorldTrackingConfiguration`; it does not create `ARSession`, read frames, access camera hardware,
or request camera permission. The MapKit functions convert a caller-owned `MapCoordinate` to a
finite `MapPoint`, convert a finite `MapPoint` back to a valid `MapCoordinate`, and return MapKit's
finite non-negative surface distance in `MapDistanceMeters`.

The ARKit support-query floor is iOS 11.0; the MapKit geometry API floor is iOS 4.0. The crate uses
`objc2-ar-kit` 0.3.2 with default features disabled and only `ARConfiguration` plus `objc2` enabled,
and `objc2-map-kit` 0.3.2 with default features disabled and only `MKGeometry` plus
`objc2-core-location` enabled. These MapKit functions do not request location access, access a
location service, create a map view, or invoke network-backed MapKit services. `ios-maps` also has
the direct `objc2-core-location` 0.3.2 `CLLocation` feature edge for its coordinate struct type.
See the [iOS ARKit guide](../../../docs/ios/arkit.md), [iOS MapKit guide](../../../docs/ios/mapkit.md),
and `sh platform/ios/ios-maps/check.sh`.
