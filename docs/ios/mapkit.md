# iOS MapKit

`ios-maps` contains MapKit coordinate projection and distance calls. The independent
`ios-map-view` crate creates a native, main-thread-bound `MKMapView` that a host UIKit controller
can embed, then gets or sets its center/span `MapRegion` and replaces caller-owned marker snapshots.

`MapSpan::new` rejects non-finite, non-positive, or over-world deltas (latitude `(0, 180]`,
longitude `(0, 360]`). The center uses `MapCoordinate`, which bounds finite latitude to `-90..=90`
and longitude to `-180..=180`. MapKit can adjust a requested region to fit the view's dimensions;
the getter returns `None` if the native center or span fails portable validation.

The view is a lower-level imperative UIKit primitive, not SwiftUI `Map`. The host controls its
frame, constraints, and view hierarchy. This slice does not enable user location, prompt for
permissions, create overlays, search, geocode, calculate directions, or provide offline maps.
MapKit owns basemap content and any native content/network fetch behavior. No claim is made that
map tiles load successfully or that compile/link validation proves live rendering.

`MKMapView` and its `region`/`setRegion:animated:` APIs are public since iOS 3.0 in the inspected
iOS SDK. `objc2-map-kit 0.3.2` exposes its generated `MKMapView` only under macOS/AppKit, so
`ios-map-view` uses a narrow `objc2` declaration that matches the public iOS `MKMapView.h` class,
initializer, getter, setter, and by-value generated `MKCoordinateRegion` struct ABI. It uses no
Swift source, private symbols, location service, or special entitlement.

## Marker snapshots

`MapAnnotation::new(id, coordinate, title)` builds a portable, caller-owned marker value. The
application keeps ids stable between snapshots and must make them unique within each snapshot.
`IosMapView::replace_annotations` rejects a duplicate id without changing the native annotation
set. For ids present before and after replacement it reuses the same `MKPointAnnotation`, updates
the public coordinate and copied optional title, removes omitted ids, and creates objects for new
ids. Call the method on the main thread; `IosMapView`'s `MainThreadMarker`-bound native member
enforces this at the Rust type boundary.

This increment uses public `MKPointAnnotation` (available from iOS 4.0) and `MKMapView` add/remove
selectors. It supplies only MapKit's built-in point annotation behavior; there is no custom view,
selection callback, subtitle, image, clustering policy, overlay, or SwiftUI-style state renderer.
The registered tests prove value and duplicate-id policy; device and Simulator link checks prove
compile/link symbols only, not visible pins, title callouts, touch behavior, or runtime map content.

## Geometry conversion

`ios-maps` provides three synchronous, by-value MapKit calls:

- `map_point_for_coordinate(MapCoordinate) -> Option<MapPoint>` calls `MKMapPointForCoordinate`
- `coordinate_for_map_point(MapPoint) -> Option<MapCoordinate>` calls `MKCoordinateForMapPoint`
- `meters_between_map_points(MapPoint, MapPoint) -> Option<MapDistanceMeters>` calls
  `MKMetersBetweenMapPoints`

The functions are available from iOS 4.0. Each checks runtime availability. A returned `None`
means either that the API is unavailable or that a native result is non-finite, outside the
coordinate range, or negative where a distance is expected. `MapCoordinate` accepts finite degree
values with latitude in `-90..=90` and longitude in `-180..=180`; `MapPoint` accepts finite x/y
projection values without asserting world-rectangle membership. Distance uses MapKit's returned
surface distance, not a Euclidean calculation over map points.

The geometry adapter uses `objc2-map-kit` 0.3.2 with default features disabled and only `MKGeometry`
and `objc2-core-location` enabled. `ios-map-view` additionally enables `MKAnnotation`,
`MKPointAnnotation`, and `MKShape` from the same generated crate. `ios-maps` also has a direct
`objc2-core-location` 0.3.2 dependency
with only `CLLocation` enabled to import the coordinate value type. This feature closure links
`MapKit.framework` and the binding's `objc2-core-location` dependency also contributes
`CoreLocation.framework` to link metadata; the adapter calls no Core Location service API. It uses
no Swift, object handles, pointers, callbacks, or retained state. The functions require no location
permission request, location usage-description key, MapKit entitlement, map UI, host registration,
or main-thread context.

The scalar geometry claims above remain limited to those functions. The map-view crate does not
use SwiftUI `Map`, access the user's location, access map tiles or network services, search,
geocode, request directions, or provide offline map content. Do not infer that this package
validates any of those behaviors.

`sh platform/ios/ios-maps/check.sh` is the package-local format, compile, Clippy, rustdoc, and link
audit. `sh platform/ios/ios-maps/check-link-imports.sh` builds device and Simulator artifacts and
inspects their imports and symbol strings; it never executes those artifacts. Link-probe deployment
targets do not raise the MapKit API floor. See [MapKit capability scope](../capabilities/mapkit.md)
and [D77](../../PLAN_CAPABILITIES_MAPKIT.md).

Apple sources: [MKPointAnnotation](https://developer.apple.com/documentation/mapkit/mkpointannotation),
[MKMapView annotations](https://developer.apple.com/documentation/mapkit/mkmapview),
[MKMapPoint](https://developer.apple.com/documentation/mapkit/mkmappoint),
[MKMetersBetweenMapPoints](https://developer.apple.com/documentation/mapkit/mkmappoint/distance%28to%3A%29),
[CLLocationCoordinate2D](https://developer.apple.com/documentation/corelocation/cllocationcoordinate2d),
and [MapKit](https://developer.apple.com/documentation/mapkit).
