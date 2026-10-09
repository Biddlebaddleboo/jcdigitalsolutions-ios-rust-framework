use framework_contacts::{ContactsAuthorization, ContactsError};
use framework_core::{Error, ErrorKind, PlatformErrorCode};

/// Maps native Contacts authorization raw values to the portable status without merging limited access.
pub(crate) const fn authorization_from_native(value: isize) -> ContactsAuthorization {
    match value {
        0 => ContactsAuthorization::NotDetermined,
        1 => ContactsAuthorization::Restricted,
        2 => ContactsAuthorization::Denied,
        3 => ContactsAuthorization::Authorized,
        4 => ContactsAuthorization::Limited,
        _ => ContactsAuthorization::Unknown,
    }
}

/// Uses the re-queried authorization state as the request result, mapping only an otherwise
/// unclassified native failure to an operation error.
pub(crate) fn request_result_from_native(
    status: isize,
    native_error_code: Option<isize>,
) -> Result<ContactsAuthorization, ContactsError> {
    let authorization = authorization_from_native(status);
    if matches!(
        authorization,
        ContactsAuthorization::Unknown | ContactsAuthorization::NotDetermined
    ) {
        if let Some(code) = native_error_code {
            return Err(error_from_native(code));
        }
    }
    Ok(authorization)
}

/// Maps an unexpected Contacts error while preserving a representable nonzero native code.
pub(crate) fn error_from_native(code: isize) -> ContactsError {
    let mut error = Error::new(ErrorKind::Platform);
    if let Ok(code) = i32::try_from(code) {
        if let Some(code) = PlatformErrorCode::new(code) {
            error = error.with_platform_code(code);
        }
    }
    ContactsError::Backend(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_core::ErrorKind;

    #[test]
    fn native_authorization_statuses_preserve_limited_access() {
        assert_eq!(
            authorization_from_native(0),
            ContactsAuthorization::NotDetermined
        );
        assert_eq!(
            authorization_from_native(1),
            ContactsAuthorization::Restricted
        );
        assert_eq!(authorization_from_native(2), ContactsAuthorization::Denied);
        assert_eq!(
            authorization_from_native(3),
            ContactsAuthorization::Authorized
        );
        assert_eq!(authorization_from_native(4), ContactsAuthorization::Limited);
        assert_eq!(authorization_from_native(5), ContactsAuthorization::Unknown);
    }

    #[test]
    fn re_queried_permission_state_wins_over_request_callback_error() {
        assert_eq!(
            request_result_from_native(2, Some(100)),
            Ok(ContactsAuthorization::Denied)
        );
        assert_eq!(
            request_result_from_native(3, Some(100)),
            Ok(ContactsAuthorization::Authorized)
        );
        assert_eq!(
            request_result_from_native(4, Some(100)),
            Ok(ContactsAuthorization::Limited)
        );
    }

    #[test]
    fn unexpected_native_error_keeps_category_and_code() {
        let error = request_result_from_native(0, Some(-42)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
        let zero = error_from_native(0);
        assert_eq!(zero.kind(), ErrorKind::Platform);
        assert_eq!(zero.platform_code(), None);
        assert_eq!(
            request_result_from_native(5, None),
            Ok(ContactsAuthorization::Unknown)
        );
    }
}
