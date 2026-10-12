#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Utf16Error {
    Invalid,
    ResourceExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccessibilityText<'a> {
    Clear,
    Set(&'a str),
}

impl<'a> AccessibilityText<'a> {
    pub(crate) const fn from_optional(value: Option<&'a str>) -> Self {
        match value {
            Some(value) => Self::Set(value),
            None => Self::Clear,
        }
    }

    pub(crate) const fn as_optional(self) -> Option<&'a str> {
        match self {
            Self::Clear => None,
            Self::Set(value) => Some(value),
        }
    }
}

pub(crate) fn decode_utf16(units: &[u16]) -> Result<String, Utf16Error> {
    let mut byte_len = 0_usize;
    for decoded in char::decode_utf16(units.iter().copied()) {
        let character = decoded.map_err(|_| Utf16Error::Invalid)?;
        byte_len = byte_len
            .checked_add(character.len_utf8())
            .ok_or(Utf16Error::ResourceExhausted)?;
    }

    let mut text = String::new();
    text.try_reserve_exact(byte_len)
        .map_err(|_| Utf16Error::ResourceExhausted)?;
    for decoded in char::decode_utf16(units.iter().copied()) {
        text.push(decoded.map_err(|_| Utf16Error::Invalid)?);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::{AccessibilityText, Utf16Error, decode_utf16};

    #[test]
    fn accessibility_metadata_distinguishes_cleared_and_empty_values() {
        assert_eq!(AccessibilityText::from_optional(None).as_optional(), None);
        assert_eq!(
            AccessibilityText::from_optional(Some("")).as_optional(),
            Some("")
        );
    }

    #[test]
    fn accessibility_metadata_preserves_embedded_nul_and_unicode() {
        let value = "copy\0 🚗";
        assert_eq!(
            AccessibilityText::from_optional(Some(value)).as_optional(),
            Some(value)
        );
    }

    #[test]
    fn decodes_empty_and_embedded_nul() {
        assert_eq!(decode_utf16(&[]).as_deref(), Ok(""));
        assert_eq!(decode_utf16(&[0, 0x00e9]).as_deref(), Ok("\0é"));
    }

    #[test]
    fn decodes_supplementary_scalar() {
        assert_eq!(decode_utf16(&[0xd83d, 0xde00]).as_deref(), Ok("😀"));
    }

    #[test]
    fn rejects_unpaired_surrogates_without_replacement() {
        assert_eq!(decode_utf16(&[0xd800]), Err(Utf16Error::Invalid));
        assert_eq!(decode_utf16(&[0xdc00]), Err(Utf16Error::Invalid));
        assert_eq!(
            decode_utf16(&[0xd800, b'a' as u16]),
            Err(Utf16Error::Invalid)
        );
    }

    #[test]
    fn selectable_text_snapshot_is_length_aware_and_strict() {
        assert_eq!(
            decode_utf16(&[0, b'a' as u16, 0xd83d, 0xde00]).as_deref(),
            Ok("\0a😀")
        );
        assert_eq!(
            decode_utf16(&[0, b'a' as u16, 0xd800]),
            Err(Utf16Error::Invalid)
        );
    }
}
