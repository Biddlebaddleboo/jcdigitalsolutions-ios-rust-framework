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

/// Rejects identifiers that cannot name a local Contacts record.
pub(crate) fn validate_identifier(identifier: &str) -> Result<(), ContactsError> {
    if identifier.is_empty() || identifier.as_bytes().contains(&0) {
        Err(ContactsError::InvalidIdentifier)
    } else {
        Ok(())
    }
}

/// Rejects a name query that cannot safely become a native Contacts string.
pub(crate) fn validate_search_query(query: &str) -> Result<(), ContactsError> {
    if query.is_empty() || query.as_bytes().contains(&0) {
        Err(ContactsError::InvalidSearchQuery)
    } else {
        Ok(())
    }
}

/// Reports whether a native callback has reached the caller's result cap.
pub(crate) const fn search_limit_reached(result_count: usize, max_results: u32) -> bool {
    result_count >= max_results as usize
}

/// Adds one snapshot only while the caller's search limit leaves room.
pub(crate) fn append_search_result<T>(
    results: &mut Vec<T>,
    max_results: u32,
    make_snapshot: impl FnOnce() -> T,
) -> bool {
    if search_limit_reached(results.len(), max_results) {
        true
    } else {
        results.push(make_snapshot());
        search_limit_reached(results.len(), max_results)
    }
}

/// Returns a semantic access error unless Contacts data access is currently permitted.
pub(crate) const fn fetch_authorization_error(
    status: ContactsAuthorization,
) -> Option<ContactsError> {
    match status {
        ContactsAuthorization::Authorized | ContactsAuthorization::Limited => None,
        status => Some(ContactsError::AuthorizationRequired(status)),
    }
}

/// Maps a native one-record fetch failure without exposing Contacts framework types portably.
pub(crate) fn fetch_error_from_native(
    status: ContactsAuthorization,
    contacts_error_domain: bool,
    code: isize,
) -> ContactsError {
    if let Some(error) = fetch_authorization_error(status) {
        return error;
    }
    if contacts_error_domain && code == 200 {
        ContactsError::NotFound
    } else {
        error_from_native(code)
    }
}

/// Maps failed enumeration after re-checking authorization; an empty match set is not an error.
pub(crate) fn search_error_from_native(
    status: ContactsAuthorization,
    native_error_code: Option<isize>,
) -> ContactsError {
    if let Some(error) = fetch_authorization_error(status) {
        return error;
    }
    error_from_native(native_error_code.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_contacts::{Contact, ContactLabeledValue};
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

    #[test]
    fn fetch_authorization_fails_closed_except_authorized_and_limited() {
        for status in [
            ContactsAuthorization::Unknown,
            ContactsAuthorization::NotDetermined,
            ContactsAuthorization::Restricted,
            ContactsAuthorization::Denied,
        ] {
            assert_eq!(
                fetch_authorization_error(status),
                Some(ContactsError::AuthorizationRequired(status))
            );
        }
        assert_eq!(
            fetch_authorization_error(ContactsAuthorization::Authorized),
            None
        );
        assert_eq!(
            fetch_authorization_error(ContactsAuthorization::Limited),
            None
        );
    }

    #[test]
    fn fetch_identifier_rejects_empty_and_nul_values() {
        assert_eq!(
            validate_identifier(""),
            Err(ContactsError::InvalidIdentifier)
        );
        assert_eq!(
            validate_identifier("local\0id"),
            Err(ContactsError::InvalidIdentifier)
        );
        assert_eq!(validate_identifier("local-id"), Ok(()));
    }

    #[test]
    fn search_query_rejects_empty_and_nul_but_preserves_valid_input() {
        assert_eq!(
            validate_search_query(""),
            Err(ContactsError::InvalidSearchQuery)
        );
        assert_eq!(
            validate_search_query("Ada\0Lovelace"),
            Err(ContactsError::InvalidSearchQuery)
        );
        assert_eq!(validate_search_query("  Ada Lovelace  "), Ok(()));
        assert_eq!(validate_search_query("山田 太郎"), Ok(()));
        assert_eq!(
            ContactsError::InvalidSearchQuery.kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(ContactsError::InvalidSearchQuery.platform_code(), None);
    }

    #[test]
    fn search_limit_stops_at_and_after_the_exact_caller_cap() {
        assert!(search_limit_reached(0, 0));
        assert!(!search_limit_reached(0, 1));
        assert!(!search_limit_reached(1, 2));
        assert!(search_limit_reached(2, 2));
        assert!(search_limit_reached(3, 2));
    }

    #[test]
    fn search_result_cap_does_not_create_snapshots_past_limit() {
        let mut results = Vec::new();
        let mut snapshots_created = 0;
        assert!(append_search_result(&mut results, 1, || {
            snapshots_created += 1;
            "contact"
        }));
        assert_eq!(snapshots_created, 1);
        assert!(append_search_result(&mut results, 1, || {
            snapshots_created += 1;
            "extra contact"
        }));
        assert_eq!(snapshots_created, 1);
        assert_eq!(results, ["contact"]);

        let mut empty = Vec::new();
        assert!(append_search_result(&mut empty, 0, || panic!(
            "zero limit created a snapshot"
        )));
        assert!(empty.is_empty());
    }

    #[test]
    fn search_native_failure_prefers_revoked_authorization() {
        assert_eq!(
            search_error_from_native(ContactsAuthorization::Denied, Some(-42)),
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied)
        );
        let error = search_error_from_native(ContactsAuthorization::Limited, Some(-42));
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
        let no_code = search_error_from_native(ContactsAuthorization::Authorized, None);
        assert_eq!(no_code.kind(), ErrorKind::Platform);
        assert_eq!(no_code.platform_code(), None);
    }

    #[test]
    fn fetch_errors_distinguish_contacts_not_found_from_native_failures() {
        assert_eq!(
            fetch_error_from_native(ContactsAuthorization::Limited, true, 200),
            ContactsError::NotFound
        );
        let unrelated = fetch_error_from_native(ContactsAuthorization::Authorized, false, 200);
        assert_eq!(unrelated.kind(), ErrorKind::Platform);
        assert_eq!(unrelated.platform_code().unwrap().get(), 200);
        let denied = fetch_error_from_native(ContactsAuthorization::Denied, true, 200);
        assert_eq!(
            denied,
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied)
        );
    }

    #[test]
    fn owned_contact_values_preserve_common_fields_and_native_order() {
        let contact = Contact {
            identifier: String::from("local-id"),
            given_name: String::from("Ada"),
            family_name: String::from("Lovelace"),
            organization_name: String::from("Analytical Engines"),
            phone_numbers: vec![
                ContactLabeledValue {
                    label: Some(String::from("mobile")),
                    value: String::from("+1 555 0100"),
                },
                ContactLabeledValue {
                    label: None,
                    value: String::from("+1 555 0101"),
                },
            ],
            email_addresses: vec![ContactLabeledValue {
                label: Some(String::from("work")),
                value: String::from("ada@example.test"),
            }],
        };

        assert_eq!(contact.identifier, "local-id");
        assert_eq!(contact.given_name, "Ada");
        assert_eq!(contact.family_name, "Lovelace");
        assert_eq!(contact.organization_name, "Analytical Engines");
        assert_eq!(contact.phone_numbers[0].value, "+1 555 0100");
        assert_eq!(contact.phone_numbers[1].value, "+1 555 0101");
        assert_eq!(contact.email_addresses[0].label.as_deref(), Some("work"));
        assert_eq!(contact.email_addresses[0].value, "ada@example.test");
    }
}
