#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    let _ = core::hint::black_box(ios_playback::hdr_playback_eligibility());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
