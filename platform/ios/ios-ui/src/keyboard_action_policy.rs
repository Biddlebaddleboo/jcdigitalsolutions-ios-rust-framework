use framework_core::{Error, ErrorKind, Result};

pub(super) fn run_with_main_thread_marker<T>(
    marker: Option<T>,
    action: impl FnOnce(T) -> bool,
) -> Result<bool> {
    match marker {
        Some(marker) => Ok(action(marker)),
        None => Err(Error::new(ErrorKind::Unavailable)),
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use framework_core::ErrorKind;

    use super::run_with_main_thread_marker;

    #[test]
    fn missing_main_thread_proof_rejects_without_sending_action() {
        let called = Cell::new(false);
        let result = run_with_main_thread_marker(None::<()>, |_| {
            called.set(true);
            true
        });

        assert!(matches!(
            result,
            Err(error) if error.kind() == ErrorKind::Unavailable
        ));
        assert!(!called.get());
    }

    #[test]
    fn action_result_is_returned_unchanged() {
        let calls = Cell::new(0);
        for expected in [false, true] {
            let result = run_with_main_thread_marker(Some(()), |_| {
                calls.set(calls.get() + 1);
                expected
            });
            assert!(matches!(result, Ok(actual) if actual == expected));
        }
        assert_eq!(calls.get(), 2);
    }
}
