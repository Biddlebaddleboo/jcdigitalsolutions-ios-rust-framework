extern crate alloc;

use alloc::{rc::Rc, vec::Vec};
use core::{
    cell::RefCell,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

use block2::RcBlock;
use dispatch2::DispatchQueue;
use framework_nfc::{NdefMessage, NdefRecord, NdefWriteBackend, NdefWriteError};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
    rc::{Retained, Weak},
    runtime::{NSObject, ProtocolObject},
};
use objc2_core_nfc::{
    NFCErrorDomain, NFCNDEFMessage, NFCNDEFPayload, NFCNDEFReaderSession,
    NFCNDEFReaderSessionDelegate, NFCNDEFStatus, NFCNDEFTag, NFCReaderSession,
    NFCReaderSessionProtocol, NFCTypeNameFormat,
};
use objc2_foundation::{NSArray, NSData, NSError, NSObjectProtocol, NSString};

use crate::write_state::{
    NdefWriteState, map_core_nfc_write_error, message_exceeds_capacity, tag_status_error,
};

type SharedWriteOperation = Rc<RefCell<NdefWriteOperation>>;

struct NdefWriteOperation {
    state: NdefWriteState,
    message: Option<Retained<NFCNDEFMessage>>,
    tag: Option<Retained<ProtocolObject<dyn objc2_core_nfc::NFCNDEFTag>>>,
    encoded_length: u32,
}

impl NdefWriteOperation {
    const fn new() -> Self {
        Self {
            state: NdefWriteState::new(),
            message: None,
            tag: None,
            encoded_length: 0,
        }
    }
}

struct NdefWriteDelegateIvars {
    operation: SharedWriteOperation,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosNfcNdefWriterDelegate"]
    #[ivars = NdefWriteDelegateIvars]
    struct NdefWriteDelegate;

    // SAFETY: NSObjectProtocol adds no required methods to NSObject.
    unsafe impl NSObjectProtocol for NdefWriteDelegate {}

    // SAFETY: The session uses the main queue and the delegate is main-thread-only. Every
    // Objective-C callback contains Rust panics before they cross the foreign ABI.
    #[allow(non_snake_case)]
    unsafe impl NFCNDEFReaderSessionDelegate for NdefWriteDelegate {
        #[unsafe(method(readerSession:didInvalidateWithError:))]
        unsafe fn readerSession_didInvalidateWithError(
            &self,
            _session: &NFCNDEFReaderSession,
            error: &NSError,
        ) {
            let operation = self.ivars().operation.clone();
            if catch_unwind(AssertUnwindSafe(|| {
                let native_domain = error.domain();
                let is_core_nfc = native_domain.isEqualToString(unsafe { NFCErrorDomain });
                let result = Err(map_core_nfc_write_error(is_core_nfc, error.code() as i64));
                settle(&operation, result, None);
            }))
            .is_err()
            {
                settle(&operation, Err(NdefWriteError::NativeFailure), None);
            }
        }

        #[unsafe(method(readerSession:didDetectNDEFs:))]
        unsafe fn readerSession_didDetectNDEFs(
            &self,
            session: &NFCNDEFReaderSession,
            _messages: &NSArray<NFCNDEFMessage>,
        ) {
            let operation = self.ivars().operation.clone();
            let weak_session = Weak::from(session);
            run_native_callback(&operation, &weak_session, || {
                settle(
                    &operation,
                    Err(NdefWriteError::NativeFailure),
                    Some(&weak_session),
                );
            });
        }

