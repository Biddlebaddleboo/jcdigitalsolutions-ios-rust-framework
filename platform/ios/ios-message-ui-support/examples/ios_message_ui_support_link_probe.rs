#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    let Some(mtm) = objc2::MainThreadMarker::new() else {
        return;
    };

    core::hint::black_box(ios_message_ui_support::can_send_mail(mtm));
    core::hint::black_box(ios_message_ui_support::can_send_text(mtm));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
