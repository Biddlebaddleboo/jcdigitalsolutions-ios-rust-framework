use framework_core::ErrorKind;

pub(crate) fn perform_push_if_unparented<T>(
    already_parented: bool,
    push: impl FnOnce() -> T,
) -> core::result::Result<T, ErrorKind> {
    if already_parented {
        Err(ErrorKind::InvalidInput)
    } else {
        Ok(push())
    }
}

#[cfg(test)]
mod tests {
    use super::perform_push_if_unparented;
    use framework_core::ErrorKind;

    #[test]
    fn unparented_screen_invokes_the_push_once() {
        let mut pushes = 0;
        let result = perform_push_if_unparented(false, || {
            pushes += 1;
            17
        });

        assert_eq!(result, Ok(17));
        assert_eq!(pushes, 1);
    }

    #[test]
    fn already_parented_screen_is_rejected_without_calling_uikit_push() {
        let mut pushed = false;
        let result = perform_push_if_unparented(true, || pushed = true);

        assert_eq!(result, Err(ErrorKind::InvalidInput));
        assert!(!pushed);
    }
}
