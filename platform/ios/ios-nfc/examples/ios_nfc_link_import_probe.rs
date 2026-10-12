#[cfg(target_os = "ios")]
fn main() {
    use framework_nfc::NdefReader;
    use framework_nfc::NfcReaderAvailabilityBackend;
    use ios_nfc::CoreNfcNdefBackend;
    use objc2::MainThreadMarker;

    core::hint::black_box(ios_nfc::CoreNfcReaderBackend.snapshot());
    if core::hint::black_box(false) {
        if let Some(marker) = MainThreadMarker::new() {
            if let Ok(backend) = CoreNfcNdefBackend::new(marker, "Scan an NDEF tag") {
                let mut reader = NdefReader::new(backend);
                let mut read = reader.read();
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                let _ = core::future::Future::poll(core::pin::Pin::new(&mut read), &mut context);
                drop(read);
                core::hint::black_box(reader.into_backend());
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
