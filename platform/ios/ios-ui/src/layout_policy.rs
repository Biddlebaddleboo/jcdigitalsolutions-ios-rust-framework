use framework_core::{Error, ErrorKind, Result};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ScrollContentMode {
    #[default]
    Unselected,
    Manual,
    Intrinsic,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ScrollLayoutPolicy(ScrollContentMode);

impl ScrollLayoutPolicy {
    pub(crate) fn select_manual(self) -> Result<Self> {
        match self.0 {
            ScrollContentMode::Unselected | ScrollContentMode::Manual => {
                Ok(Self(ScrollContentMode::Manual))
            }
            ScrollContentMode::Intrinsic => Err(Error::new(ErrorKind::InvalidInput)),
        }
    }

    pub(crate) fn select_intrinsic(self) -> Result<Self> {
        match self.0 {
            ScrollContentMode::Unselected => Ok(Self(ScrollContentMode::Intrinsic)),
            ScrollContentMode::Manual | ScrollContentMode::Intrinsic => {
                Err(Error::new(ErrorKind::InvalidInput))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OneShotConstraints(bool);

impl OneShotConstraints {
    pub(crate) fn claim(self) -> Result<Self> {
        if self.0 {
            Err(Error::new(ErrorKind::InvalidInput))
        } else {
            Ok(Self(true))
        }
    }
}

#[cfg(test)]
mod tests {
    use framework_core::ErrorKind;

    use super::{OneShotConstraints, ScrollLayoutPolicy};

    #[test]
    fn manual_content_sizing_can_be_updated_repeatedly() {
        let policy = ScrollLayoutPolicy::default();
        let policy = policy.select_manual().unwrap();
        assert_eq!(policy.select_manual().unwrap(), policy);
    }

    #[test]
    fn intrinsic_layout_is_one_shot_and_rejects_manual_sizing() {
        let policy = ScrollLayoutPolicy::default().select_intrinsic().unwrap();
        assert_eq!(
            policy.select_manual().unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            policy.select_intrinsic().unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn manual_sizing_rejects_intrinsic_layout() {
        let policy = ScrollLayoutPolicy::default().select_manual().unwrap();
        assert_eq!(
            policy.select_intrinsic().unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn constraint_setup_is_one_shot() {
        let constraints = OneShotConstraints::default().claim().unwrap();
        assert_eq!(
            constraints.claim().unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }
}
