#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use ios_maps::{
        MapCoordinate, coordinate_for_map_point, map_point_for_coordinate,
        meters_between_map_points,
    };

    let _ = core::hint::black_box(ios_maps::world_tracking_support());
    let coordinate = core::hint::black_box(MapCoordinate::new(12.0, 34.0));
    let point = coordinate.and_then(map_point_for_coordinate);
    let round_trip = point.and_then(coordinate_for_map_point);
    let distance = point.and_then(|point| meters_between_map_points(point, point));
    let _ = core::hint::black_box((coordinate, point, round_trip, distance));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
