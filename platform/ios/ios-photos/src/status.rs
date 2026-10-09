use framework_photos::PhotoLibraryAuthorizationStatus;

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
}
