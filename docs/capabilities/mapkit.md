# MapKit

`framework-maps` defines finite portable geographic coordinates, MapKit projection points,
non-negative distances, validated map spans and center/span regions, plus caller-owned marker values.
`ios-maps` adapts the geometry values to public MapKit C functions. The separate `ios-map-view`
crate creates an embeddable native UIKit `MKMapView`, gets or sets a validated `MapRegion`,
reconciles marker snapshots, offers one bounded route and polygon convenience overlay, and supports
caller-controlled user-location tracking. B602 also exposes the native UIKit/MapKit surface for
external Rust UI hosts that need direct view composition, arbitrary native overlays, delegate
callbacks, camera/viewport control, or annotation selection.

`MapCoordinate::new` rejects non-finite input, latitude outside `-90..=90`, or longitude outside
`-180..=180`. `MapPoint::new` rejects non-finite projected x/y values but does not assert that a
point lies in MapKit's world rectangle. `MapDistanceMeters::new` rejects non-finite or negative
values. The iOS conversion functions return `None` if the API is unavailable or a MapKit result
does not pass the corresponding portable-value validation.

MapKit map points use projection units, not meters or screen points. Apple advises persisting
coordinate values instead of projected points. MapKit's distance API returns surface distance over
the globe, not Euclidean distance over projected x/y values.

The map view is an imperative native UIKit primitive, not SwiftUI or a declarative map framework.
Its host owns layout and hierarchy. `IosMapView::ui_view` borrows the same map object as `UIView`,
so the host can use native parenting, frame/bounds, visibility, layout, hit-testing, and teardown.
MapKit may adjust a requested region or map rectangle to the view's dimensions.
User-location display is off by default. B586 adds explicit `None`, `Follow`, and `FollowHeading`
modes; follow modes require existing When In Use or Always authorization, fail closed otherwise,
and never request or change authorization. The host owns permission UX and the
`NSLocationWhenInUseUsageDescription` usage string. The wrapper does not query places, geocode
addresses, calculate directions, or provide offline content. MapKit controls
basemap presentation and any native content/network requests; this crate does not promise
successful tile fetches or claim runtime map behavior from compile/link checks.

## Native UIKit and MapKit surface

`NativeMapView` exposes native region, visible-map-rectangle, edge-padding, camera, overlay,
annotation, selection, renderer lookup, and delegate reads. `add_overlay` and
`add_overlay_at_level` accept any live object that conforms to Apple's `MKOverlay`; multiple
independent overlays can coexist. The host retains each native overlay handle and removes only the
overlays it owns. Generated public types are re-exported for `MKMapRect`, `MKMapCamera`, annotations,
overlays, `MKPolyline`, `MKPolygon`, and their renderers. Native path renderer properties include
line width and dash pattern.

`IosMapViewDelegate` is the public iOS `MKMapViewDelegate` subset for region changes, annotation
view selection, and nullable overlay-renderer callbacks. `IosMapView::set_delegate` retains a
delegate for MapKit's weak slot and clears only its own slot on explicit clear or drop. Built-in
route/polygon rendering uses that same native slot, so those convenience operations reject a host
delegate, and host delegate install rejects active built-in overlays. The external delegate owns its
callback state and must handle native reentrancy without panics across the Objective-C ABI.

The selected generated binding omits iOS `MKMapView` and `MKAnnotationView` class declarations;
the crate declares only these public `UIView` subclasses locally. It reuses generated MapKit
geometry/camera/annotation/overlay/renderer bindings. iOS edge padding uses `UIEdgeInsets`, as
declared by the public iOS SDK header, rather than the generated macOS `NSEdgeInsets` signature.

Static compile and link evidence does not prove map display, touch behavior, delegate delivery,
rotation, or tile fetch.

## Marker snapshots

`MapAnnotation` carries an application-defined `u64` id, a validated `MapCoordinate`, and an
optional borrowed title. `IosMapView::replace_annotations` takes the complete caller-owned
snapshot, rejects duplicate ids before native changes, preserves each `MKPointAnnotation` object
for ids that remain, updates its coordinate/title, and removes ids omitted from the new snapshot.
The method is main-thread-bound through `IosMapView`. MapKit copies non-null titles. This supports
basic native point markers; it does not create custom annotation views or callout actions, or provide
SwiftUI `Map` parity. Native selection APIs and annotation-view delegate callbacks are available
through the B602 escape surface. Link checks do not prove rendered markers or interactive callouts.

## One route path

`IosMapView::replace_route` accepts 2 through 4,096 `MapCoordinate` values and replaces one native
`MKPolyline`. Invalid point counts reject before native state changes. `clear_route` removes only
that overlay; neither it nor drop changes marker annotations. `MKPolylineRenderer` uses a fixed
`UIColor.blueColor()` stroke at 5 points. This is a route display primitive, not a route planner.

## One polygon

`IosMapView::replace_polygon` accepts 3 through 4,096 `MapCoordinate` values and replaces one
native `MKPolygon`; MapKit connects the final vertex to the first. It rejects invalid counts before
native state changes. `clear_polygon` and drop remove only this polygon. The renderer uses a fixed
20%-alpha blue fill and 2-point blue stroke. One route and one polygon coexist through the same
wrapper-owned delegate. Polygon holes, custom styles, and arbitrary overlay collections are not
part of this convenience API; native MapKit models and styles remain available through the B602
escape surface. `MapPolygonError` distinguishes short input, over-bound input, and an occupied host
delegate slot.

MapKit's `MKMapView.delegate` is weak and has one slot. While a route or polygon is active,
`IosMapView` retains and owns the shared renderer delegate. Activation returns its corresponding
`DelegateSlotOccupied` error if a host delegate is already installed. The host must not replace the
delegate while either overlay is active. If it does, replacement fails; clear/drop remove only
owned overlays and preserve the host's delegate. Arbitrary overlays use the B602 native methods and
the host's retained delegate. User tracking is an independent opt-in mode that does not use the
overlay delegate slot. Static validation proves policy, compile, link, and import checks only, not
route/polygon pixels, touch, or runtime map behavior.

See the [iOS MapKit guide](../ios/mapkit.md), [D77/B73 and B577/B581 focused plan](../../PLAN_CAPABILITIES_MAPKIT.md), and [registered validation plan](../../PLAN_VALIDATION_IOS_MAPKIT.md).
