#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackgroundModeValue {
    Location,
    Other,
}

pub(crate) fn has_location_background_mode<I>(entries: Option<I>) -> bool
where
    I: IntoIterator<Item = BackgroundModeValue>,
{
    entries.is_some_and(|entries| {
        entries
            .into_iter()
            .any(|entry| entry == BackgroundModeValue::Location)
    })
}

pub(crate) const fn has_always_usage_descriptions(
    when_in_use: bool,
    always_and_when_in_use: bool,
    legacy_always: bool,
    requires_legacy_always: bool,
) -> bool {
    when_in_use && always_and_when_in_use && (!requires_legacy_always || legacy_always)
}

pub(crate) const fn is_nonempty_string_value(is_string: bool, length: usize) -> bool {
    is_string && length != 0
}

pub(crate) fn requires_legacy_always_usage_description(minimum_ios_version: Option<f64>) -> bool {
    minimum_ios_version.is_none_or(|version| !version.is_finite() || version < 11.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_background_mode_requires_an_array_with_exact_location_entry() {
        assert!(!has_location_background_mode(
            None::<[BackgroundModeValue; 0]>
        ));
        assert!(!has_location_background_mode(Some([])));
        assert!(!has_location_background_mode(Some([
            BackgroundModeValue::Other
        ])));
        assert!(has_location_background_mode(Some([
            BackgroundModeValue::Other,
            BackgroundModeValue::Location,
        ])));
    }

    #[test]
    fn background_plist_requires_always_usage_keys() {
        assert!(has_always_usage_descriptions(true, true, false, false));
        assert!(!has_always_usage_descriptions(false, true, true, false));
        assert!(!has_always_usage_descriptions(true, false, true, false));
        assert!(has_always_usage_descriptions(true, true, true, true));
        assert!(!has_always_usage_descriptions(true, true, false, true));
    }

    #[test]
    fn legacy_always_key_requirement_tracks_deployment_target() {
        assert!(requires_legacy_always_usage_description(None));
        assert!(requires_legacy_always_usage_description(Some(10.0)));
        assert!(requires_legacy_always_usage_description(Some(10.9)));
        assert!(requires_legacy_always_usage_description(Some(f64::NAN)));
        assert!(!requires_legacy_always_usage_description(Some(11.0)));
        assert!(!requires_legacy_always_usage_description(Some(17.0)));
    }

    #[test]
    fn usage_description_values_must_be_nonempty_strings() {
        assert!(is_nonempty_string_value(true, 1));
        assert!(!is_nonempty_string_value(true, 0));
        assert!(!is_nonempty_string_value(false, 12));
        assert!(!is_nonempty_string_value(false, 0));
    }
}
