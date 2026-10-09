# MapKit geometry values

`framework-maps` defines finite portable values for a caller-owned latitude/longitude coordinate,
a point in MapKit's 2D projection, and a non-negative distance in meters. `ios-maps` adapts those
values to three public MapKit C functions: coordinate-to-map-point conversion,
map-point-to-coordinate conversion, and distance between map points.

`MapCoordinate::new` rejects non-finite input, latitude outside `-90..=90`, or longitude outside
`-180..=180`. `MapPoint::new` rejects non-finite projected x/y values but does not assert that a
point lies in MapKit's world rectangle. `MapDistanceMeters::new` rejects non-finite or negative
values. The iOS conversion functions return `None` if the API is unavailable or a MapKit result
does not pass the corresponding portable-value validation.

MapKit map points use projection units, not meters or screen points. Apple advises persisting
coordinate values instead of projected points. MapKit's distance API returns surface distance over
the globe, not Euclidean distance over projected x/y values.

This is a geometry-only slice. It does not create or show a map, download map content, query a
place, geocode an address, calculate directions, access device location, request location
authorization, or represent location authorization state. It makes no map-service, network,
user-location, or full-MapKit-parity claim.

See the [iOS MapKit guide](../ios/mapkit.md) and [D77 focused plan](../../PLAN_CAPABILITIES_MAPKIT.md).
