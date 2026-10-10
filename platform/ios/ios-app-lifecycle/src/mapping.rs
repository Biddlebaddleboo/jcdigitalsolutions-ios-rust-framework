use framework_app::ApplicationState;
use framework_core::{Error, ErrorKind, PlatformErrorCode, Result};

pub(crate) fn application_state(raw_state: i32) -> Result<ApplicationState> {
    match raw_state {
        0 => Ok(ApplicationState::Foreground),
        1 => Ok(ApplicationState::Inactive),
        2 => Ok(ApplicationState::Background),
        value => Err(Error::new(ErrorKind::Platform).with_platform_code(
            PlatformErrorCode::new(value).expect("unknown UIApplicationState is nonzero"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_uikit_application_state_values_map_to_portable_state() {
        assert_eq!(application_state(0), Ok(ApplicationState::Foreground));
        assert_eq!(application_state(1), Ok(ApplicationState::Inactive));
        assert_eq!(application_state(2), Ok(ApplicationState::Background));
    }

    #[test]
    fn unknown_signed_uikit_state_preserves_its_platform_code() {
        let code = PlatformErrorCode::new(-27).unwrap();
        assert_eq!(
            application_state(-27),
            Err(Error::new(ErrorKind::Platform).with_platform_code(code))
        );
    }
}
