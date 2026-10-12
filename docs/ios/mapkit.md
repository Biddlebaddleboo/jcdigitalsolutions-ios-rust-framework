# iOS MapKit

`ios-maps` contains MapKit coordinate projection and distance calls. The independent
`ios-map-view` crate creates a native, main-thread-bound `MKMapView` that a host UIKit or Rust UI
engine can embed. It keeps validated `MapRegion` access, caller-owned marker snapshots, one route and
one polygon convenience overlay, and caller-controlled user-location tracking. B602 adds direct
native UIKit composition and MapKit overlay, delegate, camera, viewport, and annotation-selection
APIs.

`MapSpan::new` rejects non-finite, non-positive, or over-world deltas (latitude `(0, 180]`,
longitude `(0, 360]`). The center uses `MapCoordinate`, which bounds finite latitude to `-90..=90`
and longitude to `-180..=180`. MapKit can adjust a requested region to fit the view's dimensions;
the getter returns `None` if the native center or span fails portable validation.

The view is a lower-level imperative UIKit primitive, not SwiftUI `Map`. `IosMapView::native_view`
borrows `NativeMapView`; `IosMapView::ui_view` borrows its UIKit `UIView` superclass. The host
controls its frame, constraints, parent/child hierarchy, layout, and teardown with native UIKit
methods. User-location display is off by default; B586 adds an
explicit typed tracking mode that requires existing authorization and never requests or changes
authorization. Search, geocoding, directions, and offline maps remain out of scope.
MapKit owns basemap content and any native content/network fetch behavior. No claim is made that
map tiles load successfully or that compile/link validation proves live rendering.

`MKMapView` and its `region`/`setRegion:animated:` APIs are public since iOS 3.0 in the inspected
iOS SDK. `objc2-map-kit 0.3.2` exposes its generated `MKMapView` only under macOS/AppKit, so
`ios-map-view` uses a narrow `objc2` declaration that matches the public iOS `MKMapView.h` class,
initializer, region, annotation, delegate, overlay, and user-tracking selectors, plus the by-value
generated `MKCoordinateRegion` struct ABI. It also declares the exact iOS `MKMapViewDelegate`
overlay callback because the selected binding omits that iOS protocol method. It uses no Swift
source, private symbols, or special entitlement.

## Native UIKit and MapKit surface

`NativeMapView` wraps native region, visible `MKMapRect`, edge-padding, `MKMapCamera`, overlay,
annotation, selection, and renderer-lookup selectors. It supports repeated `add_overlay` and
`add_overlay_at_level` calls for multiple independent native overlays. The host retains native
overlay objects and removes only the overlays it added. Re-exported types include MapKit geometry,
camera, annotations, overlays, polyline/polygon models, and overlay renderers. `MKOverlayPathRenderer`
exposes native fill/stroke, line-width, and line-dash APIs.

`IosMapViewDelegate` is the local iOS declaration for Apple's `MKMapViewDelegate` protocol subset.
It includes region-will/did-change, visible-region, annotation-view selection, and nullable
`rendererForOverlay:` callbacks. `IosMapView::set_delegate` retains a Rust delegate for MapKit's
weak slot; explicit clear or wrapper drop clears that slot only when it still points to the owned
delegate. The unsafe protocol implementation contract requires exact Objective-C ABI, main-thread
callbacks, no panic across the foreign boundary, valid callback state, and reentrancy safety.

The selected `objc2-map-kit 0.3.2` class declarations for `MKMapView` and `MKAnnotationView` are
macOS/AppKit-only. This crate supplies narrow iOS `UIView` subclass declarations and reuses the
generated public MapKit value/object/renderer bindings. UIKit edge-padding uses the SDK's
`UIEdgeInsets` struct encoding. The host delegate cannot share MapKit's weak slot with the
convenience route/polygon renderer: host delegate install rejects active convenience overlays, and
the convenience overlay methods reject a host-owned delegate. Native overlays added through
`NativeMapView` remain independent and can share the host's own delegate.