        #[unsafe(method(readerSession:didDetectTags:))]
        unsafe fn readerSession_didDetectTags(
            &self,
            session: &NFCNDEFReaderSession,
            tags: &NSArray<ProtocolObject<dyn objc2_core_nfc::NFCNDEFTag>>,
        ) {
            let operation = self.ivars().operation.clone();
            let weak_session = Weak::from(session);
            run_native_callback(&operation, &weak_session, || {
                if tags.is_empty() {
                    return;
                }

                if !operation.borrow_mut().state.begin_connect() {
                    return;
                }

                let tag = tags.objectAtIndex(0).retain();
                operation.borrow_mut().tag = Some(tag.clone());
                let connection = RcBlock::new({
                    let operation = operation.clone();
                    let weak_session = weak_session.clone();
                    move |native_error: *mut NSError| {
                        run_native_callback(&operation, &weak_session, || {
                            if let Some(error) = native_error_result(native_error) {
                                settle(&operation, Err(error), Some(&weak_session));
                                return;
                            }
                            if !operation.borrow_mut().state.begin_status_query() {
                                return;
                            }

                            let tag = { operation.borrow().tag.clone() };
                            let Some(tag) = tag else {
                                settle(
                                    &operation,
                                    Err(NdefWriteError::NativeFailure),
                                    Some(&weak_session),
                                );
                                return;
                            };
                            let query = RcBlock::new({
                                let operation = operation.clone();
                                let weak_session = weak_session.clone();
                                move |status: NFCNDEFStatus,
                                      capacity: usize,
                                      native_error: *mut NSError| {
                                    run_native_callback(&operation, &weak_session, || {
                                        if let Some(error) = native_error_result(native_error) {
                                            settle(&operation, Err(error), Some(&weak_session));
                                            return;
                                        }

                                        let raw_status = u8::try_from(status.0).unwrap_or(u8::MAX);
                                        if let Some(error) = tag_status_error(raw_status) {
                                            settle(&operation, Err(error), Some(&weak_session));
                                            return;
                                        }

                                        let prepared_message = {
                                            let operation_ref = operation.borrow();
                                            operation_ref.message.as_ref().map(|message| {
                                                (operation_ref.encoded_length, message.clone())
                                            })
                                        };
                                        let Some((encoded_length, message)) = prepared_message
                                        else {
                                            settle(
                                                &operation,
                                                Err(NdefWriteError::NativeFailure),
                                                Some(&weak_session),
                                            );
                                            return;
                                        };
                                        if message_exceeds_capacity(encoded_length, capacity) {
                                            settle(
                                                &operation,
                                                Err(NdefWriteError::MessageTooLarge),
                                                Some(&weak_session),
                                            );
                                            return;
                                        }

                                        let tag = { operation.borrow().tag.clone() };
                                        let Some(tag) = tag else {
                                            settle(
                                                &operation,
                                                Err(NdefWriteError::NativeFailure),
                                                Some(&weak_session),
                                            );
                                            return;
                                        };
                                        if !operation.borrow_mut().state.begin_write() {
                                            return;
                                        }

                                        let completion = RcBlock::new({
                                            let operation = operation.clone();
                                            let weak_session = weak_session.clone();
                                            move |native_error: *mut NSError| {
                                                run_native_callback(
                                                    &operation,
                                                    &weak_session,
                                                    || {
                                                        let result =
                                                            native_error_result(native_error)
                                                                .map_or(Ok(()), Err);
                                                        settle(
                                                            &operation,
                                                            result,
                                                            Some(&weak_session),
                                                        );
                                                    },
                                                );
                                            }
                                        });
                                        // SAFETY: The tag is connected, the NDEF message is owned
                                        // for the full async operation, and Core NFC copies its
                                        // record data. Completion is dispatched on the main queue.
                                        unsafe {
                                            tag.writeNDEF_completionHandler(&message, &completion)
                                        };
                                    });
                                }
                            });
                            // SAFETY: The connected NDEF tag remains owned by the active session;
                            // the query callback is retained by Core NFC until it completes.
                            unsafe { tag.queryNDEFStatusWithCompletionHandler(&query) };
                        });
                    }
                });
                // SAFETY: The NDEF tag comes from this active session and the completion block
                // is retained by Core NFC until connection completes.
                unsafe { session.connectToTag_completionHandler(&tag, &connection) };
            });
        }
    }
);

impl NdefWriteDelegate {
    fn new(main_thread: MainThreadMarker, operation: SharedWriteOperation) -> Retained<Self> {
        let this = Self::alloc(main_thread).set_ivars(NdefWriteDelegateIvars { operation });
        // SAFETY: NSObject is the direct superclass, and all ivars are initialized before init.
        unsafe { msg_send![super(this), init] }
    }
}

/// An iOS one-shot NDEF writer backend that uses public Core NFC read-write sessions.
///
/// The caller must include a non-empty NFCReaderUsageDescription and the
/// com.apple.developer.nfc.readersession.formats entitlement with NDEF enabled. Construction
/// creates an inert read-write session; the future from NdefWriteBackend::write begins polling on
/// first poll. The first detected NDEF tag is connected, checked for writable status and capacity,
/// and receives the message only when both checks pass.
pub struct CoreNfcNdefWriteBackend {
    session: Retained<NFCNDEFReaderSession>,
    delegate: Retained<NdefWriteDelegate>,
    operation: SharedWriteOperation,
    _main_thread: PhantomData<MainThreadMarker>,
}

