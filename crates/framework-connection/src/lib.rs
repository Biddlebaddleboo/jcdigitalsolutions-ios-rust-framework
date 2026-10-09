#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable secure TCP byte-stream values and a runtime-neutral static backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::num::NonZeroU16;
use core::num::NonZeroU32;

/// Maximum byte length accepted for one send or receive chunk.
pub const MAX_CHUNK_BYTES: usize = 1_048_576;

/// An outbound host-and-port endpoint for a secure TCP connection.
///
/// The host is ASCII text, either a DNS name or an IP-address string. The value is not parsed,
/// canonicalized, resolved, or converted to IDNA by this portable type. Native resolution and
/// endpoint validation remain backend behavior.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Endpoint {
    host: String,
    port: NonZeroU16,
}

impl Endpoint {
    /// Creates an endpoint after checking host text and rejecting port zero.
    pub fn new(host: &str, port: u16) -> Result<Self, ConnectionError> {
        if host.is_empty()
            || !host.is_ascii()
            || host.bytes().any(|byte| {
                byte == 0
                    || byte <= 0x20
                    || byte == 0x7f
                    || matches!(byte, b'/' | b'\\' | b'?' | b'#' | b'@' | b'[' | b']')
            })
        {
            return Err(ConnectionError::InvalidEndpoint);
        }
        let port = NonZeroU16::new(port).ok_or(ConnectionError::InvalidEndpoint)?;
        Ok(Self {
            host: String::from(host),
            port,
        })
    }

    /// Returns the caller's original host/IP text.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the validated nonzero port.
    pub const fn port(&self) -> NonZeroU16 {
        self.port
    }
}

/// The raw native error domain and code returned by a connection backend.
///
/// Both values are preserved without translation. Their meaning is backend-specific; for the
/// iOS Network.framework backend, the domain is the raw `nw_error_domain_t` value and the code is
/// the raw `nw_error_get_error_code` result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NativeConnectionError {
    domain: i32,
    code: i32,
}

impl NativeConnectionError {
    /// Creates a native error without changing either raw value.
    pub const fn new(domain: i32, code: i32) -> Self {
        Self { domain, code }
    }

    /// Returns the raw backend-native error domain.
    pub const fn domain(self) -> i32 {
        self.domain
    }

    /// Returns the raw backend-native error code.
    pub const fn code(self) -> i32 {
        self.code
    }
}

/// An endpoint, operation-state, cancellation, or native backend error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ConnectionError {
    /// The host is empty, non-ASCII, contains whitespace/control bytes, or the port is zero.
    InvalidEndpoint,
    /// A receive backend returned an empty chunk with [`ReadTerminal::More`].
    InvalidReadChunk,
    /// A send, receive request, or returned chunk exceeds [`MAX_CHUNK_BYTES`].
    ChunkTooLarge,
    /// An operation was requested before connect completed or after the stream became terminal.
    InvalidState,
    /// The stream or operation was cancelled.
    Cancelled,
    /// The backend could not allocate bounded native or owned state.
    ResourceExhausted,
    /// The backend could not copy a receive chunk; an optional native error is preserved.
    ReceiveDataUnavailable(Option<NativeConnectionError>),
    /// The backend reported a failure without a native error value.
    BackendFailure,
    /// The backend reported a native error domain and code.
    Native(NativeConnectionError),
}

/// The terminal state attached to one bounded receive result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ReadTerminal {
    /// This chunk has data, and the stream may provide more bytes in a later receive.
    More,
    /// The peer closed its sending direction; the local sending direction may remain open.
    EndOfStream,
    /// A receive failed. `ReadChunk::bytes` still contains any bytes delivered with the error.
    Error(ConnectionError),
}

/// Owned bytes from one bounded stream receive, including its terminal outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadChunk {
    bytes: Vec<u8>,
    terminal: ReadTerminal,
}

impl ReadChunk {
    /// Creates one chunk and its explicit continuation, EOF, or error outcome.
    pub fn new(bytes: Vec<u8>, terminal: ReadTerminal) -> Result<Self, ConnectionError> {
        if bytes.len() > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        if bytes.is_empty() && terminal == ReadTerminal::More {
            return Err(ConnectionError::InvalidReadChunk);
        }
        Ok(Self { bytes, terminal })
    }

    /// Borrows the owned bytes received in this chunk.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consumes the chunk and returns its owned bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns whether more bytes may arrive, the peer sent EOF, or the receive failed.
    pub const fn terminal(&self) -> ReadTerminal {
        self.terminal
    }
}

