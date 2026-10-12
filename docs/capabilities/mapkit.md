# MapKit

`framework-maps` defines finite portable geographic coordinates, MapKit projection points,
non-negative distances, validated map spans and center/span regions, plus caller-owned marker values.
`ios-maps` adapts the geometry values to public MapKit C functions. The separate `ios-map-view`
crate creates an embeddable native UIKit `MKMapView`, gets or sets a validated `MapRegion`,
reconciles marker snapshots, and displays one caller-supplied route path.

`MapCoordinate::new` rejects non-finite input, latitude outside `-90..=90`, or longitude outside
`-180..=180`. `MapPoint::new` rejects non-finite projected x/y values but does not assert that a
point lies in MapKit's world rectangle. `MapDistanceMeters::new` rejects non-finite or negative
values. The iOS conversion functions return `None` if the API is unavailable or a MapKit result
does not pass the corresponding portable-value validation.

MapKit map points use projection units, not meters or screen points. Apple advises persisting
coordinate values instead of projected points. MapKit's distance API returns surface distance over
the globe, not Euclidean distance over projected x/y values.

The map view is an imperative native UIKit primitive, not SwiftUI or a declarative map framework.
Its host owns layout and hierarchy. MapKit may adjust a requested region to the view's dimensions.
The wrapper does not enable user location, request location authorization, query places, geocode
addresses, calculate directions, or provide offline content. MapKit controls
basemap presentation and any native content/network requests; this crate does not promise
successful tile fetches or claim runtime map behavior from compile/link checks.

## Marker snapshots

`MapAnnotation` carries an application-defined `u64` id, a validated `MapCoordinate`, and an
optional borrowed title. `IosMapView::replace_annotations` takes the complete caller-owned
snapshot, rejects duplicate ids before native changes, preserves each `MKPointAnnotation` object
for ids that remain, updates its coordinate/title, and removes ids omitted from the new snapshot.
The method is main-thread-bound through `IosMapView`. MapKit copies non-null titles. This supports
basic native point markers; it does not provide custom annotation views, callout actions, other
overlay types, user location, or SwiftUI `Map` parity. Link checks do not prove rendered markers or interactive
callouts.

## One route path

`IosMapView::replace_route` accepts 2 through 4,096 `MapCoordinate` values and replaces one native
`MKPolyline`. Invalid point counts reject before native state changes. `clear_route` and drop remove
only that overlay; neither changes marker annotations. `MKPolylineRenderer` uses a fixed
`UIColor.blueColor()` stroke at 5 points. This is a route display primitive, not a route planner or
a general overlay collection.

MapKit's `MKMapView.delegate` is weak and has one slot. While a route is active, `IosMapView` retains
and owns the renderer delegate. Route activation returns `MapRouteError::DelegateSlotOccupied` if a
host delegate is already installed. The host must not replace the delegate while a route is active.
If it does, route replacement fails; clear/drop still remove the route and preserve the host's
delegate. Static validation proves policy, compile, link, and import checks only, not route pixels,
touch, or runtime map behavior.

See the [iOS MapKit guide](../ios/mapkit.md), [D77/B73 and B577/B581 focused plan](../../PLAN_CAPABILITIES_MAPKIT.md), and [registered validation plan](../../PLAN_VALIDATION_IOS_MAPKIT.md).