impl CoreNfcNdefWriteBackend {
    /// Creates an inert one-shot NDEF write session on the iOS main thread.
    ///
    /// Returns NdefWriteError::Unsupported when Core NFC reports no reader support. Provisioning,
    /// usage-description, entitlement, and user/system access failures arrive asynchronously.
    pub fn new(main_thread: MainThreadMarker, alert_message: &str) -> Result<Self, NdefWriteError> {
        // SAFETY: This readonly class property does not create a session, prompt, poll, or access
        // tag data.
        if !unsafe { NFCReaderSession::readingAvailable() } {
            return Err(NdefWriteError::Unsupported);
        }

        let operation = Rc::new(RefCell::new(NdefWriteOperation::new()));
        let delegate = NdefWriteDelegate::new(main_thread, operation.clone());
        let protocol_delegate = ProtocolObject::from_ref(&*delegate);
        // SAFETY: The backend strongly retains its main-thread delegate. The optional
        // didDetectTags callback enables read-write behavior and requires automatic first-read
        // invalidation to remain disabled.
        let session = unsafe {
            NFCNDEFReaderSession::initWithDelegate_queue_invalidateAfterFirstRead(
                NFCNDEFReaderSession::alloc(),
                protocol_delegate,
                Some(DispatchQueue::main()),
                false,
            )
        };
        let alert_message = NSString::from_str(alert_message);
        // SAFETY: The new session copies this display value before polling starts.
        unsafe { session.setAlertMessage(&alert_message) };

        Ok(Self {
            session,
            delegate,
            operation,
            _main_thread: PhantomData,
        })
    }

    fn begin(&mut self, message: Retained<NFCNDEFMessage>, encoded_length: u32) {
        {
            let mut operation = self.operation.borrow_mut();
            if operation.state.start().is_err() {
                return;
            }
            operation.message = Some(message);
            operation.encoded_length = encoded_length;
        }
        // SAFETY: This main-thread-owned session is inert, and the operation state now records
        // its sole start before Core NFC may synchronously deliver callbacks.
        unsafe { self.session.beginSession() };
    }

    fn poll_result(&mut self, context: &Context<'_>) -> Poll<Result<(), NdefWriteError>> {
        self.operation.borrow_mut().state.poll_result(context)
    }

    fn cancel_active(&mut self) {
        let cancelled = {
            let mut operation = self.operation.borrow_mut();
            let (cancelled, _waker) = operation.state.cancel();
            if cancelled {
                operation.message.take();
                operation.tag.take();
            }
            cancelled
        };
        if cancelled {
            // SAFETY: Cancellation runs on the main thread, and the active session is owned by
            // this backend.
            unsafe { self.session.invalidateSession() };
        }
    }
}

impl NdefWriteBackend for CoreNfcNdefWriteBackend {
    type WriteFuture<'a>
        = CoreNfcNdefWriteFuture<'a>
    where
        Self: 'a;

    fn write(&mut self, message: NdefMessage) -> Self::WriteFuture<'_> {
        let prepared = encode_native_message(message);
        CoreNfcNdefWriteFuture {
            backend: self,
            prepared: Some(prepared),
            started: false,
            completed: false,
        }
    }
}

impl Drop for CoreNfcNdefWriteBackend {
    fn drop(&mut self) {
        if self.operation.borrow().state.is_active() {
            // SAFETY: MainThreadMarker makes the backend thread-affine. Invalidation precedes
            // release of the strongly held delegate.
            unsafe { self.session.invalidateSession() };
        }
        let _ = &self.delegate;
    }
}

/// The future returned by NdefWriteBackend::write for the Core NFC writer.
///
/// Poll it on the iOS main thread. Dropping it while pending invalidates the active native
/// session.
pub struct CoreNfcNdefWriteFuture<'a> {
    backend: &'a mut CoreNfcNdefWriteBackend,
    prepared: Option<Result<(Retained<NFCNDEFMessage>, u32), NdefWriteError>>,
    started: bool,
    completed: bool,
}