/// A statically selected backend for one outbound secure TCP byte stream.
///
/// The backend must not begin native work before the future returned by `connect` is first
/// polled. `connect` completes only when the connection is ready or fails; waiting/preparing
/// states may remain pending indefinitely. This contract has no implicit timeout or executor.
///
/// Operations borrow the backend mutably, so callers cannot overlap send and receive operations
/// in this first contract. One send or receive is at most [`MAX_CHUNK_BYTES`] bytes; an empty
/// send is a successful no-op. Backend implementations must enforce these rules even when called
/// directly, outside [`Connection`]. One receive returns one owned bounded chunk; TCP provides no
/// message boundaries. A receive error may accompany bytes and must be represented in
/// `ReadTerminal::Error` without discarding those bytes. Empty `More` chunks are not valid.
///
/// A successful send means only that the selected backend accepted/processed the bytes according
/// to its native send completion. It does not mean the peer received or acknowledged them. A
/// failed or cancelled send may have transmitted an unknown prefix. Dropping an active operation
/// future must detach its waker and request cancellation of the whole stream; bytes already sent
/// cannot be recalled. `cancel` is idempotent and nonblocking. Dropping the facade also calls
/// `cancel`; native callback storage must remain alive until asynchronous cancellation is complete.
pub trait ConnectionBackend {
    /// The future for lazily establishing one secure TCP connection.
    type ConnectFuture<'a>: Future<Output = Result<(), ConnectionError>> + 'a
    where
        Self: 'a;

    /// The future for one bounded native send.
    type SendFuture<'a>: Future<Output = Result<(), ConnectionError>> + 'a
    where
        Self: 'a;

    /// The future for one bounded receive.
    type ReceiveFuture<'a>: Future<Output = Result<ReadChunk, ConnectionError>> + 'a
    where
        Self: 'a;

    /// The future that resolves after asynchronous native close/cancellation completes.
    type CloseFuture<'a>: Future<Output = Result<(), ConnectionError>> + 'a
    where
        Self: 'a;

    /// Creates a lazy connection request; native work starts on the returned future's first poll.
    fn connect<'a>(&'a mut self, endpoint: Endpoint) -> Self::ConnectFuture<'a>;

    /// Sends one ordered byte slice and resolves on backend send completion, not peer receipt.
    fn send<'a>(&'a mut self, bytes: &'a [u8]) -> Self::SendFuture<'a>;

    /// Receives at most `max_bytes` into an owned chunk and preserves EOF/data/error together.
    fn receive<'a>(&'a mut self, max_bytes: NonZeroU32) -> Self::ReceiveFuture<'a>;

    /// Requests close and resolves after the backend's asynchronous cancellation boundary.
    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a>;

    /// Detaches user-visible operations and requests nonblocking, idempotent whole-stream cancel.
    fn cancel(&mut self);
}

/// A thin facade over caller-owned, statically selected connection-backend state.
pub struct Connection<B: ConnectionBackend> {
    backend: B,
}

impl<B: ConnectionBackend> Connection<B> {
    /// Creates a facade around an explicitly supplied backend without starting native work.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Starts one lazy outbound secure TCP connection when the returned future is polled.
    pub fn connect(&mut self, endpoint: Endpoint) -> B::ConnectFuture<'_> {
        self.backend.connect(endpoint)
    }

    /// Sends one byte slice; success is not remote acknowledgement.
    pub async fn send<'a>(&'a mut self, bytes: &'a [u8]) -> Result<(), ConnectionError> {
        if bytes.len() > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        if bytes.is_empty() {
            return Ok(());
        }
        self.backend.send(bytes).await
    }

    /// Receives at most the nonzero requested bound into owned bytes.
    pub async fn receive(&mut self, max_bytes: NonZeroU32) -> Result<ReadChunk, ConnectionError> {
        if max_bytes.get() as usize > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        let chunk = self.backend.receive(max_bytes).await?;
        if chunk.bytes.len() > MAX_CHUNK_BYTES {
            return Err(ConnectionError::ChunkTooLarge);
        }
        Ok(chunk)
    }

    /// Closes the stream and waits for the asynchronous cancellation boundary.
    pub fn close(&mut self) -> B::CloseFuture<'_> {
        self.backend.close()
    }

    /// Requests nonblocking cancellation without waiting for native callbacks to finish.
    pub fn cancel(&mut self) {
        self.backend.cancel();
    }
}

impl<B: ConnectionBackend> Drop for Connection<B> {
    fn drop(&mut self) {
        self.backend.cancel();
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::VecDeque;
    use alloc::rc::Rc;
    use core::marker::PhantomData;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};
    use std::boxed::Box;
    use std::cell::RefCell;

    #[derive(Default)]
    struct FakeState {
        connect_polls: usize,
        cancel_count: usize,
        close_polls: usize,
        sent: Vec<Vec<u8>>,
        receive_results: VecDeque<ReadChunk>,
        receive_calls: usize,
    }

    struct FakeBackend {
        state: Rc<RefCell<FakeState>>,
    }

