#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_speech_status::authorization_status());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
