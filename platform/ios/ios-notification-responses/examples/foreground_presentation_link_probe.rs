#[cfg(target_os = "ios")]
use ios_notification_responses::{ForegroundPresentationOptions, IosNotificationResponses};

#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn ios_notification_responses_foreground_link_probe() -> u8 {
    let options = ForegroundPresentationOptions::new()
        .with_banner(true)
        .with_sound(true)
        .with_badge(true);
    match IosNotificationResponses::install_with_foreground_presentation(|_| {}, options) {
        Ok(handle) => {
            drop(handle);
            1
        }
        Err(_) => 0,
    }
}

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_notification_responses_foreground_link_probe());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
