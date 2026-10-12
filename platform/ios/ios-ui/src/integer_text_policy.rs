use framework_core::{Error, ErrorKind, Result};

pub(crate) fn parse_optional_integer_text(text: Option<&str>) -> Result<Option<i64>> {
    match text {
        None | Some("") => Ok(None),
        Some(value) => value
            .parse::<i64>()
            .map(Some)
            .map_err(|_| Error::new(ErrorKind::InvalidInput)),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_optional_integer_text;
    use framework_core::ErrorKind;

    #[test]
    fn nil_and_empty_text_are_optional_none() {
        assert_eq!(parse_optional_integer_text(None), Ok(None));
        assert_eq!(parse_optional_integer_text(Some("")), Ok(None));
    }

    #[test]
    fn signed_i64_boundaries_parse_exactly() {
        assert_eq!(
            parse_optional_integer_text(Some("9223372036854775807")),
            Ok(Some(i64::MAX))
        );
        assert_eq!(
            parse_optional_integer_text(Some("-9223372036854775808")),
            Ok(Some(i64::MIN))
        );
        assert_eq!(parse_optional_integer_text(Some("0")), Ok(Some(0)));
    }

    #[test]
    fn fractional_malformed_and_overflow_text_rejects() {
        for text in [
            "1.0",
            "1.5",
            "1,5",
            "1e3",
            "12x",
            "--1",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            assert_eq!(
                parse_optional_integer_text(Some(text)).unwrap_err().kind(),
                ErrorKind::InvalidInput,
                "{text}"
            );
        }
    }
}