    impl FakeBackend {
        fn new() -> (Self, Rc<RefCell<FakeState>>) {
            let state = Rc::new(RefCell::new(FakeState::default()));
            (
                Self {
                    state: Rc::clone(&state),
                },
                state,
            )
        }
    }

    struct FakeFuture<'a, T> {
        state: Rc<RefCell<FakeState>>,
        result: Option<T>,
        pending: bool,
        started: bool,
        finished: bool,
        start: fn(&mut FakeState),
        _backend: PhantomData<&'a mut FakeBackend>,
    }

    impl<'a, T> FakeFuture<'a, T> {
        fn ready(state: Rc<RefCell<FakeState>>, result: T, start: fn(&mut FakeState)) -> Self {
            Self {
                state,
                result: Some(result),
                pending: false,
                started: false,
                finished: false,
                start,
                _backend: PhantomData,
            }
        }

        fn pending(state: Rc<RefCell<FakeState>>, start: fn(&mut FakeState)) -> Self {
            Self {
                state,
                result: None,
                pending: true,
                started: false,
                finished: false,
                start,
                _backend: PhantomData,
            }
        }
    }

    impl<T: Unpin> Future for FakeFuture<'_, T> {
        type Output = T;

        fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            if !this.started {
                (this.start)(&mut this.state.borrow_mut());
                this.started = true;
            }
            if this.pending {
                return Poll::Pending;
            }
            this.finished = true;
            Poll::Ready(this.result.take().expect("ready fake future has a result"))
        }
    }

    impl<T> Drop for FakeFuture<'_, T> {
        fn drop(&mut self) {
            if self.started && !self.finished && self.pending {
                self.state.borrow_mut().cancel_count += 1;
            }
        }
    }

    fn count_connect(state: &mut FakeState) {
        state.connect_polls += 1;
    }

    fn count_close(state: &mut FakeState) {
        state.close_polls += 1;
    }

    fn no_op(_: &mut FakeState) {}

    impl ConnectionBackend for FakeBackend {
        type ConnectFuture<'a> = FakeFuture<'a, Result<(), ConnectionError>>;
        type SendFuture<'a> = FakeFuture<'a, Result<(), ConnectionError>>;
        type ReceiveFuture<'a> = FakeFuture<'a, Result<ReadChunk, ConnectionError>>;
        type CloseFuture<'a> = FakeFuture<'a, Result<(), ConnectionError>>;

        fn connect<'a>(&'a mut self, _endpoint: Endpoint) -> Self::ConnectFuture<'a> {
            FakeFuture::ready(Rc::clone(&self.state), Ok(()), count_connect)
        }

        fn send<'a>(&'a mut self, bytes: &'a [u8]) -> Self::SendFuture<'a> {
            self.state.borrow_mut().sent.push(bytes.to_vec());
            FakeFuture::ready(Rc::clone(&self.state), Ok(()), no_op)
        }

        fn receive<'a>(&'a mut self, _max_bytes: NonZeroU32) -> Self::ReceiveFuture<'a> {
            self.state.borrow_mut().receive_calls += 1;
            let result = self
                .state
                .borrow_mut()
                .receive_results
                .pop_front()
                .ok_or(ConnectionError::BackendFailure);
            FakeFuture::ready(Rc::clone(&self.state), result, no_op)
        }

        fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
            FakeFuture::ready(Rc::clone(&self.state), Ok(()), count_close)
        }

        fn cancel(&mut self) {
            self.state.borrow_mut().cancel_count += 1;
        }
    }

    fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        let mut context = Context::from_waker(Waker::noop());
        future.poll(&mut context)
    }

    #[test]
    fn endpoint_checks_port_and_c_string_safe_host_text() {
        assert_eq!(
            Endpoint::new("", 443),
            Err(ConnectionError::InvalidEndpoint)
        );
        assert_eq!(
            Endpoint::new("example.com", 0),
            Err(ConnectionError::InvalidEndpoint)
        );
        assert_eq!(
            Endpoint::new("example\0.com", 443),
            Err(ConnectionError::InvalidEndpoint)
        );
        assert_eq!(
            Endpoint::new("example.com/path", 443),
            Err(ConnectionError::InvalidEndpoint)
        );
        assert_eq!(
            Endpoint::new("münich.example", 443),
            Err(ConnectionError::InvalidEndpoint)
        );
    }

    #[test]
    fn endpoint_preserves_host_and_nonzero_port() {
        let endpoint = Endpoint::new("2001:db8::1", 443).unwrap();
        assert_eq!(endpoint.host(), "2001:db8::1");
        assert_eq!(endpoint.port().get(), 443);
    }

    #[test]
    fn connect_native_work_begins_on_first_future_poll() {
        let (backend, state) = FakeBackend::new();
        let mut connection = Connection::new(backend);
        let endpoint = Endpoint::new("example.com", 443).unwrap();
        let mut future = Box::pin(connection.connect(endpoint));
        assert_eq!(state.borrow().connect_polls, 0);
        assert_eq!(poll_once(future.as_mut()), Poll::Ready(Ok(())));
        assert_eq!(state.borrow().connect_polls, 1);
    }

    #[test]
    fn pending_operation_drop_requests_cancel() {
        let (_, state) = FakeBackend::new();
        let mut future = Box::pin(FakeFuture::<()>::pending(Rc::clone(&state), count_connect));
        assert_eq!(poll_once(future.as_mut()), Poll::Pending);
        assert_eq!(state.borrow().connect_polls, 1);
        drop(future);
        assert_eq!(state.borrow().cancel_count, 1);
    }

    #[test]
    fn read_chunk_preserves_bytes_with_native_error_domain_and_code() {
        let native = NativeConnectionError::new(3, -9807);
        let chunk = ReadChunk::new(
            b"partial".to_vec(),
            ReadTerminal::Error(ConnectionError::Native(native)),
        )
        .unwrap();
        assert_eq!(chunk.bytes(), b"partial");
        assert_eq!(
            chunk.terminal(),
            ReadTerminal::Error(ConnectionError::Native(native))
        );
        assert_eq!(native.domain(), 3);
        assert_eq!(native.code(), -9807);
    }

    #[test]
    fn empty_more_chunk_is_rejected() {
        assert_eq!(
            ReadChunk::new(Vec::new(), ReadTerminal::More),
            Err(ConnectionError::InvalidReadChunk)
        );
    }

    #[test]
    fn chunk_limit_accepts_boundary_and_rejects_oversize() {
        assert!(
            ReadChunk::new(alloc::vec![0; MAX_CHUNK_BYTES], ReadTerminal::EndOfStream,).is_ok()
        );
        assert_eq!(
            ReadChunk::new(
                alloc::vec![0; MAX_CHUNK_BYTES + 1],
                ReadTerminal::EndOfStream,
            ),
            Err(ConnectionError::ChunkTooLarge)
        );
    }

    #[test]
    fn facade_empty_send_is_noop_and_oversize_requests_do_not_reach_backend() {
        let (backend, state) = FakeBackend::new();
        let mut connection = Connection::new(backend);

        let mut empty_send = Box::pin(connection.send(&[]));
        assert_eq!(poll_once(empty_send.as_mut()), Poll::Ready(Ok(())));
        drop(empty_send);
        assert!(state.borrow().sent.is_empty());

        let oversized = alloc::vec![0; MAX_CHUNK_BYTES + 1];
        let mut send = Box::pin(connection.send(&oversized));
        assert_eq!(
            poll_once(send.as_mut()),
            Poll::Ready(Err(ConnectionError::ChunkTooLarge))
        );
        drop(send);
        assert!(state.borrow().sent.is_empty());

        let mut receive =
            Box::pin(connection.receive(NonZeroU32::new((MAX_CHUNK_BYTES + 1) as u32).unwrap()));
        assert_eq!(
            poll_once(receive.as_mut()),
            Poll::Ready(Err(ConnectionError::ChunkTooLarge))
        );
        assert_eq!(state.borrow().receive_calls, 0);
    }

    #[test]
    fn eof_is_distinct_and_owned_read_bytes_survive_facade() {
        let (backend, state) = FakeBackend::new();
        state
            .borrow_mut()
            .receive_results
            .push_back(ReadChunk::new(b"last bytes".to_vec(), ReadTerminal::EndOfStream).unwrap());
        let mut connection = Connection::new(backend);
        let mut future = Box::pin(connection.receive(NonZeroU32::new(32).unwrap()));
        let Poll::Ready(Ok(chunk)) = poll_once(future.as_mut()) else {
            panic!("fake receive is immediately ready");
        };
        assert_eq!(chunk.into_bytes(), b"last bytes");
        assert_eq!(state.borrow().cancel_count, 0);
    }

    #[test]
    fn successful_send_and_close_are_explicit_backend_operations() {
        let (backend, state) = FakeBackend::new();
        let mut connection = Connection::new(backend);
        let mut send = Box::pin(connection.send(b"request"));
        assert_eq!(poll_once(send.as_mut()), Poll::Ready(Ok(())));
        drop(send);
        assert_eq!(state.borrow().sent, [b"request".to_vec()]);
        let mut close = Box::pin(connection.close());
        assert_eq!(poll_once(close.as_mut()), Poll::Ready(Ok(())));
        assert_eq!(state.borrow().close_polls, 1);
    }

    #[test]
    fn dropping_connection_requests_cancel_once_from_facade() {
        let (backend, state) = FakeBackend::new();
        let connection = Connection::new(backend);
        drop(connection);
        assert_eq!(state.borrow().cancel_count, 1);
    }
}