impl Future for CoreNfcNdefWriteFuture<'_> {
    type Output = Result<(), NdefWriteError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        assert!(!this.completed, "NFC write future polled after completion");
        if !this.started {
            let Some(prepared) = this.prepared.take() else {
                this.completed = true;
                return Poll::Ready(Err(NdefWriteError::NativeFailure));
            };
            let (message, encoded_length) = match prepared {
                Ok(prepared) => prepared,
                Err(error) => {
                    this.completed = true;
                    return Poll::Ready(Err(error));
                }
            };
            this.started = true;
            this.backend.begin(message, encoded_length);
            if !this.backend.operation.borrow().state.is_active() {
                this.completed = true;
                return Poll::Ready(Err(NdefWriteError::AlreadyStarted));
            }
        }
        let result = this.backend.poll_result(context);
        if result.is_ready() {
            this.completed = true;
        }
        result
    }
}

impl Drop for CoreNfcNdefWriteFuture<'_> {
    fn drop(&mut self) {
        if self.started && !self.completed {
            self.backend.cancel_active();
        }
    }
}

fn encode_native_message(
    message: NdefMessage,
) -> Result<(Retained<NFCNDEFMessage>, u32), NdefWriteError> {
    let encoded_length = message.encoded_length()?;
    let records = message
        .into_records()
        .into_iter()
        .map(encode_native_record)
        .collect::<Result<Vec<_>, _>>()?;
    let native_records = NSArray::from_retained_slice(&records);
    // SAFETY: The array contains only records produced by the public payload initializer from
    // validated NDEF fields; Core NFC copies the array into the new message.
    let native_message =
        unsafe { NFCNDEFMessage::initWithNDEFRecords(NFCNDEFMessage::alloc(), &native_records) };
    // The native message length is the documented on-tag encoding length. Reject any unexpected
    // canonicalization so the portable byte-length contract remains exact.
    if unsafe { native_message.length() } != encoded_length as usize {
        return Err(NdefWriteError::InvalidMessage);
    }
    Ok((native_message, encoded_length))
}

fn encode_native_record(record: NdefRecord) -> Result<Retained<NFCNDEFPayload>, NdefWriteError> {
    let (format, record_type, identifier, payload) = record.into_parts();
    let native_type = NSData::with_bytes(&record_type);
    let native_identifier = NSData::with_bytes(&identifier);
    let native_payload = NSData::with_bytes(&payload);
    // SAFETY: The portable message validator checked TNF rules and field lengths; Core NFC copies
    // the supplied type, identifier, and payload data without text conversion.
    Ok(unsafe {
        NFCNDEFPayload::initWithFormat_type_identifier_payload(
            NFCNDEFPayload::alloc(),
            NFCTypeNameFormat(format.as_raw()),
            &native_type,
            &native_identifier,
            &native_payload,
        )
    })
}

fn native_error_result(native_error: *mut NSError) -> Option<NdefWriteError> {
    // SAFETY: Core NFC passes either null or a live NSError for the duration of its completion.
    let error = unsafe { native_error.as_ref() }?;
    let native_domain = error.domain();
    let is_core_nfc = native_domain.isEqualToString(unsafe { NFCErrorDomain });
    Some(map_core_nfc_write_error(is_core_nfc, error.code() as i64))
}

fn run_native_callback(
    operation: &SharedWriteOperation,
    session: &Weak<NFCNDEFReaderSession>,
    callback: impl FnOnce(),
) {
    if catch_unwind(AssertUnwindSafe(callback)).is_err() {
        settle(operation, Err(NdefWriteError::NativeFailure), Some(session));
    }
}

fn settle(
    operation: &SharedWriteOperation,
    result: Result<(), NdefWriteError>,
    session: Option<&Weak<NFCNDEFReaderSession>>,
) {
    let (settled, waker) = {
        let mut operation = operation.borrow_mut();
        let (settled, waker) = operation.state.settle(result);
        if settled {
            operation.message.take();
            operation.tag.take();
        }
        (settled, waker)
    };
    if settled {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // Callback closures hold only a weak session reference, avoiding a callback/session
            // retain cycle if Core NFC delays an operation completion.
            if let Some(session) = session.and_then(Weak::load) {
                // SAFETY: The session is active or already invalidating on the main queue.
                unsafe { session.invalidateSession() };
            }
        }));
    }
    if let Some(waker) = waker {
        let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
    }
}
