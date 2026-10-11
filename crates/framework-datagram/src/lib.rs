#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable IPv4 UDP datagram values and a runtime-neutral static backend contract."]

extern crate alloc;

use alloc::vec::Vec;

/// Maximum payload bytes in one IPv4 UDP datagram.
pub const MAX_DATAGRAM_PAYLOAD_BYTES: u32 = 65_507;

/// An IPv4 socket endpoint. Port zero is valid only when used as a local bind endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Ipv4Endpoint {
    address: [u8; 4],
    port: u16,
}

impl Ipv4Endpoint {
    /// Creates an endpoint from IPv4 address octets and a host-order UDP port.
    pub const fn new(address: [u8; 4], port: u16) -> Self {
        Self { address, port }
    }

    /// Returns the four IPv4 address octets in network order.
    pub const fn address(self) -> [u8; 4] {
        self.address
    }

    /// Returns the host-order UDP port.
    pub const fn port(self) -> u16 {
        self.port
    }

    /// Returns whether the address is `0.0.0.0`.
    pub const fn is_unspecified(self) -> bool {
        self.address[0] == 0 && self.address[1] == 0 && self.address[2] == 0 && self.address[3] == 0
    }

    /// Returns whether the address is the limited broadcast address `255.255.255.255`.
    pub const fn is_limited_broadcast(self) -> bool {
        self.address[0] == 255
            && self.address[1] == 255
            && self.address[2] == 255
            && self.address[3] == 255
    }

    /// Returns whether the address is in the IPv4 multicast range `224.0.0.0/4`.
    pub const fn is_multicast(self) -> bool {
        self.address[0] & 0xf0 == 0xe0
    }
}

/// Configuration applied before an IPv4 UDP socket binds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DatagramBindConfig {
    local_endpoint: Ipv4Endpoint,
    reuse_address: bool,
    broadcast: bool,
}

impl DatagramBindConfig {
    /// Creates a bind configuration with address reuse and broadcast disabled.
    pub const fn new(local_endpoint: Ipv4Endpoint) -> Self {
        Self {
            local_endpoint,
            reuse_address: false,
            broadcast: false,
        }
    }

    /// Returns a copy that enables or disables `SO_REUSEADDR` before bind.
    pub const fn with_reuse_address(mut self, enabled: bool) -> Self {
        self.reuse_address = enabled;
        self
    }

    /// Returns a copy that enables or disables `SO_BROADCAST` before bind.
    pub const fn with_broadcast(mut self, enabled: bool) -> Self {
        self.broadcast = enabled;
        self
    }

    /// Returns the requested local bind endpoint. Port zero asks the OS to choose a port.
    pub const fn local_endpoint(self) -> Ipv4Endpoint {
        self.local_endpoint
    }

    /// Returns whether `SO_REUSEADDR` is requested.
    pub const fn reuse_address(self) -> bool {
        self.reuse_address
    }

    /// Returns whether `SO_BROADCAST` is requested.
    pub const fn broadcast(self) -> bool {
        self.broadcast
    }
}

/// Wait policy for one receive operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReceiveWait {
    /// Return `WouldBlock` if no datagram is immediately available.
    NonBlocking,
    /// Wait up to the nonzero number of milliseconds, then return `TimedOut`.
    TimeoutMillis(core::num::NonZeroU32),
    /// Wait until a datagram arrives, the socket closes, or `cancel_receive` is called.
    UntilCancelled,
}

/// An owned IPv4 UDP datagram and its source endpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Datagram {
    source: Ipv4Endpoint,
    payload: Vec<u8>,
}

impl Datagram {
    /// Creates an owned datagram and rejects invalid source ports or oversized payloads.
    pub fn new(source: Ipv4Endpoint, payload: Vec<u8>) -> Result<Self, DatagramError> {
        if source.port() == 0 {
            return Err(DatagramError::InvalidEndpoint);
        }
        if payload.len() > MAX_DATAGRAM_PAYLOAD_BYTES as usize {
            return Err(DatagramError::PayloadTooLarge);
        }
        Ok(Self { source, payload })
    }

    /// Returns the source IPv4 endpoint.
    pub const fn source(&self) -> Ipv4Endpoint {
        self.source
    }

    /// Borrows the owned datagram payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Consumes the datagram and returns its owned payload.
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }
}

