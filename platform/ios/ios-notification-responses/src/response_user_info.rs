use core::char;
use std::string::String;
use std::vec::Vec;

pub(super) fn decode_utf16<I>(units: I) -> Option<String>
where
    I: IntoIterator<Item = u16>,
{
    char::decode_utf16(units)
        .collect::<Result<String, _>>()
        .ok()
}

pub(super) fn sorted_string_entries<I>(entries: I) -> Vec<(String, String)>
where
    I: IntoIterator<Item = (Option<String>, Option<String>)>,
{
    let mut entries: Vec<_> = entries
        .into_iter()
        .filter_map(|(key, value)| Some((key?, value?)))
        .collect();
    entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_decode_preserves_valid_unicode_without_normalization() {
        let decomposed = decode_utf16("cafe\u{301}🛰️".encode_utf16()).unwrap();
        assert_eq!(decomposed, "cafe\u{301}🛰️");
        assert_ne!(decomposed, "café🛰️");
    }

    #[test]
    fn utf16_decode_rejects_unpaired_surrogates() {
        assert_eq!(decode_utf16([0x0061, 0xD800]), None);
        assert_eq!(decode_utf16([0xDC00]), None);
    }

    #[test]
    fn native_entry_adapter_omits_non_string_or_malformed_fields_and_sorts_keys() {
        let entries = sorted_string_entries([
            (Some(String::from("z")), Some(String::from("last"))),
            (Some(String::from("number")), None),
            (None, Some(String::from("non-string key"))),
            (Some(String::from("empty")), Some(String::new())),
            (Some(String::from("bad-utf16-value")), None),
        ]);

        assert_eq!(
            entries,
            [
                (String::from("empty"), String::new()),
                (String::from("z"), String::from("last")),
            ]
        );
    }
}
