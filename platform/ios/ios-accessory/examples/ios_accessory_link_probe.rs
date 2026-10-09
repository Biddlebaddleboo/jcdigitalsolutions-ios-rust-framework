#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_accessory::ExternalAccessoryBackend;
    use ios_accessory::IosExternalAccessoryBackend;

    let _ = core::hint::black_box(IosExternalAccessoryBackend::connected_accessory_presence());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
