use framework_core::ErrorKind;

pub(crate) fn perform_if_unattached<T>(
    already_parented: bool,
    already_in_window: bool,
    operation: impl FnOnce() -> T,
) -> core::result::Result<T, ErrorKind> {
    if already_parented || already_in_window {
        Err(ErrorKind::InvalidInput)
    } else {
        Ok(operation())
    }
}

#[cfg(test)]
mod tests {
    use super::perform_if_unattached;
    use framework_core::ErrorKind;

    #[test]
    fn unattached_controller_invokes_the_operation_once() {
        let mut operations = 0;
        let result = perform_if_unattached(false, false, || {
            operations += 1;
            17
        });

        assert_eq!(result, Ok(17));
        assert_eq!(operations, 1);
    }

    #[test]
    fn already_parented_screen_is_rejected_without_calling_uikit_push() {
        let mut operated = false;
        let result = perform_if_unattached(true, false, || operated = true);

        assert_eq!(result, Err(ErrorKind::InvalidInput));
        assert!(!operated);
    }

    #[test]
    fn already_window_attached_screen_is_rejected_without_calling_uikit_push() {
        let mut operated = false;
        let result = perform_if_unattached(false, true, || operated = true);

        assert_eq!(result, Err(ErrorKind::InvalidInput));
        assert!(!operated);
    }
}