/// A validation, receive-state, cancellation, allocation, or native socket error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DatagramError {
    /// A destination has port zero or an unspecified address.
    InvalidEndpoint,
    /// The destination is multicast, which this capability does not join or send to.
    MulticastUnsupported,
    /// Limited broadcast was requested without enabling the socket broadcast option.
    BroadcastNotEnabled,
    /// A requested receive bound is zero or exceeds [`MAX_DATAGRAM_PAYLOAD_BYTES`].
    InvalidReceiveBound,
    /// An outbound or inbound datagram exceeds its allowed byte bound.
    PayloadTooLarge,
    /// A nonblocking receive found no datagram ready.
    WouldBlock,
    /// The finite receive deadline expired.
    TimedOut,
    /// The active receive was cancelled without closing the socket.
    Cancelled,
    /// The session is closed and cannot accept more operations.
    Closed,
    /// A receive is already active on this session.
    ConcurrentReceive,
    /// A bounded owned allocation failed.
    ResourceExhausted,
    /// The native POSIX `errno`, preserved without translation.
    NativeErrno(i32),
    /// The backend failed without an OS error code.
    BackendFailure,
}

/// A statically selected backend for one app-owned IPv4 UDP socket.
///
/// Implementations bind only the endpoint in `DatagramBindConfig`; they do not imply peer
/// discovery, message retries, multicast membership, or application protocol framing. They must
/// reject invalid destinations and oversized payloads before native send work. Receive operations
/// return one owned datagram with its source endpoint and must distinguish nonblocking absence,
/// finite timeout, cancellation, closure, allocation failure, and native errors. Only one receive
/// may be active at a time; sends may proceed while a receive waits. `cancel_receive` must wake the
/// pending receive without closing the socket. `close` is idempotent, wakes a pending receive, and
/// rejects later operations.
pub trait DatagramBackend: Sized {
    /// Binds one socket after applying local options and returns its backend state.
    fn bind(config: DatagramBindConfig) -> Result<Self, DatagramError>;

    /// Returns the actual local endpoint, including an OS-selected port after a port-zero bind.
    fn local_endpoint(&self) -> Result<Ipv4Endpoint, DatagramError>;

    /// Sends one datagram to an explicit destination. Empty datagrams are valid.
    fn send_to(&self, destination: Ipv4Endpoint, payload: &[u8]) -> Result<(), DatagramError>;

    /// Receives one owned datagram no larger than `max_payload_bytes`.
    fn receive(&self, max_payload_bytes: u32, wait: ReceiveWait)
    -> Result<Datagram, DatagramError>;

    /// Cancels only the currently active receive, if any.
    fn cancel_receive(&self) -> Result<(), DatagramError>;

    /// Closes the socket and wakes a pending receive. Repeated calls succeed.
    fn close(&self) -> Result<(), DatagramError>;
}

/// A thin owner of a statically selected datagram backend.
pub struct DatagramSession<B: DatagramBackend> {
    backend: B,
}

impl<B: DatagramBackend> DatagramSession<B> {
    /// Binds the selected backend without runtime backend lookup or global initialization.
    pub fn bind(config: DatagramBindConfig) -> Result<Self, DatagramError> {
        let local = config.local_endpoint();
        if local.is_limited_broadcast() || local.is_multicast() {
            return Err(DatagramError::InvalidEndpoint);
        }
        Ok(Self {
            backend: B::bind(config)?,
        })
    }

    /// Wraps an explicitly constructed backend.
    pub const fn from_backend(backend: B) -> Self {
        Self { backend }
    }

    /// Returns the actual local endpoint.
    pub fn local_endpoint(&self) -> Result<Ipv4Endpoint, DatagramError> {
        self.backend.local_endpoint()
    }

    /// Sends one datagram, rejecting oversized payloads before backend work.
    pub fn send_to(&self, destination: Ipv4Endpoint, payload: &[u8]) -> Result<(), DatagramError> {
        if payload.len() > MAX_DATAGRAM_PAYLOAD_BYTES as usize {
            return Err(DatagramError::PayloadTooLarge);
        }
        self.backend.send_to(destination, payload)
    }

    /// Receives one owned datagram under a nonzero maximum payload bound.
    pub fn receive(
        &self,
        max_payload_bytes: u32,
        wait: ReceiveWait,
    ) -> Result<Datagram, DatagramError> {
        if max_payload_bytes == 0 || max_payload_bytes > MAX_DATAGRAM_PAYLOAD_BYTES {
            return Err(DatagramError::InvalidReceiveBound);
        }
        self.backend.receive(max_payload_bytes, wait)
    }

    /// Cancels the active receive without closing the session.
    pub fn cancel_receive(&self) -> Result<(), DatagramError> {
        self.backend.cancel_receive()
    }

    /// Closes the session and wakes a pending receive.
    pub fn close(&self) -> Result<(), DatagramError> {
        self.backend.close()
    }
}

impl<B: DatagramBackend> Drop for DatagramSession<B> {
    fn drop(&mut self) {
        let _ = self.backend.close();
    }
}
