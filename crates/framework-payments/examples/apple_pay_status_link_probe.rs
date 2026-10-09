#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(framework_payments::ios::can_make_payments());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
