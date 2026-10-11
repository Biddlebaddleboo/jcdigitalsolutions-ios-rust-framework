use core::ops::BitOr;

/// Allocation-free choices for the local-notification authorization prompt.
///
/// Only alert, badge, and sound authorization are represented. Critical, provisional, and remote
/// notification options are intentionally excluded.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IosNotificationAuthorizationOptions(u8);

impl IosNotificationAuthorizationOptions {
    /// Request no alert, badge, or sound authorization.
    pub const NONE: Self = Self(0);
    /// Request alert authorization.
    pub const ALERT: Self = Self(1);
    /// Request badge authorization.
    pub const BADGE: Self = Self(2);
    /// Request sound authorization.
    pub const SOUND: Self = Self(4);

    /// Combine authorization choices.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Return whether every choice in `other` is present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) const fn requests_alert(self) -> bool {
        self.contains(Self::ALERT)
    }

    pub(crate) const fn requests_badge(self) -> bool {
        self.contains(Self::BADGE)
    }

    pub(crate) const fn requests_sound(self) -> bool {
        self.contains(Self::SOUND)
    }
}

impl BitOr for IosNotificationAuthorizationOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

pub(crate) const DEFAULT_AUTHORIZATION_OPTIONS: IosNotificationAuthorizationOptions =
    IosNotificationAuthorizationOptions::ALERT;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alert_badge_sound_combination_selects_only_requested_options() {
        let combinations = [
            (
                IosNotificationAuthorizationOptions::NONE,
                false,
                false,
                false,
            ),
            (
                IosNotificationAuthorizationOptions::ALERT,
                true,
                false,
                false,
            ),
            (
                IosNotificationAuthorizationOptions::BADGE,
                false,
                true,
                false,
            ),
            (
                IosNotificationAuthorizationOptions::ALERT
                    | IosNotificationAuthorizationOptions::BADGE,
                true,
                true,
                false,
            ),
            (
                IosNotificationAuthorizationOptions::SOUND,
                false,
                false,
                true,
            ),
            (
                IosNotificationAuthorizationOptions::ALERT
                    | IosNotificationAuthorizationOptions::SOUND,
                true,
                false,
                true,
            ),
            (
                IosNotificationAuthorizationOptions::BADGE
                    | IosNotificationAuthorizationOptions::SOUND,
                false,
                true,
                true,
            ),
            (
                IosNotificationAuthorizationOptions::ALERT
                    | IosNotificationAuthorizationOptions::BADGE
                    | IosNotificationAuthorizationOptions::SOUND,
                true,
                true,
                true,
            ),
        ];

        for (options, alert, badge, sound) in combinations {
            assert_eq!(options.requests_alert(), alert);
            assert_eq!(options.requests_badge(), badge);
            assert_eq!(options.requests_sound(), sound);
        }
    }

    #[test]
    fn portable_request_default_remains_alert_only() {
        assert_eq!(
            DEFAULT_AUTHORIZATION_OPTIONS,
            IosNotificationAuthorizationOptions::ALERT
        );
    }
}
