#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded ARKit support and MapKit geometry APIs for iOS."]

pub use framework_maps::{MapCoordinate, MapDistanceMeters, MapPoint};

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::world_tracking_support;
#[cfg(target_os = "ios")]
pub use platform::{coordinate_for_map_point, map_point_for_coordinate, meters_between_map_points};

/// Returns no iOS world-tracking status on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn world_tracking_support() -> Option<framework_maps::WorldTrackingSupport> {
    None
}

/// Returns no MapKit result on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn map_point_for_coordinate(
    _coordinate: framework_maps::MapCoordinate,
) -> Option<framework_maps::MapPoint> {
    None
}

/// Returns no MapKit result on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn coordinate_for_map_point(
    _point: framework_maps::MapPoint,
) -> Option<framework_maps::MapCoordinate> {
    None
}

/// Returns no MapKit result on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn meters_between_map_points(
    _first: framework_maps::MapPoint,
    _second: framework_maps::MapPoint,
) -> Option<framework_maps::MapDistanceMeters> {
    None
}
