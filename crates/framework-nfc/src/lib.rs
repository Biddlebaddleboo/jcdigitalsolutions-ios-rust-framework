#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable no_std NFC reader support and NDEF read contracts with static backends."]

extern crate alloc;

#[cfg(test)]
extern crate std;

use alloc::vec::Vec;
use core::future::Future;

/// One point-in-time report of whether the selected backend knows that NFC tag reading is
/// supported by the device.
///
/// `Supported` describes device reader support only. It does not establish app permission, the
/// required usage description or entitlement, a usable or started session, scan success, tag
/// presence, tag reads, or background NFC support. This snapshot itself creates no session,
/// prompt, scan, or background operation; [`NdefReader`] is a separate user-mediated one-shot NDEF
/// API.
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

/// NFC Forum NDEF record type-name format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NdefTypeNameFormat {
    /// The record has no type field.
    Empty,
    /// NFC Forum well-known type.
    WellKnown,
    /// Media type.
    Media,
    /// Absolute URI type.
    AbsoluteUri,
    /// NFC Forum external type.
    External,
    /// An unknown or reserved format value; the byte is preserved.
    Unknown(u8),
    /// Continuation record in a chunked NDEF message.
    Unchanged,
}

impl NdefTypeNameFormat {
    /// Creates a format from the NFC Forum TNF byte while preserving reserved values.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Empty,
            1 => Self::WellKnown,
            2 => Self::Media,
            3 => Self::AbsoluteUri,
            4 => Self::External,
            6 => Self::Unchanged,
            value => Self::Unknown(value),
        }
    }

    /// Returns the encoded NFC Forum TNF byte.
    pub const fn as_raw(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::WellKnown => 1,
            Self::Media => 2,
            Self::AbsoluteUri => 3,
            Self::External => 4,
            Self::Unknown(value) => value,
            Self::Unchanged => 6,
        }
    }
}

/// One owned NDEF record with byte-preserving type, identifier, and payload fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NdefRecord {
    format: NdefTypeNameFormat,
    record_type: Vec<u8>,
    identifier: Vec<u8>,
    payload: Vec<u8>,
}

impl NdefRecord {
    /// Creates a record without interpreting or transcoding its byte fields.
    pub fn new(
        format: NdefTypeNameFormat,
        record_type: Vec<u8>,
        identifier: Vec<u8>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            format,
            record_type,
            identifier,
            payload,
        }
    }

    /// Returns the record's NFC Forum TNF.
    pub const fn format(&self) -> NdefTypeNameFormat {
        self.format
    }

    /// Returns the record type bytes without text conversion.
    pub fn record_type(&self) -> &[u8] {
        &self.record_type
    }

    /// Returns the record identifier bytes without text conversion.
    pub fn identifier(&self) -> &[u8] {
        &self.identifier
    }

    /// Returns the record payload bytes without text conversion.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the owned record fields.
    pub fn into_parts(self) -> (NdefTypeNameFormat, Vec<u8>, Vec<u8>, Vec<u8>) {
        (self.format, self.record_type, self.identifier, self.payload)
    }
}

/// An owned NDEF message in native record order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NdefMessage {
    records: Vec<NdefRecord>,
}

impl NdefMessage {
    /// Creates a message from its ordered records.
    pub const fn new(records: Vec<NdefRecord>) -> Self {
        Self { records }
    }

    /// Returns the message's ordered records.
    pub fn records(&self) -> &[NdefRecord] {
        &self.records
    }

    /// Returns the owned record list.
    pub fn into_records(self) -> Vec<NdefRecord> {
        self.records
    }
}

/// A terminal outcome from a one-shot NDEF read session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NdefReadError {
    /// The device or reader mode does not support NFC tag reading.
    Unsupported,
    /// Core NFC reports an unsupported reader feature or missing reader entitlement.
    UnsupportedOrNotProvisioned,
    /// The app lacks a required NFC entitlement or usage-description declaration.
    SecurityRequirementsUnsatisfied,
    /// The NFC radio is disabled.
    RadioDisabled,
    /// The operating system rejected access to the reader session.
    AccessDenied,
    /// The app or device is not eligible to start the reader session.
    Ineligible,
    /// The user cancelled the reader session or the caller cancelled it.
    Cancelled,
    /// The system ended the reader session after its time limit.
    TimedOut,
    /// The system terminated the reader session when the app was interrupted or backgrounded.
    Interrupted,
    /// Another app or session currently owns the NFC reader.
    Busy,
    /// The reader session produced no NDEF message or an unclassified native failure.
    NativeFailure,
    /// The caller attempted to read a one-shot backend more than once.
    AlreadyStarted,
}

