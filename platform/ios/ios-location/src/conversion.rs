use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_location::{
    AccuracyTarget, Coordinate, LocationAuthorization, LocationError, LocationFix,
    LocationTimestamp,
};

/// Maps Core Location authorization raw values to the portable foreground/background state.
pub(crate) const fn authorization_from_native(value: i32) -> LocationAuthorization {
    match value {
        0 => LocationAuthorization::NotDetermined,
        1 => LocationAuthorization::Restricted,
        2 => LocationAuthorization::Denied,
        3 => LocationAuthorization::Background,
        4 => LocationAuthorization::Foreground,
        _ => LocationAuthorization::Unknown,
    }
}

/// A Core Location accuracy choice after translating the portable meter target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativeAccuracy {
    /// Ask Core Location for its best available accuracy constant.
    Best,
    /// Preserve a positive caller target as native meters.
    Meters(f64),
}

/// Maps the portable target to Core Location's best constant or an explicit meter value.
pub(crate) fn native_accuracy(target: AccuracyTarget) -> NativeAccuracy {
    if target.meters() == 0.0 {
        NativeAccuracy::Best
    } else {
        NativeAccuracy::Meters(target.meters())
    }
}

/// Maps the terminal error returned by a Core Location one-shot request.
pub(crate) fn error_from_native(code: i64) -> LocationError {
    let kind = match code {
        0 => ErrorKind::Unavailable,
        1 => ErrorKind::PermissionDenied,
        2 => ErrorKind::Unavailable,
        _ => ErrorKind::Platform,
    };
    let mut error = Error::new(kind);
    if let Ok(code) = i32::try_from(code) {
        if let Some(code) = PlatformErrorCode::new(code) {
            error = error.with_platform_code(code);
        }
    }
    LocationError::Backend(error)
}

/// Converts native coordinate, accuracy, and Unix-seconds values without clamping or refreshing.
pub(crate) fn fix_from_native(
    latitude_degrees: f64,
    longitude_degrees: f64,
    horizontal_accuracy_meters: f64,
    timestamp_unix_seconds: f64,
) -> Result<LocationFix, LocationError> {
    let coordinate = Coordinate::new(latitude_degrees, longitude_degrees)?;
    if !horizontal_accuracy_meters.is_finite() || horizontal_accuracy_meters < 0.0 {
        return Err(LocationError::InvalidAccuracy);
    }
    let timestamp_millis = timestamp_unix_seconds * 1_000.0;
    if !timestamp_millis.is_finite()
        || timestamp_millis < 0.0
        || timestamp_millis >= u64::MAX as f64
    {
        return Err(LocationError::Backend(Error::new(ErrorKind::Unsupported)));
    }
    LocationFix::new(
        coordinate,
        horizontal_accuracy_meters,
        LocationTimestamp::from_unix_millis(timestamp_millis as u64),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_statuses_preserve_foreground_and_background_levels() {
        assert_eq!(
            authorization_from_native(0),
            LocationAuthorization::NotDetermined
        );
        assert_eq!(
            authorization_from_native(1),
            LocationAuthorization::Restricted
        );
        assert_eq!(authorization_from_native(2), LocationAuthorization::Denied);
        assert_eq!(
            authorization_from_native(3),
            LocationAuthorization::Background
        );
        assert_eq!(
            authorization_from_native(4),
            LocationAuthorization::Foreground
        );
        assert_eq!(authorization_from_native(5), LocationAuthorization::Unknown);
    }

    #[test]
    fn accuracy_targets_keep_meters_and_zero_means_best_available() {
        assert_eq!(
            native_accuracy(AccuracyTarget::new(0.0).unwrap()),
            NativeAccuracy::Best
        );
        assert_eq!(
            native_accuracy(AccuracyTarget::new(0.5).unwrap()),
            NativeAccuracy::Meters(0.5)
        );
        assert_eq!(
            native_accuracy(AccuracyTarget::new(250.0).unwrap()),
            NativeAccuracy::Meters(250.0)
        );
    }

    #[test]
    fn native_errors_keep_portable_categories_and_nonzero_native_codes() {
        let unknown_location = error_from_native(0);
        assert_eq!(unknown_location.kind(), ErrorKind::Unavailable);
        assert_eq!(unknown_location.platform_code(), None);

        let denied = error_from_native(1);
        assert_eq!(denied.kind(), ErrorKind::PermissionDenied);
        assert_eq!(denied.platform_code().unwrap().get(), 1);

        let network = error_from_native(2);
        assert_eq!(network.kind(), ErrorKind::Unavailable);
        assert_eq!(network.platform_code().unwrap().get(), 2);

        let future_native_code = error_from_native(23);
        assert_eq!(future_native_code.kind(), ErrorKind::Platform);
        assert_eq!(future_native_code.platform_code().unwrap().get(), 23);
    }

    #[test]
    fn native_fix_preserves_values_and_rejects_unrepresentable_timestamp() {
        let fix = fix_from_native(43.2, -79.4, 12.5, 1_700_000_000.123).unwrap();
        assert_eq!(fix.coordinate().latitude_degrees(), 43.2);
        assert_eq!(fix.coordinate().longitude_degrees(), -79.4);
        assert_eq!(fix.horizontal_accuracy_meters(), 12.5);
        assert_eq!(fix.timestamp().unix_millis(), 1_700_000_000_123);
        assert_eq!(
            fix_from_native(0.0, 0.0, 1.0, f64::NAN).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
    }
}
