use framework_nfc::{NfcReaderAvailability, NfcReaderAvailabilityBackend};
use objc2_core_nfc::NFCReaderSession;

/// Stateless iOS backend for the Core NFC reader-support property.
///
/// Construction is inert. Calling [`NfcReaderAvailabilityBackend::snapshot`] only queries
/// `NFCReaderSession.readingAvailable`; it does not create a reader session or use NFC radio
/// polling.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoreNfcReaderBackend;

impl NfcReaderAvailabilityBackend for CoreNfcReaderBackend {
    fn snapshot(&self) -> NfcReaderAvailability {
        // SAFETY: `readingAvailable` is a readonly class property with no instance preconditions.
        // This call only asks Core NFC whether the device supports NFC tag reading; it does not
        // allocate a session, prompt, enable polling, or return a borrowed native value.
        if unsafe { NFCReaderSession::readingAvailable() } {
            NfcReaderAvailability::Supported
        } else {
            NfcReaderAvailability::Unsupported
        }
    }
}
