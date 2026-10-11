#[path = "../src/value_policy.rs"]
mod value_policy;

use value_policy::{
    IosPreferencesValueError, decode_utf16, map_optional_value, validate_date_seconds,
};

#[test]
fn missing_value_remains_distinct_from_type_mismatch() {
    let value = map_optional_value(None::<u8>, |_| Ok(String::from("unused")));
    assert_eq!(value, Ok(None));

    let error =
        map_optional_value::<u8, ()>(Some(1_u8), |_| Err(IosPreferencesValueError::WrongType))
            .unwrap_err();
    assert_eq!(error, IosPreferencesValueError::WrongType);
}

#[test]
fn matching_value_is_converted() {
    let value = map_optional_value(Some(true), Ok).unwrap();
    assert_eq!(value, Some(true));
}

#[test]
fn non_boolean_number_is_rejected_instead_of_reported_missing() {
    enum StoredNumber {
        Boolean(bool),
        Numeric(i64),
    }

    let to_bool = |number| match number {
        StoredNumber::Boolean(value) => Ok(value),
        StoredNumber::Numeric(value) => {
            assert_eq!(value, 1);
            Err(IosPreferencesValueError::WrongType)
        }
    };

    assert_eq!(
        map_optional_value(Some(StoredNumber::Boolean(true)), to_bool),
        Ok(Some(true))
    );
    assert_eq!(
        map_optional_value(Some(StoredNumber::Numeric(1)), to_bool),
        Err(IosPreferencesValueError::WrongType)
    );
}

#[test]
fn valid_utf16_preserves_embedded_nul_and_surrogate_pairs() {
    assert_eq!(
        decode_utf16(&[b'a' as u16, 0, b'b' as u16]),
        Ok(String::from("a\0b"))
    );
    assert_eq!(decode_utf16(&[0xD83D, 0xDE00]), Ok(String::from("😀")));
}

#[test]
fn unpaired_utf16_surrogate_is_reported_without_lossy_replacement() {
    assert_eq!(
        decode_utf16(&[0xD800]),
        Err(IosPreferencesValueError::InvalidUtf16)
    );
}

#[test]
fn native_date_type_is_accepted_and_numeric_value_is_rejected() {
    enum NativeValue {
        Date(f64),
        Number(f64),
    }

    let to_seconds = |value| match value {
        NativeValue::Date(seconds) => validate_date_seconds(seconds),
        NativeValue::Number(seconds) => {
            assert!(seconds.is_finite());
            Err(IosPreferencesValueError::WrongType)
        }
    };
    let seconds = 1_700_000_000.125;

    assert_eq!(
        map_optional_value(Some(NativeValue::Date(seconds)), to_seconds),
        Ok(Some(seconds))
    );
    let result = map_optional_value(Some(NativeValue::Number(seconds)), to_seconds);

    assert_eq!(result, Err(IosPreferencesValueError::WrongType));
}

#[test]
fn date_seconds_keep_fractional_seconds_and_reject_non_finite_values() {
    let seconds = 1_700_000_000.125;
    assert_eq!(validate_date_seconds(seconds), Ok(seconds));
    assert_eq!(
        validate_date_seconds(f64::NAN),
        Err(IosPreferencesValueError::InvalidDateSeconds)
    );
    assert_eq!(
        validate_date_seconds(f64::INFINITY),
        Err(IosPreferencesValueError::InvalidDateSeconds)
    );
}
