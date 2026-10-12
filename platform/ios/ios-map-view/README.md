# ios-map-view

`ios-map-view` creates a main-thread-bound public `MKMapView`, exposes its borrowed native view for UIKit embedding, gets/sets a finite `MapRegion`, and replaces caller-owned marker snapshots. The host owns layout and hierarchy; MapKit can adjust the requested region to the current view dimensions.

`MKMapView` is an imperative native UIKit view, not SwiftUI or a declarative map framework. `MapAnnotation` holds an application-defined stable id, validated coordinate, and optional borrowed title. `replace_annotations` rejects duplicate ids, reuses native `MKPointAnnotation` objects for ids that persist, updates coordinates and titles, and removes omitted ids. MapKit copies titles. The crate does not enable user location, request permission, add overlays, search, geocode, calculate directions, or provide offline maps. MapKit owns basemap content and any native network fetch behavior; this crate does not promise map content availability or prove runtime fetching.

The selected `objc2-map-kit 0.3.2` generated `MKMapView` binding is gated behind macOS/AppKit, so this iOS crate defines the public iOS `MKMapView` class plus `initWithFrame:`, `region`, `setRegion:animated:`, `addAnnotation:`, and `removeAnnotation:` declarations with `objc2`. It reuses the generated public `MKCoordinateRegion`, `MKCoordinateSpan`, and `MKPointAnnotation` bindings. `MKPointAnnotation` is public from iOS 4.0. No Swift source is used.

See the [MapKit capability guide](../../../docs/capabilities/mapkit.md), [iOS MapKit guide](../../../docs/ios/mapkit.md), and [registered validation plan](../../../PLAN_VALIDATION_IOS_MAPKIT.md).
