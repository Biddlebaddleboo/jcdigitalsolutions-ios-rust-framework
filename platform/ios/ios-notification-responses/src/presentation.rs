/// Foreground notification styles selected by an iOS notification delegate
///
/// All styles are disabled by default. A requested banner uses Apple's legacy alert presentation
/// option on iOS 10 through iOS 13, where the banner option is not available
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ForegroundPresentationOptions {
    banner: bool,
    sound: bool,
    badge: bool,
}

impl ForegroundPresentationOptions {
    /// Create a selection with no foreground presentation
    pub const fn new() -> Self {
        Self {
            banner: false,
            sound: false,
            badge: false,
        }
    }

    /// Request a banner on iOS 14 or later, or the legacy alert presentation on iOS 10 through iOS 13
    pub const fn with_banner(mut self, enabled: bool) -> Self {
        self.banner = enabled;
        self
    }

    /// Request notification sound during foreground presentation
    pub const fn with_sound(mut self, enabled: bool) -> Self {
        self.sound = enabled;
        self
    }

    /// Request badge updates during foreground presentation
    pub const fn with_badge(mut self, enabled: bool) -> Self {
        self.badge = enabled;
        self
    }

    #[cfg(target_os = "ios")]
    pub(crate) const fn banner_requested(self) -> bool {
        self.banner
    }

    pub(crate) const fn for_system(self, supports_banner: bool) -> SystemPresentationOptions {
        SystemPresentationOptions {
            banner: self.banner && supports_banner,
            legacy_alert: self.banner && !supports_banner,
            sound: self.sound,
            badge: self.badge,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SystemPresentationOptions {
    pub(crate) banner: bool,
    pub(crate) legacy_alert: bool,
    pub(crate) sound: bool,
    pub(crate) badge: bool,
}

#[cfg(test)]
mod tests {
    use super::ForegroundPresentationOptions;

    #[test]
    fn default_and_new_suppress_foreground_presentation() {
        assert_eq!(
            ForegroundPresentationOptions::default().for_system(true),
            ForegroundPresentationOptions::new().for_system(false)
        );
        assert_eq!(
            ForegroundPresentationOptions::new().for_system(true),
            Default::default()
        );
        assert_eq!(
            ForegroundPresentationOptions::new().for_system(false),
            Default::default()
        );
    }

    #[test]
    fn banner_uses_the_modern_option_only_when_supported() {
        let selection = ForegroundPresentationOptions::new().with_banner(true);
        let modern = selection.for_system(true);
        let legacy = selection.for_system(false);

        assert!(modern.banner);
        assert!(!modern.legacy_alert);
        assert!(!legacy.banner);
        assert!(legacy.legacy_alert);
    }

    #[test]
    fn sound_and_badge_are_independent_of_banner_availability() {
        let selection = ForegroundPresentationOptions::new()
            .with_sound(true)
            .with_badge(true);

        for supports_banner in [false, true] {
            let actual = selection.for_system(supports_banner);
            assert!(!actual.banner);
            assert!(!actual.legacy_alert);
            assert!(actual.sound);
            assert!(actual.badge);
        }
    }

    #[test]
    fn banner_sound_and_badge_combine_without_extra_options() {
        let selection = ForegroundPresentationOptions::new()
            .with_banner(true)
            .with_sound(true)
            .with_badge(true);
        let modern = selection.for_system(true);
        let legacy = selection.for_system(false);

        assert_eq!(
            modern,
            super::SystemPresentationOptions {
                banner: true,
                legacy_alert: false,
                sound: true,
                badge: true,
            }
        );
        assert_eq!(
            legacy,
            super::SystemPresentationOptions {
                banner: false,
                legacy_alert: true,
                sound: true,
                badge: true,
            }
        );
    }
}
