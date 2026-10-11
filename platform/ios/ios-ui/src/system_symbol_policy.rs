use framework_core::{Error, ErrorKind, Result};

pub(crate) fn validate_system_symbol_name(name: &str) -> Result<()> {
    if name.is_empty() {
        Err(Error::new(ErrorKind::InvalidInput))
    } else {
        Ok(())
    }
}

pub(crate) fn resolve_system_symbol<T>(image: Option<T>) -> Result<T> {
    image.ok_or_else(|| Error::new(ErrorKind::Unavailable))
}

#[cfg(test)]
mod tests {
    use framework_core::ErrorKind;

    use super::{resolve_system_symbol, validate_system_symbol_name};

    #[test]
    fn rejects_empty_symbol_names() {
        assert_eq!(
            validate_system_symbol_name("").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn accepts_nonempty_system_symbol_names_without_an_allowlist() {
        assert!(validate_system_symbol_name("doc.on.doc").is_ok());
        assert!(validate_system_symbol_name("new.public.symbol").is_ok());
    }

    #[test]
    fn reports_unknown_symbols_as_unavailable() {
        assert_eq!(
            resolve_system_symbol::<u8>(None).unwrap_err().kind(),
            ErrorKind::Unavailable
        );
        assert_eq!(resolve_system_symbol(Some(7)).unwrap(), 7);
    }
}
