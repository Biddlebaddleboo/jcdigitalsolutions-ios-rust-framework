#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_core_ml_status::has_available_compute_device());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
