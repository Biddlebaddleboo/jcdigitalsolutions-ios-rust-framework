use framework_core::{Error, ErrorKind, Result};

pub(crate) fn validated_content_size(width: f64, height: f64) -> Result<(f64, f64)> {
    if width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0 {
        Ok((width, height))
    } else {
        Err(Error::new(ErrorKind::InvalidInput))
    }
}

#[cfg(test)]
mod tests {
    use framework_core::ErrorKind;

    use super::validated_content_size;

    #[test]
    fn content_sizes_reject_invalid_without_clamping() {
        assert_eq!(validated_content_size(0.0, -0.0).unwrap(), (0.0, -0.0));
        assert_eq!(
            validated_content_size(12.5, f64::MAX).unwrap(),
            (12.5, f64::MAX)
        );
        for (width, height) in [
            (-1.0, 0.0),
            (0.0, -1.0),
            (f64::NAN, 1.0),
            (1.0, f64::NAN),
            (f64::INFINITY, 1.0),
            (1.0, f64::INFINITY),
            (f64::NEG_INFINITY, 1.0),
            (1.0, f64::NEG_INFINITY),
        ] {
            assert_eq!(
                validated_content_size(width, height).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
        }
    }
}
