use framework_contacts::{
    ContactDraft, ContactLabeledValue, ContactUpdate, ContactsAuthorization, ContactsError,
};
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

/// Rejects empty drafts and empty phone/email values before creating a native record.
pub(crate) fn validate_contact_draft(draft: &ContactDraft) -> Result<(), ContactsError> {
    if draft
        .phone_numbers
        .iter()
        .any(|value| value.value.is_empty())
        || draft
            .email_addresses
            .iter()
            .any(|value| value.value.is_empty())
        || (draft.given_name.is_empty()
            && draft.family_name.is_empty()
            && draft.organization_name.is_empty()
            && draft.phone_numbers.is_empty()
            && draft.email_addresses.is_empty())
    {
        Err(ContactsError::InvalidContactDraft)
    } else {
        Ok(())
    }
}

/// Rejects no-op patches and empty phone/email values before native work.
pub(crate) fn validate_contact_update(update: &ContactUpdate) -> Result<(), ContactsError> {
    if (update.given_name.is_none()
        && update.family_name.is_none()
        && update.organization_name.is_none()
        && update.phone_numbers.is_none()
        && update.email_addresses.is_none())
        || update
            .phone_numbers
            .as_ref()
            .is_some_and(|values| values.iter().any(|value| value.value.is_empty()))
        || update
            .email_addresses
            .as_ref()
            .is_some_and(|values| values.iter().any(|value| value.value.is_empty()))
    {
        Err(ContactsError::InvalidContactUpdate)
    } else {
        Ok(())
    }
}

/// Returns the exact optional label and value sent to `CNLabeledValue`.
pub(crate) fn contact_value_parts(value: &ContactLabeledValue) -> (Option<&str>, &str) {
    (value.label.as_deref(), value.value.as_str())
}

/// Requires an authorization state that permits add-only Contacts work.
pub(crate) const fn create_authorization_error(
    status: ContactsAuthorization,
) -> Option<ContactsError> {
    if status.is_full_access() || matches!(status, ContactsAuthorization::Limited) {
        None
    } else {
        Some(ContactsError::AuthorizationRequired(status))
    }
}

/// Maps a save failure after the backend re-queries current authorization.
pub(crate) fn create_error_from_native(
    status: ContactsAuthorization,
    native_error_code: isize,
) -> ContactsError {
    match create_authorization_error(status) {
        Some(error) => error,
        None => error_from_native(native_error_code),
    }
}

/// Maps a save failure after re-checking authorization and preserving Contacts not-found errors.
pub(crate) fn update_error_from_native(
    status: ContactsAuthorization,
    contacts_error_domain: bool,
    native_error_code: isize,
) -> ContactsError {
    if let Some(error) = fetch_authorization_error(status) {
        return error;
    }
    if contacts_error_domain && native_error_code == 200 {
        ContactsError::NotFound
    } else {
        error_from_native(native_error_code)
    }
}

