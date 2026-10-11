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
}
