/// An iOS native preference existed but did not have the requested property-list type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum IosPreferencesValueError {
    /// The stored native value was not the exact requested Foundation type.
    WrongType,
    /// The stored native string contained an unpaired UTF-16 surrogate.
    InvalidUtf16,
    /// Date seconds were not finite or the stored native date returned non-finite seconds.
    InvalidDateSeconds,
}

pub(crate) fn map_optional_value<T, U>(
    value: Option<T>,
    map: impl FnOnce(T) -> Result<U, IosPreferencesValueError>,
) -> Result<Option<U>, IosPreferencesValueError> {
    value.map(map).transpose()
}

pub(crate) fn decode_utf16(code_units: &[u16]) -> Result<String, IosPreferencesValueError> {
    String::from_utf16(code_units).map_err(|_| IosPreferencesValueError::InvalidUtf16)
}

pub(crate) fn validate_date_seconds(seconds: f64) -> Result<f64, IosPreferencesValueError> {
    if seconds.is_finite() {
        Ok(seconds)
    } else {
        Err(IosPreferencesValueError::InvalidDateSeconds)
    }
}
