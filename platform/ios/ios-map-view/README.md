# ios-map-view

`ios-map-view` creates a main-thread-bound public `MKMapView`, exposes its borrowed native view for UIKit embedding, and gets/sets a finite `MapRegion`. The host owns layout and hierarchy; MapKit can adjust the requested region to the current view dimensions.

`MKMapView` is an imperative native UIKit view, not SwiftUI or a declarative map framework. The crate does not enable user location, request permission, add annotations or overlays, search, geocode, calculate directions, or provide offline maps. MapKit owns basemap content and any native network fetch behavior; this crate does not promise map content availability or prove runtime fetching.

The selected `objc2-map-kit 0.3.2` generated `MKMapView` binding is gated behind macOS/AppKit, so this iOS crate defines only the public `MKMapView` class and `initWithFrame:`, `region`, and `setRegion:animated:` declarations with `objc2`. It reuses the generated public `MKCoordinateRegion` and `MKCoordinateSpan` value bindings. No Swift source is used.

See the [MapKit capability guide](../../../docs/capabilities/mapkit.md), [iOS MapKit guide](../../../docs/ios/mapkit.md), and [registered validation plan](../../../PLAN_VALIDATION_IOS_MAPKIT.md).
