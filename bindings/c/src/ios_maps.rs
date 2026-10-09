use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};
use framework_maps::{MapCoordinate, MapPoint};

#[cfg(target_os = "ios")]
use ios_maps::{coordinate_for_map_point, map_point_for_coordinate, meters_between_map_points};

/// MapKit distance in meters as an IEEE-754 binary64 value.
pub type FrameworkIosMapsDistanceMeters = f64;

/// Converts a geographic coordinate to a MapKit map point.
///
/// Latitude must be finite and in `-90..=90` degrees; longitude must be finite and in
/// `-180..=180` degrees. MapKit map-point units are not meters. A native result is accepted only
/// when both projected values are finite. The MapKit geometry API floor is iOS 4.0.
///
/// # Safety
/// Each non-null output must point to aligned writable storage. Both outputs are required and
/// must be distinct and non-overlapping. Each non-null output is initialized to zero before any
/// argument or platform-status result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_maps_map_point_for_coordinate(
    latitude_degrees: f64,
    longitude_degrees: f64,
    out_x: *mut f64,
    out_y: *mut f64,
) -> FrameworkStatus {
    // SAFETY: The caller promises each non-null output is writable.
    unsafe {
        if !out_x.is_null() {
            out_x.write(0.0);
        }
        if !out_y.is_null() {
            out_y.write(0.0);
        }
    }
    if out_x.is_null() || out_y.is_null() || out_x == out_y {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Some(coordinate) = MapCoordinate::new(latitude_degrees, longitude_degrees) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(point) = map_point_for_coordinate(coordinate) else {
                return FrameworkStatus::UNAVAILABLE;
            };
            // SAFETY: The caller supplied distinct writable output slots above.
            unsafe {
                out_x.write(point.x());
                out_y.write(point.y());
            }
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = coordinate;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Converts a finite MapKit map point to geographic degrees.
///
/// The native result must have finite latitude in `-90..=90` degrees and longitude in
/// `-180..=180` degrees. The MapKit geometry API floor is iOS 4.0.
///
/// # Safety
/// Each non-null output must point to aligned writable storage. Both outputs are required and
/// must be distinct and non-overlapping. Each non-null output is initialized to zero before any
/// argument or platform-status result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_maps_coordinate_for_map_point(
    x: f64,
    y: f64,
    out_latitude_degrees: *mut f64,
    out_longitude_degrees: *mut f64,
) -> FrameworkStatus {
    // SAFETY: The caller promises each non-null output is writable.
    unsafe {
        if !out_latitude_degrees.is_null() {
            out_latitude_degrees.write(0.0);
        }
        if !out_longitude_degrees.is_null() {
            out_longitude_degrees.write(0.0);
        }
    }
    if out_latitude_degrees.is_null()
        || out_longitude_degrees.is_null()
        || out_latitude_degrees == out_longitude_degrees
    {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    let Some(point) = MapPoint::new(x, y) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(coordinate) = coordinate_for_map_point(point) else {
                return FrameworkStatus::UNAVAILABLE;
            };
            // SAFETY: The caller supplied distinct writable output slots above.
            unsafe {
                out_latitude_degrees.write(coordinate.latitude_degrees());
                out_longitude_degrees.write(coordinate.longitude_degrees());
            }
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = point;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Returns MapKit's surface distance between two finite map points.
///
/// The result is not Euclidean distance in map-point units. The MapKit geometry API floor is iOS
/// 4.0.
///
/// # Safety
/// `out_meters` must point to aligned writable storage. It is initialized to zero before any
/// argument or platform-status result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_maps_meters_between_map_points(
    first_x: f64,
    first_y: f64,
    second_x: f64,
    second_y: f64,
    out_meters: *mut FrameworkIosMapsDistanceMeters,
) -> FrameworkStatus {
    if out_meters.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises this output is writable.
    unsafe { out_meters.write(0.0) };
    let (Some(first), Some(second)) = (
        MapPoint::new(first_x, first_y),
        MapPoint::new(second_x, second_y),
    ) else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(distance) = meters_between_map_points(first, second) else {
                return FrameworkStatus::UNAVAILABLE;
            };
            // SAFETY: The caller supplied writable output storage above.
            unsafe { out_meters.write(distance.meters()) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = (first, second);
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
