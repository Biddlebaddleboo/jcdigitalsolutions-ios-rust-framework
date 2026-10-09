#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_metal::MetalDevicePresenceBackend;
    use ios_metal::CoreMetalDeviceBackend;

    let backend = CoreMetalDeviceBackend;
    let _ = core::hint::black_box(backend.snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
