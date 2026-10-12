/// A polygon could not be applied to a native map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapPolygonError {
    /// A polygon needs at least three coordinates.
    TooFewCoordinates,
    /// A polygon may contain no more than [`MAX_POLYGON_COORDINATES`] coordinates.
    TooManyCoordinates,
    /// The native map delegate slot belongs to another object.
    DelegateSlotOccupied,
}

/// Maximum coordinate count for one polygon.
pub const MAX_POLYGON_COORDINATES: u32 = 4_096;

#[cfg(any(target_os = "ios", test))]
pub(crate) fn validate_polygon_point_count(point_count: usize) -> Result<(), MapPolygonError> {
    if point_count < 3 {
        return Err(MapPolygonError::TooFewCoordinates);
    }
    if point_count > MAX_POLYGON_COORDINATES as usize {
        return Err(MapPolygonError::TooManyCoordinates);
    }
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
pub(crate) fn validate_polygon_delegate_slot(
    delegate_is_empty: bool,
    delegate_is_owned: bool,
) -> Result<(), MapPolygonError> {
    if !crate::route::overlay_delegate_slot_is_available(delegate_is_empty, delegate_is_owned) {
        return Err(MapPolygonError::DelegateSlotOccupied);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_POLYGON_COORDINATES, MapPolygonError, validate_polygon_delegate_slot,
        validate_polygon_point_count,
    };

    #[test]
    fn polygon_rejects_too_few_coordinates_before_delegate_policy() {
        for point_count in 0..3 {
            assert_eq!(
                validate_polygon_point_count(point_count),
                Err(MapPolygonError::TooFewCoordinates)
            );
        }
    }

    #[test]
    fn polygon_accepts_three_and_the_maximum_coordinate_count() {
        assert_eq!(validate_polygon_point_count(3), Ok(()));
        assert_eq!(
            validate_polygon_point_count(MAX_POLYGON_COORDINATES as usize),
            Ok(())
        );
    }

    #[test]
    fn polygon_rejects_coordinates_over_the_bounded_limit() {
        assert_eq!(
            validate_polygon_point_count(MAX_POLYGON_COORDINATES as usize + 1),
            Err(MapPolygonError::TooManyCoordinates)
        );
    }

    #[test]
    fn polygon_rejects_an_unrelated_host_delegate() {
        assert_eq!(
            validate_polygon_delegate_slot(false, false),
            Err(MapPolygonError::DelegateSlotOccupied)
        );
        assert_eq!(validate_polygon_delegate_slot(true, false), Ok(()));
        assert_eq!(validate_polygon_delegate_slot(false, true), Ok(()));
    }
}
