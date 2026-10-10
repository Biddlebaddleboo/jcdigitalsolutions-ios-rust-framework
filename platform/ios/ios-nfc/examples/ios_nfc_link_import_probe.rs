#[cfg(target_os = "ios")]
fn main() {
    use framework_nfc::NfcReaderAvailabilityBackend;

    core::hint::black_box(ios_nfc::CoreNfcReaderBackend.snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
