#![allow(deprecated)]

#[cfg(target_os = "ios")]
fn main() {
    std::hint::black_box(ios_storekit_status::legacy_can_make_payments());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
