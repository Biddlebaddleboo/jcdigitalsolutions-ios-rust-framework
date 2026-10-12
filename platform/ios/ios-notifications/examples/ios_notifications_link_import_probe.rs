#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_notifications::response::NotificationActionId;
#[cfg(target_os = "ios")]
use framework_notifications::{
    Notification, NotificationBackend, NotificationCategoryId, NotificationContent, NotificationId,
    NotificationTrigger,
};
#[cfg(target_os = "ios")]
use ios_notifications::IosNotificationsBackend;
#[cfg(target_os = "ios")]
use ios_notifications::{
    IosNotificationAction, IosNotificationActionOptions, IosNotificationAuthorizationOptions,
    IosNotificationCategory,
};

#[cfg(target_os = "ios")]
fn main() {
    let mut backend = IosNotificationsBackend::new();
    core::mem::drop(core::hint::black_box(backend.authorization()));
    core::mem::drop(core::hint::black_box(
        backend.authorization_status_raw_value(),
    ));
    core::mem::drop(core::hint::black_box(
        backend.notification_setting_raw_values(),
    ));
    core::mem::drop(core::hint::black_box(
        backend.notification_settings_extended_raw_values(),
    ));
    core::mem::drop(core::hint::black_box(
        backend.notification_settings_surface_raw_values(),
    ));
    core::mem::drop(core::hint::black_box(backend.request_authorization()));
    core::mem::drop(core::hint::black_box(
        backend.request_authorization_with_options(
            IosNotificationAuthorizationOptions::ALERT
                | IosNotificationAuthorizationOptions::BADGE
                | IosNotificationAuthorizationOptions::SOUND,
        ),
    ));
    core::mem::drop(core::hint::black_box(backend.pending_request_count()));

    let category_identifier =
        NotificationCategoryId::new(String::from("link-probe.category")).unwrap();
    let mut category = IosNotificationCategory::new(category_identifier.clone());
    category
        .add_action(IosNotificationAction::button(
            NotificationActionId::new(String::from("link-probe.open")).unwrap(),
            String::from("Open"),
            IosNotificationActionOptions::FOREGROUND,
        ))
        .unwrap();
    category
        .add_action(IosNotificationAction::text_input(
            NotificationActionId::new(String::from("link-probe.reply")).unwrap(),
            String::from("Reply"),
            String::from("Send"),
            String::from("Write a reply"),
            IosNotificationActionOptions::AUTHENTICATION_REQUIRED,
        ))
        .unwrap();
    let category = category.with_custom_dismiss_action(true);
    let _ = backend.register_categories(core::slice::from_ref(&category));

    if let Ok(identifier) = NotificationId::new(String::from("link-probe")) {
        let notification = Notification::new(
            identifier,
            NotificationContent::new(String::from("Link probe"), None)
                .with_badge_count(3)
                .with_system_default_sound()
                .with_category_identifier(category_identifier)
                .with_thread_identifier(String::from("probe-group"))
                .with_user_info_entry(String::from("reference"), String::from("item:42")),
            NotificationTrigger::immediate(),
        );
        core::mem::drop(core::hint::black_box(backend.schedule(notification)));
    }

    if let Ok(identifier) = NotificationId::new(String::from("link-probe-cancel")) {
        core::mem::drop(core::hint::black_box(backend.cancel(identifier)));
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
