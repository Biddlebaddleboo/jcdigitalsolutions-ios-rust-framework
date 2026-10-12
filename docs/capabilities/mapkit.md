# MapKit

`framework-maps` defines finite portable geographic coordinates, MapKit projection points,
non-negative distances, validated map spans, and center/span regions. `ios-maps` adapts the geometry
values to public MapKit C functions. The separate `ios-map-view` crate creates an embeddable native
UIKit `MKMapView` and gets or sets a validated `MapRegion`.

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
The wrapper does not enable user location, request location authorization, add annotations or
overlays, query places, geocode addresses, calculate directions, or provide offline content.
MapKit controls basemap presentation and any native content/network requests; this crate does not
promise successful tile fetches or claim runtime map behavior from compile/link checks.

See the [iOS MapKit guide](../ios/mapkit.md), [D77/B73 and B577 focused plan](../../PLAN_CAPABILITIES_MAPKIT.md), and [registered validation plan](../../PLAN_VALIDATION_IOS_MAPKIT.md).
