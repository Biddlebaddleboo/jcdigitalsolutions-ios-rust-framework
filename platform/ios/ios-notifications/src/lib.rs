#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Executor-neutral local notifications through public iOS UserNotifications APIs."]

mod authorization;
mod conversion;
mod operation;

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
mod notification_sound_tests {
    use framework_notifications::NotificationContent;

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
        assert_eq!(content.thread_identifier(), None);
        assert!(content.user_info_entries().is_empty());
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
