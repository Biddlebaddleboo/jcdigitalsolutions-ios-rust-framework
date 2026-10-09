#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    let _ = core::hint::black_box(ios_sound_analysis::support_snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
