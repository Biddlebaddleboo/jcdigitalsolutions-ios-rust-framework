#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    let _ = core::hint::black_box(ios_replaykit::availability_snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
