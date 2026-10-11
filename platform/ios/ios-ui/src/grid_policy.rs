use framework_core::{Error, ErrorKind, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridPolicy {
    pub(crate) columns: u8,
    pub(crate) row_spacing: f64,
    pub(crate) column_spacing: f64,
}

impl GridPolicy {
    pub(crate) fn new(columns: u8, row_spacing: f64, column_spacing: f64) -> Result<Self> {
        if columns == 0
            || !is_valid_grid_spacing(row_spacing)
            || !is_valid_grid_spacing(column_spacing)
        {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self {
            columns,
            row_spacing,
            column_spacing,
        })
    }
}

fn is_valid_grid_spacing(spacing: f64) -> bool {
    spacing.is_finite() && spacing >= 0.0
}

#[cfg(test)]
mod tests {
    use framework_core::ErrorKind;

    use super::GridPolicy;

    #[test]
    fn accepts_one_and_two_column_layouts() {
        assert_eq!(
            GridPolicy::new(1, 0.0, 0.0).unwrap(),
            GridPolicy {
                columns: 1,
                row_spacing: 0.0,
                column_spacing: 0.0,
            }
        );
        assert_eq!(
            GridPolicy::new(2, 10.0, 8.0).unwrap(),
            GridPolicy {
                columns: 2,
                row_spacing: 10.0,
                column_spacing: 8.0,
            }
        );
    }

    #[test]
    fn rejects_zero_columns_and_negative_spacing() {
        assert_eq!(
            GridPolicy::new(0, 0.0, 0.0).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            GridPolicy::new(2, -1.0, 0.0).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            GridPolicy::new(2, 0.0, -1.0).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn rejects_non_finite_spacing() {
        for spacing in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                GridPolicy::new(2, spacing, 0.0).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
            assert_eq!(
                GridPolicy::new(2, 0.0, spacing).unwrap_err().kind(),
                ErrorKind::InvalidInput
            );
        }
    }
}
