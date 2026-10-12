#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Executor-neutral local notifications through public iOS UserNotifications APIs."]

extern crate alloc;

mod actions;
mod authorization;
mod conversion;
mod operation;

pub use actions::{
    IosNotificationAction, IosNotificationActionOptions, IosNotificationCategory,
    IosNotificationConfigurationError,
};
pub use authorization::IosNotificationAuthorizationOptions;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosAuthorizationFuture, IosAuthorizationStatusRawValueFuture, IosCancelFuture,
    IosNotificationSettingRawValues, IosNotificationSettingRawValuesFuture,
    IosNotificationSettingsExtendedRawValues, IosNotificationSettingsExtendedRawValuesFuture,
    IosNotificationSettingsSurfaceRawValues, IosNotificationSettingsSurfaceRawValuesFuture,
    IosNotificationsBackend, IosPendingRequestCountFuture, IosRequestAuthorizationFuture,
    IosScheduleFuture,
};

#[cfg(test)]
mod notification_content_tests {
    use framework_notifications::{NotificationCategoryId, NotificationContent, NotificationError};

    #[test]
    fn existing_content_keeps_system_default_sound_disabled() {
        let content = NotificationContent::new(String::from("Title"), None);
        assert!(!content.uses_system_default_sound());
    }

    #[test]
    fn content_can_opt_into_system_default_sound() {
        let content =
            NotificationContent::new(String::from("Title"), None).with_system_default_sound();
        assert!(content.uses_system_default_sound());
    }

    #[test]
    fn content_metadata_is_absent_by_default() {
        let content = NotificationContent::new(String::from("Title"), None);
        assert_eq!(content.category_identifier(), None);
        assert_eq!(content.thread_identifier(), None);
        assert!(content.user_info_entries().is_empty());
    }

    #[test]
    fn content_category_identifier_is_opt_in_and_exact() {
        let category = NotificationCategoryId::new(String::from("reply.é")).unwrap();
        let content = NotificationContent::new(String::from("Title"), None)
            .with_category_identifier(category);
        assert_eq!(content.category_identifier(), Some("reply.é"));
    }

    #[test]
    fn invalid_category_identifiers_are_rejected() {
        assert_eq!(
            NotificationCategoryId::new(String::new()).err(),
            Some(NotificationError::InvalidCategoryIdentifier)
        );
        assert_eq!(
            NotificationCategoryId::new(String::from("bad\0category")).err(),
            Some(NotificationError::InvalidCategoryIdentifier)
        );
    }

    #[test]
    fn badge_count_defaults_to_unchanged_and_preserves_zero_and_maximum() {
        let content = NotificationContent::new(String::from("Title"), None);
        assert_eq!(content.badge_count(), None);
        assert_eq!(content.clone().with_badge_count(0).badge_count(), Some(0));
        assert_eq!(
            content.with_badge_count(u32::MAX).badge_count(),
            Some(u32::MAX)
        );
    }

    #[test]
    fn content_metadata_preserves_unicode_and_last_duplicate_value() {
        let content = NotificationContent::new(String::from("Title"), None)
            .with_thread_identifier(String::from("alerts/é/🔔"))
            .with_user_info_entry(
                String::from("route"),
                String::from("urn:demo:café?place=🛰️"),
            )
            .with_user_info_entry(String::from("unicode"), String::from("cafe\u{301}🛰️"))
            .with_user_info_entry(String::from("source"), String::from("local"))
            .with_user_info_entry(
                String::from("route"),
                String::from("urn:demo:東京?place=🗺️"),
            );

        assert_eq!(content.thread_identifier(), Some("alerts/é/🔔"));
        assert_eq!(
            content.user_info_entries(),
            &[
                (
                    String::from("route"),
                    String::from("urn:demo:東京?place=🗺️")
                ),
                (String::from("unicode"), String::from("cafe\u{301}🛰️")),
                (String::from("source"), String::from("local")),
            ]
        );
    }
}
