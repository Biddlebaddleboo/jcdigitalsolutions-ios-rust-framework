use framework_maps::{MapCoordinate, MapDistanceMeters, MapPoint, WorldTrackingSupport};
use objc2::{ClassType, msg_send};
use objc2_ar_kit::ARWorldTrackingConfiguration;
use objc2_core_location::CLLocationCoordinate2D;
use objc2_map_kit::{MKCoordinateForMapPoint, MKMapPoint, MKMetersBetweenMapPoints};

/// Queries whether this system reports support for an ARKit world-tracking configuration.
///
/// Returns `None` when the runtime is earlier than iOS 11.0. This query does not create an
/// `ARSession`, access the camera, or request camera permission.
pub fn world_tracking_support() -> Option<WorldTrackingSupport> {
    if !objc2::available!(ios = 11.0, ..) {
        return None;
    }

    let class = ARWorldTrackingConfiguration::class();
    // SAFETY: `ARWorldTrackingConfiguration` is the concrete class receiver for the inherited
    // `ARConfiguration.isSupported` class property. Apple declares this property as a BOOL and
    // objc2-ar-kit binds the same property as `bool` on ARConfiguration. The runtime API floor is
    // checked above; this call has no object, pointer, ownership, or camera-access precondition.
    let supported: bool = unsafe { msg_send![class, isSupported] };
    Some(WorldTrackingSupport::from_system(supported))
}

/// Converts a caller-supplied coordinate to a finite MapKit map point.
///
/// Returns `None` before iOS 4.0 or when MapKit returns a non-finite projected value. The input
/// coordinate is validated by `MapCoordinate`; no location service, permission, map view, or
/// network-backed MapKit service is used.
pub fn map_point_for_coordinate(coordinate: MapCoordinate) -> Option<MapPoint> {
    if !objc2::available!(ios = 4.0, ..) {
        return None;
    }

    let coordinate = CLLocationCoordinate2D {
        latitude: coordinate.latitude_degrees(),
        longitude: coordinate.longitude_degrees(),
    };
    // SAFETY: the generated binding accepts this repr(C) coordinate by value. Both fields come
    // from the finite, geographic-range-checked portable value; the call receives no pointers,
    // objects, callbacks, or retained state. The iOS 4.0 runtime floor is checked above.
    let point = unsafe { MKMapPoint::for_coordinate(coordinate) };
    MapPoint::new(point.x, point.y)
}

/// Converts a caller-supplied finite map point to a valid geographic coordinate.
///
/// Returns `None` before iOS 4.0 or when MapKit returns a coordinate outside the finite latitude
/// and longitude ranges accepted by `MapCoordinate`. Map points are projection values, not
/// persistent geographic coordinates.
pub fn coordinate_for_map_point(point: MapPoint) -> Option<MapCoordinate> {
    if !objc2::available!(ios = 4.0, ..) {
        return None;
    }

    let point = MKMapPoint {
        x: point.x(),
        y: point.y(),
    };
    // SAFETY: the generated binding accepts this repr(C) map point by value. Both fields are
    // finite by construction; the call receives no pointers, objects, callbacks, or retained
    // state. The iOS 4.0 runtime floor is checked above.
    let coordinate = unsafe { MKCoordinateForMapPoint(point) };
    MapCoordinate::new(coordinate.latitude, coordinate.longitude)
}

/// Returns MapKit's surface distance between two caller-supplied finite map points.
///
/// Returns `None` before iOS 4.0 or when MapKit returns a non-finite or negative distance. The
/// result is MapKit's distance over the globe's surface, not Euclidean distance between projected
/// `x`/`y` values.
pub fn meters_between_map_points(first: MapPoint, second: MapPoint) -> Option<MapDistanceMeters> {
    if !objc2::available!(ios = 4.0, ..) {
        return None;
    }

    let first = MKMapPoint {
        x: first.x(),
        y: first.y(),
    };
    let second = MKMapPoint {
        x: second.x(),
        y: second.y(),
    };
    // SAFETY: both generated repr(C) points contain finite `f64` fields by construction. This
    // by-value C call has no pointers, objects, callbacks, or retained state; its iOS 4.0 runtime
    // floor is checked above.
    let meters = unsafe { MKMetersBetweenMapPoints(first, second) };
    MapDistanceMeters::new(meters)
}
