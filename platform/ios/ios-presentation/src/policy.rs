pub(crate) fn valid_popover_source_rect<T: Into<f64>>(x: T, y: T, width: T, height: T) -> bool {
    let x = x.into();
    let y = y.into();
    let width = width.into();
    let height = height.into();
    x.is_finite()
        && y.is_finite()
        && width.is_finite()
        && height.is_finite()
        && width >= 0.0
        && height >= 0.0
}

#[cfg(test)]
mod tests {
    use super::valid_popover_source_rect;

    #[test]
    fn accepts_zero_and_finite_nonnegative_anchor_rects() {
        assert!(valid_popover_source_rect(0.0, 0.0, 0.0, 0.0));
        assert!(valid_popover_source_rect(-4.0, 8.0, 12.0, 16.0));
    }

    #[test]
    fn rejects_nonfinite_or_negative_anchor_rect_dimensions() {
        assert!(!valid_popover_source_rect(f64::NAN, 0.0, 1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, f64::INFINITY, 1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, 0.0, -1.0, 1.0));
        assert!(!valid_popover_source_rect(0.0, 0.0, 1.0, f64::NEG_INFINITY));
    }
}
