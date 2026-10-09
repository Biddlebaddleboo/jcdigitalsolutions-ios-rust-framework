#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    let _ = core::hint::black_box(ios_device_integrity::availability_snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
