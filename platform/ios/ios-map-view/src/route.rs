/// A route path could not be applied to a native map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapRouteError {
    /// A route needs at least two coordinates to form a line.
    TooFewCoordinates,
    /// A route may contain no more than [`MAX_ROUTE_COORDINATES`] coordinates.
    TooManyCoordinates,
    /// The native map delegate slot belongs to another object.
    DelegateSlotOccupied,
}

/// Maximum coordinate count for one route path.
pub const MAX_ROUTE_COORDINATES: u32 = 4_096;

#[cfg(any(target_os = "ios", test))]
pub(crate) fn validate_route_point_count(point_count: usize) -> Result<(), MapRouteError> {
    if point_count < 2 {
        return Err(MapRouteError::TooFewCoordinates);
    }
    if point_count > MAX_ROUTE_COORDINATES as usize {
        return Err(MapRouteError::TooManyCoordinates);
    }
    Ok(())
}

#[cfg(any(target_os = "ios", test))]
pub(crate) const fn route_delegate_slot_is_available(is_empty: bool, is_owned: bool) -> bool {
    is_empty || is_owned
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_ROUTE_COORDINATES, MapRouteError, route_delegate_slot_is_available,
        validate_route_point_count,
    };

    #[test]
    fn route_rejects_empty_and_single_point_paths() {
        assert_eq!(
            validate_route_point_count(0),
            Err(MapRouteError::TooFewCoordinates)
        );
        assert_eq!(
            validate_route_point_count(1),
            Err(MapRouteError::TooFewCoordinates)
        );
    }

    #[test]
    fn route_accepts_two_points_and_the_maximum_path() {
        assert_eq!(validate_route_point_count(2), Ok(()));
        assert_eq!(
            validate_route_point_count(MAX_ROUTE_COORDINATES as usize),
            Ok(())
        );
    }

    #[test]
    fn route_rejects_paths_over_the_bounded_coordinate_limit() {
        assert_eq!(
            validate_route_point_count(MAX_ROUTE_COORDINATES as usize + 1),
            Err(MapRouteError::TooManyCoordinates)
        );
    }

    #[test]
    fn route_delegate_policy_accepts_only_empty_or_owned_slots() {
        assert!(route_delegate_slot_is_available(true, false));
        assert!(route_delegate_slot_is_available(false, true));
        assert!(!route_delegate_slot_is_available(false, false));
    }
}
