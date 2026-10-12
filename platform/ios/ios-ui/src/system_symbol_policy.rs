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

pub(crate) fn update_system_symbol_image<T>(
    symbol_name: Option<&str>,
    symbols_available: impl FnOnce() -> bool,
    resolve: impl FnOnce(&str) -> Option<T>,
    update: impl FnOnce(Option<T>),
) -> Result<()> {
    let Some(symbol_name) = symbol_name else {
        update(None);
        return Ok(());
    };

    validate_system_symbol_name(symbol_name)?;
    if !symbols_available() {
        return Err(Error::new(ErrorKind::Unsupported));
    }
    let image = resolve_system_symbol(resolve(symbol_name))?;
    update(Some(image));
    Ok(())
}

#[cfg(test)]
mod tests {
    use framework_core::ErrorKind;

    use super::{resolve_system_symbol, update_system_symbol_image, validate_system_symbol_name};

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

    #[test]
    fn rejects_empty_button_symbols_without_mutating_the_current_image() {
        let mut current = Some("old image");
        let mut resolved = false;

        let result = update_system_symbol_image(
            Some(""),
            || true,
            |_| {
                resolved = true;
                Some("new image")
            },
            |image| current = image,
        );

        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        assert!(!resolved);
        assert_eq!(current, Some("old image"));
    }

    #[test]
    fn rejects_missing_button_symbols_without_mutating_the_current_image() {
        let mut current = Some("old image");

        let result = update_system_symbol_image(
            Some("unknown.symbol"),
            || true,
            |_| None::<&str>,
            |image| current = image,
        );

        assert_eq!(result.unwrap_err().kind(), ErrorKind::Unavailable);
        assert_eq!(current, Some("old image"));
    }

    #[test]
    fn clearing_a_button_symbol_removes_the_current_image() {
        let mut current = Some("old image");
        let mut availability_checked = false;
        let mut resolved = false;

        update_system_symbol_image(
            None,
            || {
                availability_checked = true;
                false
            },
            |_| {
                resolved = true;
                Some("unexpected image")
            },
            |image| current = image,
        )
        .unwrap();

        assert!(!availability_checked);
        assert!(!resolved);
        assert_eq!(current, None);
    }

    #[test]
    fn replacing_a_button_symbol_uses_the_resolved_image() {
        let mut current = Some("old image");

        update_system_symbol_image(
            Some("trash"),
            || true,
            |name| (name == "trash").then_some("trash image"),
            |image| current = image,
        )
        .unwrap();

        assert_eq!(current, Some("trash image"));
    }

    #[test]
    fn rejects_empty_symbols_before_reporting_unsupported() {
        let mut current = Some("old image");
        let mut availability_checked = false;
        let mut resolved = false;

        let result = update_system_symbol_image(
            Some(""),
            || {
                availability_checked = true;
                false
            },
            |_| {
                resolved = true;
                Some("new image")
            },
            |image| current = image,
        );

        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
        assert!(!availability_checked);
        assert!(!resolved);
        assert_eq!(current, Some("old image"));
    }

    #[test]
    fn rejects_symbols_as_unsupported_without_resolving_or_mutating() {
        let mut current = Some("old image");
        let mut resolved = false;

        let result = update_system_symbol_image(
            Some("trash"),
            || false,
            |_| {
                resolved = true;
                Some("new image")
            },
            |image| current = image,
        );

        assert_eq!(result.unwrap_err().kind(), ErrorKind::Unsupported);
        assert!(!resolved);
        assert_eq!(current, Some("old image"));
    }
}