/// A backend that owns one NDEF read session and its native lifecycle.
pub trait NdefReadBackend {
    /// The one-shot asynchronous result, which starts native work when first polled.
    type ReadFuture<'a>: Future<Output = Result<NdefMessage, NdefReadError>> + 'a
    where
        Self: 'a;

    /// Reads the first NDEF message without requiring a framework-owned executor.
    ///
    /// Dropping a pending future cancels its active native session when the backend supports
    /// cancellation.
    fn read(&mut self) -> Self::ReadFuture<'_>;
}

/// A statically dispatched facade for one caller-owned NDEF reader session.
pub struct NdefReader<B> {
    backend: B,
}

impl<B: NdefReadBackend> NdefReader<B> {
    /// Creates the facade without starting tag polling or prompting the user.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reads the first NDEF message; dropping the pending future cancels the session.
    pub fn read(&mut self) -> B::ReadFuture<'_> {
        self.backend.read()
    }

    /// Borrows the selected backend for platform-specific controls.
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
    use alloc::vec;

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

    #[test]
    fn tnf_round_trip_preserves_reserved_values() {
        for raw in 0..=u8::MAX {
            assert_eq!(NdefTypeNameFormat::from_raw(raw).as_raw(), raw);
        }
    }

    #[test]
    fn ndef_message_preserves_record_order_and_opaque_bytes() {
        let first = NdefRecord::new(
            NdefTypeNameFormat::WellKnown,
            b"T".to_vec(),
            vec![0, 0xff],
            vec![0, 0xff, b'x'],
        );
        let second = NdefRecord::new(
            NdefTypeNameFormat::Unknown(0x80),
            vec![0xfe],
            Vec::new(),
            vec![0x80, 0],
        );
        let message = NdefMessage::new(vec![first.clone(), second.clone()]);

        assert_eq!(message.records(), &[first, second]);
        assert_eq!(message.clone().into_records(), message.records());
        assert_eq!(message.records()[0].payload(), &[0, 0xff, b'x']);
        assert_eq!(message.records()[1].format().as_raw(), 0x80);
    }

    struct FakeNdefBackend {
        started: bool,
        result: Option<Result<NdefMessage, NdefReadError>>,
    }

    impl NdefReadBackend for FakeNdefBackend {
        type ReadFuture<'a>
            = core::future::Ready<Result<NdefMessage, NdefReadError>>
        where
            Self: 'a;

        fn read(&mut self) -> Self::ReadFuture<'_> {
            if self.started {
                core::future::ready(Err(NdefReadError::AlreadyStarted))
            } else {
                self.started = true;
                core::future::ready(self.result.take().expect("fixture result"))
            }
        }
    }

    struct NoopWake;

    impl std::task::Wake for NoopWake {
        fn wake(self: alloc::sync::Arc<Self>) {}
    }

    fn poll_once<F: Future + Unpin>(future: &mut F) -> core::task::Poll<F::Output> {
        let waker = std::task::Waker::from(alloc::sync::Arc::new(NoopWake));
        let mut context = core::task::Context::from_waker(&waker);
        core::pin::Pin::new(future).poll(&mut context)
    }

    #[test]
    fn ndef_reader_forwards_async_result_without_framework_runtime() {
        let result = Err(NdefReadError::AccessDenied);
        let mut reader = NdefReader::new(FakeNdefBackend {
            started: false,
            result: Some(result.clone()),
        });

        let mut first_read = reader.read();
        assert_eq!(poll_once(&mut first_read), core::task::Poll::Ready(result));
        drop(first_read);
        let mut second_read = reader.read();
        assert_eq!(
            poll_once(&mut second_read),
            core::task::Poll::Ready(Err(NdefReadError::AlreadyStarted))
        );
        assert!(reader.into_backend().started);
    }
}
