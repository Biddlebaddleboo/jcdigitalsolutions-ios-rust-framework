/// Caller-selected user-location behavior for the iOS map view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosUserTrackingMode {
    /// Stop following the user's location and stop displaying it on this map.
    None,
    /// Center the map on the user's location as it changes.
    Follow,
    /// Center the map on the user's location and orient it by heading.
    FollowHeading,
}

/// A requested user-location mode could not be enabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapUserTrackingError {
    /// The app does not already have When In Use or Always location authorization.
    LocationAuthorizationRequired,
}

#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocationAuthorization {
    NotDetermined,
    Restricted,
    Denied,
    Authorized,
    Unknown,
}

#[cfg(any(target_os = "ios", test))]
pub(crate) const fn validate_user_tracking_request(
    mode: IosUserTrackingMode,
    authorization: LocationAuthorization,
) -> Result<(), MapUserTrackingError> {
    match mode {
        IosUserTrackingMode::None => Ok(()),
        IosUserTrackingMode::Follow | IosUserTrackingMode::FollowHeading => match authorization {
            LocationAuthorization::Authorized => Ok(()),
            LocationAuthorization::NotDetermined
            | LocationAuthorization::Restricted
            | LocationAuthorization::Denied
            | LocationAuthorization::Unknown => {
                Err(MapUserTrackingError::LocationAuthorizationRequired)
            }
        },
    }
}

#[cfg(target_os = "ios")]
pub(crate) fn current_location_authorization() -> LocationAuthorization {
    use objc2_core_location::{CLAuthorizationStatus, CLLocationManager};

    // SAFETY: The public class method reads this app's current authorization status and neither
    // creates a location service nor changes authorization. It is available from iOS 4.2.
    #[allow(deprecated)]
    let status = unsafe { CLLocationManager::authorizationStatus_class() };
    match status {
        CLAuthorizationStatus::NotDetermined => LocationAuthorization::NotDetermined,
        CLAuthorizationStatus::Restricted => LocationAuthorization::Restricted,
        CLAuthorizationStatus::Denied => LocationAuthorization::Denied,
        CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse => {
            LocationAuthorization::Authorized
        }
        _ => LocationAuthorization::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IosUserTrackingMode, LocationAuthorization, MapUserTrackingError,
        validate_user_tracking_request,
    };

    #[test]
    fn none_mode_is_allowed_without_location_authorization() {
        for authorization in [
            LocationAuthorization::NotDetermined,
            LocationAuthorization::Restricted,
            LocationAuthorization::Denied,
            LocationAuthorization::Unknown,
        ] {
            assert_eq!(
                validate_user_tracking_request(IosUserTrackingMode::None, authorization),
                Ok(())
            );
        }
    }

    #[test]
    fn follow_modes_require_existing_location_authorization() {
        for mode in [
            IosUserTrackingMode::Follow,
            IosUserTrackingMode::FollowHeading,
        ] {
            assert_eq!(
                validate_user_tracking_request(mode, LocationAuthorization::Authorized),
                Ok(())
            );
        }
    }

    #[test]
    fn follow_modes_reject_unavailable_authorization_states() {
        for mode in [
            IosUserTrackingMode::Follow,
            IosUserTrackingMode::FollowHeading,
        ] {
            for authorization in [
                LocationAuthorization::NotDetermined,
                LocationAuthorization::Restricted,
                LocationAuthorization::Denied,
                LocationAuthorization::Unknown,
            ] {
                assert_eq!(
                    validate_user_tracking_request(mode, authorization),
                    Err(MapUserTrackingError::LocationAuthorizationRequired)
                );
            }
        }
    }
}
