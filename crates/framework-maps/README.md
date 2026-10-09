# framework-maps

`framework-maps` contains portable, framework-owned map and spatial values. Its ARKit slice is
`WorldTrackingSupport`, a boolean snapshot supplied by an iOS backend. Its MapKit geometry values
are finite `MapCoordinate`, `MapPoint`, and `MapDistanceMeters` records; they do not contain Apple
classes or call platform APIs. `MapCoordinate::new` accepts finite latitude/longitude degrees in
`-90..=90` and `-180..=180`, `MapPoint::new` accepts finite projected x/y values, and
`MapDistanceMeters::new` accepts finite non-negative meters.

See the [ARKit capability guide](../../docs/capabilities/arkit.md) and [MapKit capability guide](../../docs/capabilities/mapkit.md).
