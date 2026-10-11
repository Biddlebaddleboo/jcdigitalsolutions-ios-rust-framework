use framework_photos::{PhotoLibraryAddOnlyAuthorizationStatus, PhotoLibraryAuthorizationStatus};

/// Maps the generated PhotoKit `PHAuthorizationStatus` integer values to the portable status.
///
/// Values 0 through 4 are the public PhotoKit values `NotDetermined`, `Restricted`, `Denied`,
/// `Authorized`, and iOS 14's `Limited`, respectively. Unknown future values remain `Unknown`.
pub fn authorization_status_from_native(value: isize) -> PhotoLibraryAuthorizationStatus {
    match value {
        0 => PhotoLibraryAuthorizationStatus::NotDetermined,
        1 => PhotoLibraryAuthorizationStatus::Restricted,
        2 => PhotoLibraryAuthorizationStatus::Denied,
        3 => PhotoLibraryAuthorizationStatus::Authorized,
        4 => PhotoLibraryAuthorizationStatus::Limited,
        _ => PhotoLibraryAuthorizationStatus::Unknown,
    }
}

/// Maps the generated PhotoKit status to an add-only status without treating `Limited` as access.
pub fn add_only_authorization_status_from_native(
    value: isize,
) -> PhotoLibraryAddOnlyAuthorizationStatus {
    match value {
        0 => PhotoLibraryAddOnlyAuthorizationStatus::NotDetermined,
        1 => PhotoLibraryAddOnlyAuthorizationStatus::Restricted,
        2 => PhotoLibraryAddOnlyAuthorizationStatus::Denied,
        3 => PhotoLibraryAddOnlyAuthorizationStatus::Authorized,
        _ => PhotoLibraryAddOnlyAuthorizationStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_each_public_photo_authorization_status() {
        assert_eq!(
            authorization_status_from_native(0),
            PhotoLibraryAuthorizationStatus::NotDetermined
        );
        assert_eq!(
            authorization_status_from_native(1),
            PhotoLibraryAuthorizationStatus::Restricted
        );
        assert_eq!(
            authorization_status_from_native(2),
            PhotoLibraryAuthorizationStatus::Denied
        );
        assert_eq!(
            authorization_status_from_native(3),
            PhotoLibraryAuthorizationStatus::Authorized
        );
        assert_eq!(
            authorization_status_from_native(4),
            PhotoLibraryAuthorizationStatus::Limited
        );
    }

    #[test]
    fn maps_unknown_future_status_without_collapsing_limited() {
        assert_eq!(
            authorization_status_from_native(99),
            PhotoLibraryAuthorizationStatus::Unknown
        );
        assert_ne!(
            authorization_status_from_native(4),
            authorization_status_from_native(3)
        );
    }

    #[test]
    fn maps_add_only_status_and_fails_closed_for_limited_or_unknown() {
        assert_eq!(
            add_only_authorization_status_from_native(0),
            PhotoLibraryAddOnlyAuthorizationStatus::NotDetermined
        );
        assert_eq!(
            add_only_authorization_status_from_native(1),
            PhotoLibraryAddOnlyAuthorizationStatus::Restricted
        );
        assert_eq!(
            add_only_authorization_status_from_native(2),
            PhotoLibraryAddOnlyAuthorizationStatus::Denied
        );
        assert_eq!(
            add_only_authorization_status_from_native(3),
            PhotoLibraryAddOnlyAuthorizationStatus::Authorized
        );
        assert_eq!(
            add_only_authorization_status_from_native(4),
            PhotoLibraryAddOnlyAuthorizationStatus::Unknown
        );
        assert_eq!(
            add_only_authorization_status_from_native(99),
            PhotoLibraryAddOnlyAuthorizationStatus::Unknown
        );
    }
}
