pub(crate) fn is_valid_stack_spacing(spacing: f64) -> bool {
    spacing.is_finite()
}

pub(crate) fn is_valid_stack_alignment(horizontal_stack: bool, baseline_alignment: bool) -> bool {
    horizontal_stack || !baseline_alignment
}

#[cfg(test)]
mod tests {
    use super::{is_valid_stack_alignment, is_valid_stack_spacing};

    #[test]
    fn accepts_finite_spacing() {
        assert!(is_valid_stack_spacing(0.0));
        assert!(is_valid_stack_spacing(8.5));
        assert!(is_valid_stack_spacing(-2.0));
    }

    #[test]
    fn rejects_non_finite_spacing() {
        assert!(!is_valid_stack_spacing(f64::NAN));
        assert!(!is_valid_stack_spacing(f64::INFINITY));
        assert!(!is_valid_stack_spacing(f64::NEG_INFINITY));
    }

    #[test]
    fn baseline_alignment_requires_a_horizontal_stack() {
        assert!(is_valid_stack_alignment(true, true));
        assert!(!is_valid_stack_alignment(false, true));
    }

    #[test]
    fn cross_axis_alignment_is_valid_for_both_stack_axes() {
        assert!(is_valid_stack_alignment(true, false));
        assert!(is_valid_stack_alignment(false, false));
    }
}
