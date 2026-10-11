#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Utf16Error {
    Invalid,
    ResourceExhausted,
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
    use super::{Utf16Error, decode_utf16};

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
}
