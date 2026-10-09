#![deny(warnings)]

#[cfg(target_os = "ios")]
use ios_system_services::is_classkit_deep_link;
#[cfg(target_os = "ios")]
use objc2_foundation::NSUserActivity;

#[cfg(target_os = "ios")]
fn main() {
    let activity = NSUserActivity::new();
    core::hint::black_box(is_classkit_deep_link(&activity));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
