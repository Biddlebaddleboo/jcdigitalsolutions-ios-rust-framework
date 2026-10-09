#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_notifications::{
    Notification, NotificationBackend, NotificationContent, NotificationId, NotificationTrigger,
};
#[cfg(target_os = "ios")]
use ios_notifications::IosNotificationsBackend;

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
    core::mem::drop(core::hint::black_box(backend.pending_request_count()));

    if let Ok(identifier) = NotificationId::new(String::from("link-probe")) {
        let notification = Notification::new(
            identifier,
            NotificationContent::new(String::from("Link probe"), None),
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
