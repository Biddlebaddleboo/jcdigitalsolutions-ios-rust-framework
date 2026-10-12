#[cfg(target_os = "ios")]
fn main() {
    use framework_nfc::NdefReader;
    use framework_nfc::NfcReaderAvailabilityBackend;
    #[cfg(feature = "ndef-write")]
    use framework_nfc::{NdefMessage, NdefRecord, NdefTypeNameFormat, NdefWriter};
    use ios_nfc::CoreNfcNdefBackend;
    #[cfg(feature = "ndef-write")]
    use ios_nfc::CoreNfcNdefWriteBackend;
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
    #[cfg(feature = "ndef-write")]
    {
        if core::hint::black_box(false) {
            if let Some(marker) = MainThreadMarker::new() {
                if let Ok(backend) = CoreNfcNdefWriteBackend::new(marker, "Write an NDEF tag") {
                    let mut writer = NdefWriter::new(backend);
                    let message = NdefMessage::new(vec![NdefRecord::new(
                        NdefTypeNameFormat::WellKnown,
                        vec![b'T'],
                        Vec::new(),
                        vec![0, 0xff],
                    )]);
                    let mut write = writer.write(message);
                    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                    let _ =
                        core::future::Future::poll(core::pin::Pin::new(&mut write), &mut context);
                    drop(write);
                    core::hint::black_box(writer.into_backend());
                }
            }
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
