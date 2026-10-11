use framework_calendar::{
    CalendarAuthorizationStatus, CalendarEvent, CalendarEventDeleteError, CalendarEventError,
    CalendarEventRange, CalendarEventTime, CalendarEventUpdateError,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode};

const MAX_EVENTKIT_QUERY_SECONDS: f64 = 1461.0 * 86_400.0;

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

/// Maps an EventKit mutation error to the selected-event update contract.
pub(crate) fn update_error_from_native(code: isize) -> CalendarEventUpdateError {
    match error_from_native(code) {
        framework_calendar::CalendarError::Backend(error) => {
            CalendarEventUpdateError::Backend(error)
        }
        _ => CalendarEventUpdateError::Backend(Error::new(ErrorKind::Platform)),
    }
}

/// Fails closed unless EventKit currently permits full event-data access.
pub(crate) fn require_full_access(
    status: CalendarAuthorizationStatus,
) -> Result<(), CalendarEventError> {
    if status == CalendarAuthorizationStatus::FullAccess {
        Ok(())
    } else {
        Err(CalendarEventError::NotFullAccess(status))
    }
}

/// Rejects ranges EventKit would silently shorten to its four-year query horizon.
pub(crate) fn validate_eventkit_range(range: CalendarEventRange) -> Result<(), CalendarEventError> {
    let seconds = range.end().unix_seconds() - range.start().unix_seconds();
    if !seconds.is_finite() || seconds > MAX_EVENTKIT_QUERY_SECONDS {
        return Err(CalendarEventError::RangeTooWide);
    }
    Ok(())
}

/// Rejects a stale or ambiguous selected occurrence before EventKit mutation.
pub(crate) fn validate_event_delete_snapshot(
    event: &CalendarEvent,
    resolved_identifier: &str,
    start_seconds: f64,
    end_seconds: f64,
) -> Result<(), CalendarEventDeleteError> {
    if !selected_event_snapshot_matches(event, resolved_identifier, start_seconds, end_seconds) {
        Err(CalendarEventDeleteError::EventNotFound)
    } else {
        Ok(())
    }
}

/// Rejects a stale or ambiguous selected occurrence before EventKit update.
pub(crate) fn validate_event_update_snapshot(
    event: &CalendarEvent,
    resolved_identifier: &str,
    start_seconds: f64,
    end_seconds: f64,
) -> Result<(), CalendarEventUpdateError> {
    if !selected_event_snapshot_matches(event, resolved_identifier, start_seconds, end_seconds) {
        Err(CalendarEventUpdateError::EventNotFound)
    } else {
        Ok(())
    }
}

fn selected_event_snapshot_matches(
    event: &CalendarEvent,
    resolved_identifier: &str,
    start_seconds: f64,
    end_seconds: f64,
) -> bool {
    event.event_identifier() == Some(resolved_identifier)
        && start_seconds == event.start().unix_seconds()
        && end_seconds == event.end().unix_seconds()
}