The registered device/Simulator probes compile and link this surface, but do not execute MapKit.
Visible overlays, callbacks, annotation touches, camera motion, view rotation, basemap requests, and
physical-device behavior remain runtime-unverified.

## Marker snapshots

`MapAnnotation::new(id, coordinate, title)` builds a portable, caller-owned marker value. The
application keeps ids stable between snapshots and must make them unique within each snapshot.
`IosMapView::replace_annotations` rejects a duplicate id without changing the native annotation
set. For ids present before and after replacement it reuses the same `MKPointAnnotation`, updates
the public coordinate and copied optional title, removes omitted ids, and creates objects for new
ids. Call the method on the main thread; `IosMapView`'s `MainThreadMarker`-bound native member
enforces this at the Rust type boundary.

This increment uses public `MKPointAnnotation` (available from iOS 4.0) and `MKMapView` add/remove
selectors. It supplies only MapKit's built-in point annotation behavior; it does not create custom
view objects or expose subtitle, image, clustering policy, or a SwiftUI-style state renderer. Native
selection and annotation-view delegate callbacks are available through B602.
The registered tests prove value and duplicate-id policy; device and Simulator link checks prove
compile/link symbols only, not visible pins, title callouts, touch behavior, or runtime map content.

## One route path

`IosMapView::replace_route(&[MapCoordinate])` accepts 2 through 4,096 coordinates and replaces its
one route path. Each coordinate is already range-checked by `MapCoordinate`; the wrapper passes the
corresponding `CLLocationCoordinate2D` values to public `MKPolyline` and applies a fixed
`MKPolylineRenderer` style of UIKit blue at 5 points. `clear_route` and drop remove only this route
overlay, not marker annotations. The API does not calculate or simplify routes.

`MKMapView.delegate` is a weak singleton slot. A route needs that slot for public
`mapView:rendererForOverlay:` callbacks, so route activation fails with
`MapRouteError::DelegateSlotOccupied` if a host delegate already owns it. While a route is active,
the host must not replace this delegate through `native_view()`. A later replacement request detects
such a foreign delegate and fails; clear/drop remove the route and clear the slot only if it still
holds this wrapper's delegate. The delegate does not alter marker annotations. Link checks do not
prove a visible route on Simulator or device.

## One polygon

`IosMapView::replace_polygon(&[MapCoordinate])` accepts 3 through 4,096 coordinates and replaces
one public `MKPolygon`; MapKit connects the final vertex to the first. `clear_polygon` and drop
remove only this owned polygon. The fixed renderer uses a 20%-alpha UIKit blue fill and 2-point
blue stroke. Invalid counts reject before native map state changes. Polygon holes, caller styles,
and arbitrary overlay collections are not part of this convenience API; native MapKit models and
renderer styles remain available through B602.

The route and polygon share one wrapper-owned delegate and one of each can coexist. Each activation
returns its corresponding `DelegateSlotOccupied` error if a host delegate already occupies MapKit's
weak singleton slot. The host must not replace this delegate through `native_view()` while either
overlay is active. Clear/drop remove only the corresponding owned overlay and release the slot only
after both are gone and only if it still points to this wrapper. This convenience delegate does not
forward host callbacks. These limits apply to the route/polygon helpers; B602 exposes native overlay
and delegate control for other MapKit models. Device/Simulator link checks prove public selectors,
not rendered route or polygon pixels.

## Caller-controlled user location

