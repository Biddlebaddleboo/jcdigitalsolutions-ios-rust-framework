#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable no_std NFC reader-support snapshots with a caller-selected static backend."]

/// One point-in-time report of whether the selected backend knows that NFC tag reading is
/// supported by the device.
///
/// `Supported` describes reader support only. It does not establish that an app has the required
/// usage description or entitlement, that a reader session can start, that NFC is currently
/// usable, or that a tag can be discovered. This crate exposes no session, prompt, polling, or tag
/// operation.
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
    /// tags, or read a tag. A backend that cannot classify support returns `Unknown`.
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
    fn facade_preserves_snapshot_without_platform_or_global_state() {
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
