extern crate alloc;

use alloc::vec::Vec;
use core::{
    cell::{RefCell, RefMut},
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

use dispatch2::DispatchQueue;
use framework_nfc::{NdefMessage, NdefReadBackend, NdefReadError, NdefRecord, NdefTypeNameFormat};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Retained, autoreleasepool},
    runtime::{NSObject, ProtocolObject},
};
use objc2_core_nfc::{
    NFCErrorDomain, NFCNDEFMessage, NFCNDEFReaderSession, NFCNDEFReaderSessionDelegate,
    NFCReaderSession, NFCReaderSessionProtocol,
};
use objc2_foundation::{NSArray, NSError, NSObjectProtocol, NSString};

use crate::read_state::{NdefReadState, map_core_nfc_error};

struct NdefReaderDelegateIvars {
    state: RefCell<NdefReadState>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosNfcNdefReaderDelegate"]
    #[ivars = NdefReaderDelegateIvars]
    struct NdefReaderDelegate;

    // SAFETY: NSObjectProtocol adds no required methods to NSObject.
    unsafe impl NSObjectProtocol for NdefReaderDelegate {}

    // SAFETY: The backend supplies DispatchQueue::main; the delegate is main-thread-only, owns
    // no native references in its ivars, and contains all Rust panics before they cross Objective-C.
    #[allow(non_snake_case)]
    unsafe impl NFCNDEFReaderSessionDelegate for NdefReaderDelegate {
        #[unsafe(method(readerSession:didInvalidateWithError:))]
        unsafe fn readerSession_didInvalidateWithError(
            &self,
            _session: &NFCNDEFReaderSession,
            error: &NSError,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let native_domain = error.domain();
                let is_core_nfc = native_domain.isEqualToString(unsafe { NFCErrorDomain });
                let code = error.code() as i64;
                let result = Err(map_core_nfc_error(is_core_nfc, code));
                let wake = self.state().settle(result).1;
                if let Some(waker) = wake {
                    waker.wake();
                }
            }));
        }

        #[unsafe(method(readerSession:didDetectNDEFs:))]
        unsafe fn readerSession_didDetectNDEFs(
            &self,
            _session: &NFCNDEFReaderSession,
            messages: &NSArray<NFCNDEFMessage>,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    autoreleasepool(|_| first_message_from_native(messages))
                }))
                .unwrap_or(Err(NdefReadError::NativeFailure));
                let wake = self.state().settle(result).1;
                if let Some(waker) = wake {
                    waker.wake();
                }
            }));
        }
    }
);

impl NdefReaderDelegate {
    fn new(main_thread: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(main_thread).set_ivars(NdefReaderDelegateIvars {
            state: RefCell::new(NdefReadState::new()),
        });
        // SAFETY: NSObject is the direct superclass, and the ivars are initialized before init.
        unsafe { msg_send![super(this), init] }
    }

    fn state(&self) -> RefMut<'_, NdefReadState> {
        self.ivars().state.borrow_mut()
    }
}

/// An iOS one-shot NDEF reader backend using the public Core NFC session API.
///
/// The caller must include a non-empty `NFCReaderUsageDescription` value and the
/// `com.apple.developer.nfc.readersession.formats` entitlement with NDEF enabled. Construction
/// creates an inert session; the future from [`NdefReadBackend::read`] presents Apple's reader UI
/// on first poll. Core NFC reports provisioning/access failures asynchronously as
/// [`NdefReadError`] results.
pub struct CoreNfcNdefBackend {
    session: Retained<NFCNDEFReaderSession>,
    delegate: Retained<NdefReaderDelegate>,
    _main_thread: PhantomData<MainThreadMarker>,
}

