#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_shared_with_you_support::is_system_collaboration_support_available());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