`IosMapView::set_user_tracking_mode` accepts `IosUserTrackingMode::{None, Follow,
FollowHeading}`. `None` always succeeds, sets MapKit's user tracking mode to none, and disables
`showsUserLocation`. `Follow` and `FollowHeading` read the current app status through the public
`CLLocationManager.authorizationStatus()` class method and proceed only when status is
`AuthorizedWhenInUse` or `AuthorizedAlways`. `NotDetermined`, `Restricted`, `Denied`, and unknown
statuses fail closed: the map's tracking/display is disabled and the method returns
`MapUserTrackingError::LocationAuthorizationRequired`. No location manager is created and no
authorization request or authorization mutation occurs. The status is checked for each call; the
host owns permission UX, handles status changes, and reapplies a desired mode as needed.

The app must provide `NSLocationWhenInUseUsageDescription` when it uses this optional feature.
The API is for foreground map display, not background location updates, continuous location
callbacks, or a Rust coordinate feed. It leaves the map's delegate slot unchanged, preserving the
overlay renderer's existing ownership. The public `userTrackingMode` and
`setUserTrackingMode:animated:` selectors are available from iOS 5.0; the status query is available
from iOS 4.2. The package validator's iOS 12.0 device and iOS 14.0 Simulator link floors remain
unchanged and are not runtime proof.

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
and `objc2-core-location` enabled. `ios-map-view` additionally enables annotation and shape
features plus `MKMapView`, `MKMultiPoint`, `MKOverlay`, `MKOverlayRenderer`, `MKOverlayPathRenderer`,
`MKPolygon`, `MKPolygonRenderer`, `MKPolyline`, and `MKPolylineRenderer` from the same generated crate.
It enables the `UIColor` UIKit binding for the fixed overlay styles. `ios-maps` has a direct
`objc2-core-location` 0.3.2 dependency
with only `CLLocation` enabled to import the coordinate value type; `ios-map-view` also enables
`CLLocationManager` to read authorization status for B586 without creating a manager. `MapKit.framework`
and `CoreLocation.framework` remain native link dependencies. The geometry adapter itself calls no
Core Location service API; it uses no Swift, object handles, pointers, callbacks, or retained state.
Its functions require no location permission, usage-description key, MapKit entitlement, map UI,
host registration, or main-thread context.

The scalar geometry claims above remain limited to those functions. The geometry adapter does not
access the user's location. The map-view crate's opt-in tracking mode asks MapKit to display and
follow the user's location only with preexisting authorization; it does not provide coordinates,
background updates, search, geocoding, directions, or offline map content. Do not infer runtime
location or map-content behavior from compile/link validation.

`sh platform/ios/ios-maps/check.sh` is the package-local format, compile, Clippy, rustdoc, and link
audit. `sh platform/ios/ios-maps/check-link-imports.sh` builds device and Simulator artifacts and
inspects their imports and symbol strings; it never executes those artifacts. Link-probe deployment
targets do not raise the MapKit API floor. See [MapKit capability scope](../capabilities/mapkit.md)
and [D77](../../PLAN_CAPABILITIES_MAPKIT.md).

Apple sources: [MKPointAnnotation](https://developer.apple.com/documentation/mapkit/mkpointannotation),
[MKPolygon](https://developer.apple.com/documentation/mapkit/mkpolygon),
[MKPolygonRenderer](https://developer.apple.com/documentation/mapkit/mkpolygonrenderer),
[MKMapView annotations](https://developer.apple.com/documentation/mapkit/mkmapview),
[MKMapView user tracking](https://developer.apple.com/documentation/mapkit/mkmapview/usertrackingmode),
[showing the user's location](https://developer.apple.com/documentation/mapkit/mkmapview/showsuserlocation),
[CLLocationManager authorization status](https://developer.apple.com/documentation/corelocation/cllocationmanager/authorizationstatus%28%29),
[MKMapPoint](https://developer.apple.com/documentation/mapkit/mkmappoint),
[MKMetersBetweenMapPoints](https://developer.apple.com/documentation/mapkit/mkmappoint/distance%28to%3A%29),
[CLLocationCoordinate2D](https://developer.apple.com/documentation/corelocation/cllocationcoordinate2d),
and [MapKit](https://developer.apple.com/documentation/mapkit).