/// Copies decoded EventKit fields into a validated Rust-owned summary.
pub(crate) fn event_from_native_fields(
    event_identifier: Option<String>,
    title: String,
    start_seconds: f64,
    end_seconds: f64,
    all_day: bool,
    location: Option<String>,
    notes: Option<String>,
) -> Result<CalendarEvent, CalendarEventError> {
    let start = CalendarEventTime::from_unix_seconds(start_seconds)
        .map_err(|_| CalendarEventError::InvalidNativeEvent)?;
    let end = CalendarEventTime::from_unix_seconds(end_seconds)
        .map_err(|_| CalendarEventError::InvalidNativeEvent)?;
    CalendarEvent::new(
        event_identifier,
        title,
        start,
        end,
        all_day,
        location,
        notes,
    )
    .map_err(|_| CalendarEventError::InvalidNativeEvent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_calendar::{Calendar, CalendarEventBackend};
    use std::cell::Cell;
    use std::string::ToString;
    use std::vec::Vec;

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

        let update_error = update_error_from_native(-42);
        assert_eq!(update_error.kind(), ErrorKind::Platform);
        assert_eq!(update_error.platform_code().unwrap().get(), -42);

        let zero = error_from_native(0);
        assert_eq!(zero.kind(), ErrorKind::Platform);
        assert_eq!(zero.platform_code(), None);

        let zero_update = update_error_from_native(0);
        assert_eq!(zero_update.kind(), ErrorKind::Platform);
        assert_eq!(zero_update.platform_code(), None);
    }

    #[test]
    fn event_reads_require_full_access_and_reject_write_only() {
        assert_eq!(
            require_full_access(CalendarAuthorizationStatus::FullAccess),
            Ok(())
        );
        for status in [
            CalendarAuthorizationStatus::Unknown,
            CalendarAuthorizationStatus::NotDetermined,
            CalendarAuthorizationStatus::Restricted,
            CalendarAuthorizationStatus::Denied,
            CalendarAuthorizationStatus::WriteOnly,
        ] {
            assert_eq!(
                require_full_access(status),
                Err(CalendarEventError::NotFullAccess(status))
            );
        }
    }

    #[test]
    fn event_ranges_preserve_fractional_seconds_and_reject_eventkit_truncation() {
        let start = CalendarEventTime::from_unix_seconds(-0.125_000_25).unwrap();
        let end = CalendarEventTime::from_unix_seconds(1.125_000_75).unwrap();
        let range = CalendarEventRange::new(start, end).unwrap();
        assert_eq!(range.start().unix_seconds(), -0.125_000_25);
        assert_eq!(range.end().unix_seconds(), 1.125_000_75);
        assert_eq!(validate_eventkit_range(range), Ok(()));

        let too_wide = CalendarEventRange::new(
            CalendarEventTime::from_unix_seconds(0.0).unwrap(),
            CalendarEventTime::from_unix_seconds(MAX_EVENTKIT_QUERY_SECONDS + 0.5).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_eventkit_range(too_wide),
            Err(CalendarEventError::RangeTooWide)
        );
    }

    #[test]
    fn event_conversion_keeps_nullable_fields_and_all_day_instants() {
        let event = event_from_native_fields(
            None,
            "All day".to_string(),
            1_700_000_000.125,
            1_700_086_400.875,
            true,
            None,
            Some("private notes".to_string()),
        )
        .unwrap();
        assert_eq!(event.event_identifier(), None);
        assert_eq!(event.title(), "All day");
        assert!(event.is_all_day());
        assert_eq!(event.start().unix_seconds(), 1_700_000_000.125);
        assert_eq!(event.end().unix_seconds(), 1_700_086_400.875);
        assert_eq!(event.location(), None);
        assert_eq!(event.notes(), Some("private notes"));
    }

    #[test]
    fn malformed_native_event_dates_fail_instead_of_being_dropped_or_rewritten() {
        assert_eq!(
            event_from_native_fields(None, String::new(), f64::NAN, 1.0, false, None, None),
            Err(CalendarEventError::InvalidNativeEvent)
        );
        assert_eq!(
            event_from_native_fields(None, String::new(), 2.0, 1.0, false, None, None),
            Err(CalendarEventError::InvalidNativeEvent)
        );
    }

    #[test]
    fn recurring_first_occurrence_mismatch_fails_before_delete() {
        let selected = CalendarEvent::new(
            Some("recurring-event".to_string()),
            "Selected occurrence".to_string(),
            CalendarEventTime::from_unix_seconds(1_700_086_400.0).unwrap(),
            CalendarEventTime::from_unix_seconds(1_700_086_800.0).unwrap(),
            false,
            None,
            None,
        )
        .unwrap();
        let mut remove_calls = 0_u8;
        let result = validate_event_delete_snapshot(
            &selected,
            "recurring-event",
            1_700_000_000.0,
            1_700_000_400.0,
        )
        .map(|()| remove_calls += 1);

        assert_eq!(result, Err(CalendarEventDeleteError::EventNotFound));
        assert_eq!(remove_calls, 0);
    }

    #[test]
    fn recurring_first_occurrence_mismatch_fails_before_update() {
        let selected = CalendarEvent::new(
            Some("recurring-event".to_string()),
            "Selected occurrence".to_string(),
            CalendarEventTime::from_unix_seconds(1_700_086_400.0).unwrap(),
            CalendarEventTime::from_unix_seconds(1_700_086_800.0).unwrap(),
            false,
            None,
            None,
        )
        .unwrap();
        let mut save_calls = 0_u8;
        let result = validate_event_update_snapshot(
            &selected,
            "recurring-event",
            1_700_000_000.0,
            1_700_000_400.0,
        )
        .map(|()| save_calls += 1);

        assert_eq!(result, Err(CalendarEventUpdateError::EventNotFound));
        assert_eq!(save_calls, 0);
    }

    #[test]
    fn portable_event_range_rejects_non_finite_and_empty_bounds() {
        assert_eq!(
            CalendarEventTime::from_unix_seconds(f64::INFINITY),
            Err(CalendarEventError::InvalidTime)
        );
        let start = CalendarEventTime::from_unix_seconds(4.0).unwrap();
        assert_eq!(
            CalendarEventRange::new(start, start),
            Err(CalendarEventError::InvalidRange)
        );
        assert_eq!(
            CalendarEventRange::new(CalendarEventTime::from_unix_seconds(5.0).unwrap(), start,),
            Err(CalendarEventError::InvalidRange)
        );
    }

    struct FakeEventBackend {
        status: CalendarAuthorizationStatus,
        query_count: Cell<u8>,
    }

    impl CalendarEventBackend for FakeEventBackend {
        fn events_in_range(
            &self,
            _range: CalendarEventRange,
        ) -> Result<Vec<CalendarEvent>, CalendarEventError> {
            require_full_access(self.status)?;
            self.query_count.set(self.query_count.get() + 1);
            Ok(Vec::new())
        }
    }

    #[test]
    fn calendar_facade_event_query_fails_closed_before_backend_fetch() {
        let calendar = Calendar::new(FakeEventBackend {
            status: CalendarAuthorizationStatus::WriteOnly,
            query_count: Cell::new(0),
        });
        let range = CalendarEventRange::new(
            CalendarEventTime::from_unix_seconds(0.0).unwrap(),
            CalendarEventTime::from_unix_seconds(1.0).unwrap(),
        )
        .unwrap();

        assert_eq!(
            calendar.events_in_range(range),
            Err(CalendarEventError::NotFullAccess(
                CalendarAuthorizationStatus::WriteOnly
            ))
        );
        assert_eq!(calendar.backend().query_count.get(), 0);
    }
}