/// Maps a delete failure after re-checking authorization and preserving missing records.
pub(crate) fn delete_error_from_native(
    status: ContactsAuthorization,
    contacts_error_domain: bool,
    native_error_code: isize,
) -> ContactsError {
    if let Some(error) = fetch_authorization_error(status) {
        return error;
    }
    if contacts_error_domain && native_error_code == 200 {
        ContactsError::NotFound
    } else {
        error_from_native(native_error_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_contacts::{Contact, ContactLabeledValue, ContactUpdate};
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

    #[test]
    fn create_draft_validation_rejects_empty_record_and_empty_labeled_values() {
        assert_eq!(
            validate_contact_draft(&ContactDraft::default()),
            Err(ContactsError::InvalidContactDraft)
        );
        let empty_phone = ContactDraft {
            given_name: String::from("Ada"),
            phone_numbers: vec![ContactLabeledValue {
                label: None,
                value: String::new(),
            }],
            ..ContactDraft::default()
        };
        assert_eq!(
            validate_contact_draft(&empty_phone),
            Err(ContactsError::InvalidContactDraft)
        );
        let empty_email = ContactDraft {
            email_addresses: vec![ContactLabeledValue {
                label: Some(String::from("work")),
                value: String::new(),
            }],
            ..ContactDraft::default()
        };
        assert_eq!(
            validate_contact_draft(&empty_email),
            Err(ContactsError::InvalidContactDraft)
        );
    }

    #[test]
    fn create_draft_validation_preserves_valid_unicode_values() {
        let draft = ContactDraft {
            given_name: String::from("山田 太郎"),
            phone_numbers: vec![ContactLabeledValue {
                label: Some(String::from("_$!<Mobile>!$_")),
                value: String::from("+81 90 1234 5678"),
            }],
            ..ContactDraft::default()
        };
        assert_eq!(validate_contact_draft(&draft), Ok(()));
        assert_eq!(
            contact_value_parts(&draft.phone_numbers[0]),
            (Some("_$!<Mobile>!$_"), "+81 90 1234 5678")
        );
        let unlabeled_email = ContactLabeledValue {
            label: None,
            value: String::from("zoe@example.test"),
        };
        assert_eq!(
            contact_value_parts(&unlabeled_email),
            (None, "zoe@example.test")
        );
    }

    #[test]
    fn contact_update_validation_preserves_none_and_explicit_clear_semantics() {
        assert_eq!(
            validate_contact_update(&ContactUpdate::default()),
            Err(ContactsError::InvalidContactUpdate)
        );

        let clear_name = ContactUpdate {
            given_name: Some(String::new()),
            ..ContactUpdate::default()
        };
        assert_eq!(validate_contact_update(&clear_name), Ok(()));

        let clear_phones = ContactUpdate {
            phone_numbers: Some(Vec::new()),
            ..ContactUpdate::default()
        };
        assert_eq!(validate_contact_update(&clear_phones), Ok(()));

        let clear_emails = ContactUpdate {
            email_addresses: Some(Vec::new()),
            ..ContactUpdate::default()
        };
        assert_eq!(validate_contact_update(&clear_emails), Ok(()));

        let replace_values = ContactUpdate {
            family_name: Some(String::from("Lovelace")),
            organization_name: Some(String::new()),
            phone_numbers: Some(vec![ContactLabeledValue {
                label: Some(String::from("mobile")),
                value: String::from("+1 555 0100"),
            }]),
            email_addresses: Some(vec![ContactLabeledValue {
                label: None,
                value: String::from("ada@example.test"),
            }]),
            ..ContactUpdate::default()
        };
        assert_eq!(validate_contact_update(&replace_values), Ok(()));
    }

    #[test]
    fn contact_update_validation_rejects_empty_phone_and_email_values() {
        let empty_phone = ContactUpdate {
            given_name: Some(String::from("Ada")),
            phone_numbers: Some(vec![ContactLabeledValue {
                label: None,
                value: String::new(),
            }]),
            ..ContactUpdate::default()
        };
        assert_eq!(
            validate_contact_update(&empty_phone),
            Err(ContactsError::InvalidContactUpdate)
        );

        let empty_email = ContactUpdate {
            email_addresses: Some(vec![ContactLabeledValue {
                label: Some(String::from("work")),
                value: String::new(),
            }]),
            ..ContactUpdate::default()
        };
        assert_eq!(
            validate_contact_update(&empty_email),
            Err(ContactsError::InvalidContactUpdate)
        );
    }

    #[test]
    fn create_authorization_accepts_only_full_or_limited_access() {
        for status in [
            ContactsAuthorization::Unknown,
            ContactsAuthorization::NotDetermined,
            ContactsAuthorization::Restricted,
            ContactsAuthorization::Denied,
        ] {
            assert_eq!(
                create_authorization_error(status),
                Some(ContactsError::AuthorizationRequired(status))
            );
        }
        assert_eq!(
            create_authorization_error(ContactsAuthorization::Authorized),
            None
        );
        assert_eq!(
            create_authorization_error(ContactsAuthorization::Limited),
            None
        );
    }

    #[test]
    fn create_error_rechecks_authorization_and_preserves_native_code() {
        assert_eq!(
            create_error_from_native(ContactsAuthorization::Denied, -42),
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied)
        );
        let error = create_error_from_native(ContactsAuthorization::Authorized, -42);
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
    }

    #[test]
    fn update_error_maps_missing_contact_and_preserves_authorization_and_native_code() {
        assert_eq!(
            update_error_from_native(ContactsAuthorization::Limited, true, 200),
            ContactsError::NotFound
        );
        assert_eq!(
            update_error_from_native(ContactsAuthorization::Denied, true, 200),
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied)
        );
        let unrelated = update_error_from_native(ContactsAuthorization::Authorized, false, 200);
        assert_eq!(unrelated.kind(), ErrorKind::Platform);
        assert_eq!(unrelated.platform_code().unwrap().get(), 200);
        let native = update_error_from_native(ContactsAuthorization::Limited, true, -42);
        assert_eq!(native.kind(), ErrorKind::Platform);
        assert_eq!(native.platform_code().unwrap().get(), -42);
    }

    #[test]
    fn delete_error_maps_missing_contact_and_preserves_authorization_and_native_code() {
        assert_eq!(
            delete_error_from_native(ContactsAuthorization::Limited, true, 200),
            ContactsError::NotFound
        );
        assert_eq!(
            delete_error_from_native(ContactsAuthorization::Denied, true, 200),
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied)
        );
        let unrelated = delete_error_from_native(ContactsAuthorization::Authorized, false, 200);
        assert_eq!(unrelated.kind(), ErrorKind::Platform);
        assert_eq!(unrelated.platform_code().unwrap().get(), 200);
        let native = delete_error_from_native(ContactsAuthorization::Limited, true, -42);
        assert_eq!(native.kind(), ErrorKind::Platform);
        assert_eq!(native.platform_code().unwrap().get(), -42);
    }
}
