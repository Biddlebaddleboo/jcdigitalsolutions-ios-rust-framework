#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_safety::is_crash_detection_available());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
