use crate::alert_state::IosAlertError;

pub(crate) const MAX_ALERT_TEXT_BYTES: usize = 4096;
pub(crate) const MAX_ALERT_ACTIONS: u32 = 32;
pub(crate) const MAX_ALERT_TEXT_FIELDS: u8 = 2;
pub(crate) const DEFAULT_TEXT_FIELD_MAX_UTF16_UNITS: u16 = 256;
pub(crate) const MAX_TEXT_FIELD_MAX_UTF16_UNITS: u16 = 4096;

pub(crate) fn valid_alert_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_ALERT_TEXT_BYTES
}

pub(crate) fn valid_optional_text(value: Option<&str>) -> bool {
    value.is_none_or(|text| text.len() <= MAX_ALERT_TEXT_BYTES)
}

pub(crate) fn next_action_id(action_count: u32) -> Result<u32, IosAlertError> {
    if action_count >= MAX_ALERT_ACTIONS {
        return Err(IosAlertError::ActionLimitReached);
    }
    Ok(action_count)
}

pub(crate) fn next_text_field_count(field_count: u8) -> Result<u8, IosAlertError> {
    if field_count >= MAX_ALERT_TEXT_FIELDS {
        return Err(IosAlertError::TextFieldLimitReached);
    }
    Ok(field_count + 1)
}

pub(crate) fn valid_text_field_limit(limit: u16) -> bool {
    (1..=MAX_TEXT_FIELD_MAX_UTF16_UNITS).contains(&limit)
}

pub(crate) fn valid_initial_text(value: Option<&str>, maximum_length: u16) -> bool {
    value.is_none_or(|text| text.encode_utf16().count() <= usize::from(maximum_length))
}

pub(crate) fn valid_rect_components(x: f64, y: f64, width: f64, height: f64) -> bool {
    let max_x = x + width;
    let max_y = y + height;
    x.is_finite()
        && y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width >= 0.0
        && height >= 0.0
        && max_x.is_finite()
        && max_y.is_finite()
}

pub(crate) fn allows_utf16_replacement(
    current_length: usize,
    range_start: usize,
    range_length: usize,
    replacement_length: usize,
    maximum_length: usize,
) -> bool {
    let Some(range_end) = range_start.checked_add(range_length) else {
        return false;
    };
    if range_end > current_length {
        return false;
    }
    current_length
        .checked_sub(range_length)
        .and_then(|length| length.checked_add(replacement_length))
        .is_some_and(|length| length <= maximum_length)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_TEXT_FIELD_MAX_UTF16_UNITS, MAX_ALERT_ACTIONS, MAX_ALERT_TEXT_BYTES,
        MAX_ALERT_TEXT_FIELDS, MAX_TEXT_FIELD_MAX_UTF16_UNITS, allows_utf16_replacement,
        next_action_id, next_text_field_count, valid_alert_text, valid_initial_text,
        valid_optional_text, valid_rect_components, valid_text_field_limit,
    };
    use crate::alert_state::IosAlertError;

    #[test]
    fn action_ids_follow_insertion_order_and_are_bounded() {
        assert_eq!(next_action_id(0), Ok(0));
        assert_eq!(next_action_id(1), Ok(1));
        assert_eq!(
            next_action_id(MAX_ALERT_ACTIONS - 1),
            Ok(MAX_ALERT_ACTIONS - 1)
        );
        assert_eq!(
            next_action_id(MAX_ALERT_ACTIONS),
            Err(IosAlertError::ActionLimitReached)
        );
    }

    #[test]
    fn text_field_count_and_utf16_limit_are_bounded() {
        assert_eq!(next_text_field_count(0), Ok(1));
        assert_eq!(
            next_text_field_count(MAX_ALERT_TEXT_FIELDS - 1),
            Ok(MAX_ALERT_TEXT_FIELDS)
        );
        assert_eq!(
            next_text_field_count(MAX_ALERT_TEXT_FIELDS),
            Err(IosAlertError::TextFieldLimitReached)
        );
        assert!(valid_text_field_limit(1));
        assert!(valid_text_field_limit(DEFAULT_TEXT_FIELD_MAX_UTF16_UNITS));
        assert!(valid_text_field_limit(MAX_TEXT_FIELD_MAX_UTF16_UNITS));
        assert!(!valid_text_field_limit(0));
        assert!(!valid_text_field_limit(MAX_TEXT_FIELD_MAX_UTF16_UNITS + 1));
    }

    #[test]
    fn alert_and_field_text_limits_are_explicit_and_length_aware() {
        assert!(valid_alert_text("Continue"));
        assert!(!valid_alert_text(""));
        assert!(valid_optional_text(None));
        assert!(valid_optional_text(Some("")));
        assert!(valid_optional_text(Some(&"x".repeat(MAX_ALERT_TEXT_BYTES))));
        assert!(!valid_optional_text(Some(
            &"x".repeat(MAX_ALERT_TEXT_BYTES + 1)
        )));
    }

    #[test]
    fn initial_text_and_popover_geometry_are_bounded() {
        assert!(valid_initial_text(None, 1));
        assert!(valid_initial_text(Some("a🚗"), 3));
        assert!(!valid_initial_text(Some("a🚗"), 2));
        assert!(valid_rect_components(0.0, 0.0, 0.0, 0.0));
        assert!(valid_rect_components(-4.0, 8.0, 2.0, 3.0));
        assert!(!valid_rect_components(f64::NAN, 0.0, 1.0, 1.0));
        assert!(!valid_rect_components(0.0, 0.0, -1.0, 1.0));
        assert!(!valid_rect_components(f64::MAX, 0.0, f64::MAX, 1.0));
    }

    #[test]
    fn utf16_edit_limit_handles_replacement_and_supplementary_characters() {
        assert!(allows_utf16_replacement(4, 1, 2, 2, 4));
        assert!(!allows_utf16_replacement(4, 1, 2, 3, 4));
        assert!(!allows_utf16_replacement(4, 5, 0, 0, 8));
        assert!(!allows_utf16_replacement(usize::MAX, 0, 0, 1, usize::MAX));
        // A supplementary Unicode scalar occupies two UTF-16 units.
        assert!(!allows_utf16_replacement(2, 0, 0, 2, 1));
    }
}
