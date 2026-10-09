#[cfg(target_os = "ios")]
fn main() {
    std::hint::black_box(ios_mps_status::preferred_mps_device_available());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
