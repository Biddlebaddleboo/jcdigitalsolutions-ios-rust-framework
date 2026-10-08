use core::time::Duration;
use framework_core::{AuthorizationState, Error, ErrorKind};
use framework_notifications::NotificationError;

/// Last UTC timestamp supported by the iOS calendar-trigger conversion (9999-12-31 23:59:59.999).
pub(crate) const MAX_IOS_UNIX_TIMESTAMP_MILLIS: u64 = 253_402_300_799_999;

/// Gregorian UTC components for one absolute, non-repeating notification trigger.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UtcDateComponents {
    pub(crate) year: i32,
    pub(crate) month: i32,
    pub(crate) day: i32,
    pub(crate) hour: i32,
    pub(crate) minute: i32,
    pub(crate) second: i32,
    pub(crate) nanosecond: i32,
}

/// Maps UserNotifications authorization raw values to the coarser portable state.
pub(crate) const fn authorization_state_from_native(value: isize) -> AuthorizationState {
    match value {
        0 => AuthorizationState::NotDetermined,
        1 => AuthorizationState::Denied,
        2..=4 => AuthorizationState::Authorized,
        _ => AuthorizationState::Unknown,
    }
}

/// Converts an optional Unix-millisecond timestamp into one absolute UTC trigger.
///
/// `None` means the portable trigger is immediate or already due. A date beyond the
/// backend's Gregorian year-9999 bound is rejected instead of being wrapped or truncated.
pub(crate) fn utc_trigger_components(
    timestamp_millis: Option<u64>,
    now: Duration,
) -> Result<Option<UtcDateComponents>, NotificationError> {
    let Some(timestamp_millis) = timestamp_millis else {
        return Ok(None);
    };
    if timestamp_millis > MAX_IOS_UNIX_TIMESTAMP_MILLIS {
        return Err(NotificationError::Backend(Error::new(
            ErrorKind::Unsupported,
        )));
    }
    if Duration::from_millis(timestamp_millis) <= now {
        return Ok(None);
    }
    let whole_days = timestamp_millis / 86_400_000;
    let day_millis = timestamp_millis % 86_400_000;
    let (year, month, day) = civil_from_days(whole_days as i64);
    Ok(Some(UtcDateComponents {
        year,
        month,
        day,
        hour: (day_millis / 3_600_000) as i32,
        minute: ((day_millis / 60_000) % 60) as i32,
        second: ((day_millis / 1_000) % 60) as i32,
        nanosecond: ((day_millis % 1_000) * 1_000_000) as i32,
    }))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, i32, i32) {
    let shifted_days = days_since_unix_epoch + 719_468;
    let era = shifted_days / 146_097;
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year as i32, month as i32, day as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_core::ErrorKind;

    #[test]
    fn authorization_statuses_normalize_without_claiming_restricted() {
        assert_eq!(
            authorization_state_from_native(0),
            AuthorizationState::NotDetermined
        );
        assert_eq!(
            authorization_state_from_native(1),
            AuthorizationState::Denied
        );
        for value in [2, 3, 4] {
            assert_eq!(
                authorization_state_from_native(value),
                AuthorizationState::Authorized
            );
        }
        assert_eq!(
            authorization_state_from_native(5),
            AuthorizationState::Unknown
        );
    }

    #[test]
    fn immediate_and_past_triggers_become_nil_native_triggers() {
        assert_eq!(utc_trigger_components(None, Duration::ZERO), Ok(None));
        assert_eq!(
            utc_trigger_components(Some(1_000), Duration::from_secs(2)),
            Ok(None)
        );
    }

    #[test]
    fn future_timestamp_preserves_utc_date_and_millisecond_component() {
        assert_eq!(
            utc_trigger_components(Some(1_700_000_000_123), Duration::ZERO),
            Ok(Some(UtcDateComponents {
                year: 2023,
                month: 11,
                day: 14,
                hour: 22,
                minute: 13,
                second: 20,
                nanosecond: 123_000_000,
            }))
        );
    }

    #[test]
    fn leap_day_and_supported_upper_bound_convert_without_overflow() {
        assert_eq!(
            utc_trigger_components(Some(1_709_210_096_789), Duration::ZERO),
            Ok(Some(UtcDateComponents {
                year: 2024,
                month: 2,
                day: 29,
                hour: 12,
                minute: 34,
                second: 56,
                nanosecond: 789_000_000,
            }))
        );
        let last = utc_trigger_components(Some(MAX_IOS_UNIX_TIMESTAMP_MILLIS), Duration::ZERO)
            .unwrap()
            .unwrap();
        assert_eq!((last.year, last.month, last.day), (9999, 12, 31));
        assert_eq!((last.hour, last.minute, last.second), (23, 59, 59));
        assert_eq!(last.nanosecond, 999_000_000);
    }

    #[test]
    fn timestamp_after_backend_range_returns_unsupported_without_truncation() {
        let error = utc_trigger_components(Some(MAX_IOS_UNIX_TIMESTAMP_MILLIS + 1), Duration::ZERO)
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported);
    }
}