impl CoreNfcNdefBackend {
    /// Creates an inert one-shot NDEF session on the iOS main thread.
    ///
    /// Returns [`NdefReadError::Unsupported`] when Core NFC reports no reader support. This
    /// property does not preflight permissions, provisioning, the usage description, or the
    /// reader-session entitlement; Core NFC reports those conditions when the session starts.
    pub fn new(main_thread: MainThreadMarker, alert_message: &str) -> Result<Self, NdefReadError> {
        // SAFETY: This readonly class property has no instance preconditions and does not start a
        // session, prompt, poll, or access tag data.
        if !unsafe { NFCReaderSession::readingAvailable() } {
            return Err(NdefReadError::Unsupported);
        }

        let delegate = NdefReaderDelegate::new(main_thread);
        let protocol_delegate = ProtocolObject::from_ref(&*delegate);
        // SAFETY: The delegate remains strongly retained in this backend for the full session
        // lifetime. The main dispatch queue matches the delegate's main-thread class and future
        // contract. First-read invalidation makes this an NDEF read-only session because the
        // delegate does not implement didDetectTags.
        let session = unsafe {
            NFCNDEFReaderSession::initWithDelegate_queue_invalidateAfterFirstRead(
                NFCNDEFReaderSession::alloc(),
                protocol_delegate,
                Some(DispatchQueue::main()),
                true,
            )
        };
        let alert_message = NSString::from_str(alert_message);
        // SAFETY: The session is newly created, valid, and not started. Core NFC copies this
        // display value before the system reader UI begins.
        unsafe { session.setAlertMessage(&alert_message) };

        Ok(Self {
            session,
            delegate,
            _main_thread: PhantomData,
        })
    }
}

impl NdefReadBackend for CoreNfcNdefBackend {
    type ReadFuture<'a>
        = CoreNfcNdefReadFuture<'a>
    where
        Self: 'a;

    fn read(&mut self) -> Self::ReadFuture<'_> {
        CoreNfcNdefReadFuture {
            backend: self,
            started: false,
            completed: false,
        }
    }
}

impl CoreNfcNdefBackend {
    fn begin(&mut self) -> Result<(), NdefReadError> {
        self.delegate.state().start()?;
        // SAFETY: The main-thread marker is retained by this backend. The session has not been
        // invalidated and can be started exactly once according to NdefReadState.
        unsafe { self.session.beginSession() };
        Ok(())
    }

    fn poll_result(&mut self, context: &Context<'_>) -> Poll<Result<NdefMessage, NdefReadError>> {
        self.delegate.state().poll_result(context)
    }

    fn cancel_active(&mut self) {
        let (cancelled, _waker) = self.delegate.state().cancel();
        if cancelled {
            // SAFETY: This session is active, this method runs on the construction thread, and
            // Core NFC documents invalidateSession as the read-only session close operation.
            unsafe { self.session.invalidateSession() };
        }
    }
}

/// The future returned by [`NdefReadBackend::read`] for an iOS Core NFC session.
///
/// Poll it on the iOS main thread. Dropping it while pending invalidates the native session.
pub struct CoreNfcNdefReadFuture<'a> {
    backend: &'a mut CoreNfcNdefBackend,
    started: bool,
    completed: bool,
}

impl Future for CoreNfcNdefReadFuture<'_> {
    type Output = Result<NdefMessage, NdefReadError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        assert!(!this.completed, "NFC read future polled after completion");
        if !this.started {
            if let Err(error) = this.backend.begin() {
                this.completed = true;
                return Poll::Ready(Err(error));
            }
            this.started = true;
        }
        let result = this.backend.poll_result(context);
        if result.is_ready() {
            this.completed = true;
        }
        result
    }
}

impl Drop for CoreNfcNdefReadFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.completed {
            self.backend.cancel_active();
        }
    }
}

impl Drop for CoreNfcNdefBackend {
    fn drop(&mut self) {
        if self.delegate.state().is_active() {
            // SAFETY: Drop runs on the construction thread because MainThreadMarker makes the
            // backend thread-affine. Invalidation precedes release of the strongly held delegate.
            unsafe { self.session.invalidateSession() };
        }
    }
}

fn first_message_from_native(
    messages: &NSArray<NFCNDEFMessage>,
) -> Result<NdefMessage, NdefReadError> {
    let message = messages.firstObject().ok_or(NdefReadError::NativeFailure)?;
    let native_records = unsafe { message.records() };
    let mut records = Vec::with_capacity(native_records.len());
    for index in 0..native_records.len() {
        let record = native_records.objectAtIndex(index);
        // SAFETY: The payload object's documented readonly properties are queried while the
        // retained NFCNDEFPayload remains alive. NSData::to_vec copies the exact byte ranges.
        let (format, record_type, identifier, payload) = unsafe {
            (
                record.typeNameFormat(),
                record.r#type().to_vec(),
                record.identifier().to_vec(),
                record.payload().to_vec(),
            )
        };
        records.push(NdefRecord::new(
            NdefTypeNameFormat::from_raw(format.0),
            record_type,
            identifier,
            payload,
        ));
    }
    Ok(NdefMessage::new(records))
}
