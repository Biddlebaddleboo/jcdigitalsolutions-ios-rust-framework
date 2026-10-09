use framework_calendar::CalendarAuthorizationStatus;
use framework_core::{Error, ErrorKind, PlatformErrorCode};

/// Maps an EventKit authorization-status raw value to the portable Calendar status.
pub(crate) const fn authorization_from_native(value: isize) -> CalendarAuthorizationStatus {
    match value {
        0 => CalendarAuthorizationStatus::NotDetermined,
        1 => CalendarAuthorizationStatus::Restricted,
        2 => CalendarAuthorizationStatus::Denied,
        3 => CalendarAuthorizationStatus::FullAccess,
        4 => CalendarAuthorizationStatus::WriteOnly,
        _ => CalendarAuthorizationStatus::Unknown,
    }
}

/// Maps an NSError code from the EventKit access callback to a portable backend error.
pub(crate) fn error_from_native(code: isize) -> framework_calendar::CalendarError {
    let mut error = Error::new(ErrorKind::Platform);
    if let Ok(code) = i32::try_from(code)
        && let Some(code) = PlatformErrorCode::new(code)
    {
        error = error.with_platform_code(code);
    }
    framework_calendar::CalendarError::Backend(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_authorization_statuses_keep_write_only_separate() {
        assert_eq!(
            authorization_from_native(0),
            CalendarAuthorizationStatus::NotDetermined
        );
        assert_eq!(
            authorization_from_native(1),
            CalendarAuthorizationStatus::Restricted
        );
        assert_eq!(
            authorization_from_native(2),
            CalendarAuthorizationStatus::Denied
        );
        assert_eq!(
            authorization_from_native(3),
            CalendarAuthorizationStatus::FullAccess
        );
        assert_eq!(
            authorization_from_native(4),
            CalendarAuthorizationStatus::WriteOnly
        );
        assert_eq!(
            authorization_from_native(5),
            CalendarAuthorizationStatus::Unknown
        );
    }

    #[test]
    fn native_errors_keep_platform_kind_and_representable_code() {
        let error = error_from_native(-42);
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);

        let zero = error_from_native(0);
        assert_eq!(zero.kind(), ErrorKind::Platform);
        assert_eq!(zero.platform_code(), None);
    }
}
