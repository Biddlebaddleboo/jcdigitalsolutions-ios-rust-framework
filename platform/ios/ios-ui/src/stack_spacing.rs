pub(crate) fn is_valid_stack_spacing(spacing: f64) -> bool {
    spacing.is_finite()
}

#[cfg(test)]
mod tests {
    use super::is_valid_stack_spacing;

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
}
