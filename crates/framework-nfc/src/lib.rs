#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable no_std NFC reader-support snapshots with a caller-selected static backend."]

/// One point-in-time report of whether the selected backend knows that NFC tag reading is
/// supported by the device.
///
/// `Supported` describes device reader support only. It does not establish app permission, the
/// required usage description or entitlement, a usable or started session, scan success, tag
/// presence, tag reads, or background NFC support. This crate exposes no session, prompt, scan,
/// polling, tag, or background operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum NfcReaderAvailability {
    /// The backend cannot determine whether NFC tag reading is supported.
    Unknown = 0,
    /// The backend reports that the device supports NFC tag reading.
    Supported = 1,
    /// The backend reports that the device does not support NFC tag reading.
    Unsupported = 2,
}

/// A statically selected backend that returns an NFC reader-support snapshot.
pub trait NfcReaderAvailabilityBackend {
    /// Returns the backend's current point-in-time reader-support report.
    ///
    /// This synchronous query must not create or start a reader session, prompt the user, scan for
    /// tags, read a tag, or perform background NFC work. Its result is not a promise of permission,
    /// session availability, scan success, tag presence, or readable tag data. A backend that
    /// cannot classify support returns `Unknown`.
    fn snapshot(&self) -> NfcReaderAvailability;
}

/// A thin facade over caller-owned, statically selected NFC reader-support state.
pub struct NfcReader<B> {
    backend: B,
}

impl<B: NfcReaderAvailabilityBackend> NfcReader<B> {
    /// Creates the facade around an explicitly supplied backend without starting native work.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Returns one point-in-time reader-support snapshot.
    pub fn snapshot(&self) -> NfcReaderAvailability {
        self.backend.snapshot()
    }

    /// Borrows the selected backend for backend-specific controls.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns the selected backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend(NfcReaderAvailability);

    impl NfcReaderAvailabilityBackend for FakeBackend {
        fn snapshot(&self) -> NfcReaderAvailability {
            self.0
        }
    }

    #[test]
    fn facade_forwards_all_statuses_and_returns_owned_backend() {
        assert_eq!(core::mem::size_of::<NfcReaderAvailability>(), 1);
        for expected in [
            NfcReaderAvailability::Unknown,
            NfcReaderAvailability::Supported,
            NfcReaderAvailability::Unsupported,
        ] {
            let reader = NfcReader::new(FakeBackend(expected));
            assert_eq!(reader.snapshot(), expected);
            assert_eq!(reader.backend().snapshot(), expected);
            assert_eq!(reader.into_backend().snapshot(), expected);
        }
    }
}
