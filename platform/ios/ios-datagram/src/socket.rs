use core::mem::{size_of, zeroed};
use framework_datagram::{
    Datagram, DatagramBackend, DatagramBindConfig, DatagramError, Ipv4Endpoint,
    MAX_DATAGRAM_PAYLOAD_BYTES, ReceiveWait,
};
use std::ffi::c_void;
use std::io::{Error as IoError, ErrorKind};
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

struct SocketState {
    socket: Option<Arc<UdpSocket>>,
    closed: bool,
}

#[derive(Default)]
struct ReceiveState {
    active: bool,
    cancelled: bool,
}

struct WakePipe {
    read: OwnedFd,
    write: OwnedFd,
}

impl WakePipe {
    fn new() -> Result<Self, DatagramError> {
        let mut fds = [-1; 2];
        // SAFETY: `pipe` writes two fresh descriptors to the valid two-element array.
        if unsafe { libc::pipe(fds.as_mut_ptr()) } < 0 {
            return Err(last_errno());
        }
        // SAFETY: a successful `pipe` initialized each descriptor exactly once.
        let read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        // SAFETY: a successful `pipe` initialized each descriptor exactly once.
        let write = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        set_fd_flags(read.as_raw_fd())?;
        set_fd_flags(write.as_raw_fd())?;
        Ok(Self { read, write })
    }

    fn notify(&self) -> Result<(), DatagramError> {
        let byte = [1_u8];
        loop {
            // SAFETY: `write` points to one initialized byte and the descriptor is owned here.
            let result = unsafe {
                libc::write(
                    self.write.as_raw_fd(),
                    byte.as_ptr().cast::<c_void>(),
                    byte.len(),
                )
            };
            if result == 1 || (result < 0 && last_errno_code() == libc::EAGAIN) {
                return Ok(());
            }
            if result < 0 && last_errno_code() == libc::EINTR {
                continue;
            }
            return Err(last_errno());
        }
    }

    fn drain(&self) -> Result<(), DatagramError> {
        let mut bytes = [0_u8; 64];
        loop {
            // SAFETY: `bytes` is writable for its full length and the descriptor is owned here.
            let result = unsafe {
                libc::read(
                    self.read.as_raw_fd(),
                    bytes.as_mut_ptr().cast::<c_void>(),
                    bytes.len(),
                )
            };
            if result > 0 {
                continue;
            }
            if result == 0 || (result < 0 && last_errno_code() == libc::EAGAIN) {
                return Ok(());
            }
            if result < 0 && last_errno_code() == libc::EINTR {
                continue;
            }
            return Err(last_errno());
        }
    }
}

struct Shared {
    socket: Mutex<SocketState>,
    receive: Mutex<ReceiveState>,
    wake: WakePipe,
    broadcast: bool,
}

/// The public POSIX-socket backend used by [`crate::IosDatagramSession`].
///
/// The socket uses BSD `socket`, `setsockopt`, and `bind` calls for configuration-before-bind,
/// followed by Rust's standard UDP send/receive operations. `SO_REUSEADDR` and `SO_BROADCAST` are
/// applied before bind. Multicast destination and membership APIs are intentionally unsupported.
pub struct IosDatagramBackend {
    shared: Shared,
}

impl IosDatagramBackend {
    fn local_endpoint(&self) -> Result<Ipv4Endpoint, DatagramError> {
        let state = lock(&self.shared.socket)?;
        let socket = state.socket.as_ref().ok_or(DatagramError::Closed)?;
        match socket.local_addr().map_err(map_io_error)? {
            SocketAddr::V4(address) => Ok(endpoint_from_socket(address)),
            SocketAddr::V6(_) => Err(DatagramError::BackendFailure),
        }
    }

    fn send_to(&self, destination: Ipv4Endpoint, payload: &[u8]) -> Result<(), DatagramError> {
        validate_destination(destination, self.shared.broadcast)?;
        if payload.len() > MAX_DATAGRAM_PAYLOAD_BYTES as usize {
            return Err(DatagramError::PayloadTooLarge);
        }
        let state = lock(&self.shared.socket)?;
        let socket = state.socket.as_ref().ok_or(DatagramError::Closed)?;
        let address = SocketAddrV4::new(destination.address().into(), destination.port());
        let sent = socket.send_to(payload, address).map_err(map_io_error)?;
        if sent != payload.len() {
            return Err(DatagramError::BackendFailure);
        }
        Ok(())
    }

    fn receive(
        &self,
        max_payload_bytes: u32,
        wait: ReceiveWait,
    ) -> Result<Datagram, DatagramError> {
        if max_payload_bytes == 0 || max_payload_bytes > MAX_DATAGRAM_PAYLOAD_BYTES {
            return Err(DatagramError::InvalidReceiveBound);
        }
        let _guard = ReceiveGuard::begin(self)?;
        let socket = {
            let state = lock(&self.shared.socket)?;
            if state.closed {
                return Err(DatagramError::Closed);
            }
            Arc::clone(state.socket.as_ref().ok_or(DatagramError::Closed)?)
        };
        let maximum = max_payload_bytes as usize;
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(maximum + 1)
            .map_err(|_| DatagramError::ResourceExhausted)?;
        buffer.resize(maximum + 1, 0);
        let started = Instant::now();

        loop {
            if self.is_closed()? {
                return Err(DatagramError::Closed);
            }
            if self.is_receive_cancelled()? {
                return Err(DatagramError::Cancelled);
            }
            match socket.recv_from(&mut buffer) {
                Ok((received, source)) => {
                    if self.is_closed()? {
                        return Err(DatagramError::Closed);
                    }
                    if self.is_receive_cancelled()? {
                        return Err(DatagramError::Cancelled);
                    }
                    if received > maximum {
                        return Err(DatagramError::PayloadTooLarge);
                    }
                    buffer.truncate(received);
                    let source = match source {
                        SocketAddr::V4(address) => endpoint_from_socket(address),
                        SocketAddr::V6(_) => return Err(DatagramError::BackendFailure),
                    };
                    return Datagram::new(source, buffer);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => match wait {
                    ReceiveWait::NonBlocking => return Err(DatagramError::WouldBlock),
                    ReceiveWait::TimeoutMillis(timeout) => {
                        let elapsed = started.elapsed();
                        let total = Duration::from_millis(u64::from(timeout.get()));
                        let remaining = total.saturating_sub(elapsed);
                        if remaining.is_zero() {
                            return Err(DatagramError::TimedOut);
                        }
                        self.poll_readable(&socket, Some(remaining))?;
                    }
                    ReceiveWait::UntilCancelled => self.poll_readable(&socket, None)?,
                },
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(map_io_error(error)),
            }
        }
    }

    fn cancel_receive(&self) -> Result<(), DatagramError> {
        let mut receive = lock(&self.shared.receive)?;
        if receive.active {
            receive.cancelled = true;
            self.shared.wake.notify()?;
        }
        Ok(())
    }

    fn close(&self) -> Result<(), DatagramError> {
        let mut state = lock(&self.shared.socket)?;
        if state.closed {
            return Ok(());
        }
        state.closed = true;
        let wake_result = self.shared.wake.notify();
        state.socket.take();
        wake_result
    }

    fn is_closed(&self) -> Result<bool, DatagramError> {
        Ok(lock(&self.shared.socket)?.closed)
    }

    fn is_receive_cancelled(&self) -> Result<bool, DatagramError> {
        Ok(lock(&self.shared.receive)?.cancelled)
    }

    fn poll_readable(
        &self,
        socket: &UdpSocket,
        remaining: Option<Duration>,
    ) -> Result<(), DatagramError> {
        let timeout = match remaining {
            None => -1,
            Some(value) => poll_timeout_millis(value),
        };
        let mut descriptors = [
            libc::pollfd {
                fd: socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: self.shared.wake.read.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // SAFETY: `descriptors` contains two initialized poll descriptors for this call.
        let result =
            unsafe { libc::poll(descriptors.as_mut_ptr(), descriptors.len() as _, timeout) };
        if result < 0 {
            if last_errno_code() == libc::EINTR {
                return Ok(());
            }
            return Err(last_errno());
        }
        if result == 0 {
            if let Some(remaining) = remaining {
                if remaining <= Duration::from_millis(i64::from(timeout.max(0)) as u64) {
                    return Err(DatagramError::TimedOut);
                }
            }
            return Ok(());
        }
        let wake_events = descriptors[1].revents;
        if wake_events & libc::POLLIN != 0 {
            self.shared.wake.drain()?;
        }
        if descriptors
            .iter()
            .any(|descriptor| descriptor.revents & libc::POLLNVAL != 0)
        {
            return Err(DatagramError::BackendFailure);
        }
        Ok(())
    }
}

impl DatagramBackend for IosDatagramBackend {
    fn bind(config: DatagramBindConfig) -> Result<Self, DatagramError> {
        let local = config.local_endpoint();
        if local.is_limited_broadcast() || local.is_multicast() {
            return Err(DatagramError::InvalidEndpoint);
        }
        let raw =
            // SAFETY: `socket` receives valid domain/type/protocol constants and owns its result.
            unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if raw < 0 {
            return Err(last_errno());
        }
        // SAFETY: a nonnegative socket descriptor is uniquely transferred into `UdpSocket`.
        let socket = unsafe { UdpSocket::from_raw_fd(raw) };
        set_fd_flags(socket.as_raw_fd())?;
        set_socket_option(&socket, libc::SO_REUSEADDR, config.reuse_address())?;
        set_socket_option(&socket, libc::SO_BROADCAST, config.broadcast())?;
        let address = sockaddr_from_endpoint(config.local_endpoint());
        // SAFETY: `address` is a fully initialized IPv4 sockaddr for the live socket descriptor.
        let result = unsafe {
            libc::bind(
                socket.as_raw_fd(),
                (&address as *const libc::sockaddr_in).cast::<libc::sockaddr>(),
                size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        };
        if result < 0 {
            return Err(last_errno());
        }
        let wake = WakePipe::new()?;
        Ok(Self {
            shared: Shared {
                socket: Mutex::new(SocketState {
                    socket: Some(Arc::new(socket)),
                    closed: false,
                }),
                receive: Mutex::new(ReceiveState::default()),
                wake,
                broadcast: config.broadcast(),
            },
        })
    }

    fn local_endpoint(&self) -> Result<Ipv4Endpoint, DatagramError> {
        IosDatagramBackend::local_endpoint(self)
    }

    fn send_to(&self, destination: Ipv4Endpoint, payload: &[u8]) -> Result<(), DatagramError> {
        IosDatagramBackend::send_to(self, destination, payload)
    }

    fn receive(
        &self,
        max_payload_bytes: u32,
        wait: ReceiveWait,
    ) -> Result<Datagram, DatagramError> {
        IosDatagramBackend::receive(self, max_payload_bytes, wait)
    }

    fn cancel_receive(&self) -> Result<(), DatagramError> {
        IosDatagramBackend::cancel_receive(self)
    }

    fn close(&self) -> Result<(), DatagramError> {
        IosDatagramBackend::close(self)
    }
}

struct ReceiveGuard<'a> {
    backend: &'a IosDatagramBackend,
}

impl<'a> ReceiveGuard<'a> {
    fn begin(backend: &'a IosDatagramBackend) -> Result<Self, DatagramError> {
        let mut state = lock(&backend.shared.receive)?;
        if state.active {
            return Err(DatagramError::ConcurrentReceive);
        }
        state.active = true;
        state.cancelled = false;
        Ok(Self { backend })
    }
}

impl Drop for ReceiveGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.backend.shared.receive.lock() {
            state.active = false;
            state.cancelled = false;
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, DatagramError> {
    mutex.lock().map_err(|_| DatagramError::BackendFailure)
}

fn validate_destination(
    destination: Ipv4Endpoint,
    broadcast_enabled: bool,
) -> Result<(), DatagramError> {
    if destination.port() == 0 || destination.is_unspecified() {
        return Err(DatagramError::InvalidEndpoint);
    }
    if destination.is_multicast() {
        return Err(DatagramError::MulticastUnsupported);
    }
    if destination.is_limited_broadcast() && !broadcast_enabled {
        return Err(DatagramError::BroadcastNotEnabled);
    }
    Ok(())
}

fn endpoint_from_socket(address: SocketAddrV4) -> Ipv4Endpoint {
    Ipv4Endpoint::new(address.ip().octets(), address.port())
}

fn sockaddr_from_endpoint(endpoint: Ipv4Endpoint) -> libc::sockaddr_in {
    // SAFETY: zero is a valid initial bit pattern for the C sockaddr fields before assignment.
    let mut address: libc::sockaddr_in = unsafe { zeroed() };
    #[cfg(target_vendor = "apple")]
    {
        address.sin_len = size_of::<libc::sockaddr_in>() as u8;
    }
    address.sin_family = libc::AF_INET as libc::sa_family_t;
    address.sin_port = endpoint.port().to_be();
    address.sin_addr.s_addr = u32::from_ne_bytes(endpoint.address());
    address
}

fn set_socket_option(
    socket: &UdpSocket,
    option: libc::c_int,
    enabled: bool,
) -> Result<(), DatagramError> {
    let value: libc::c_int = if enabled { 1 } else { 0 };
    // SAFETY: `value` points to one initialized integer and the option is a SOL_SOCKET integer.
    let result = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            option,
            (&value as *const libc::c_int).cast::<c_void>(),
            size_of::<libc::c_int>() as libc::socklen_t,
        )
    };
    if result < 0 {
        Err(last_errno())
    } else {
        Ok(())
    }
}

fn set_fd_flags(fd: libc::c_int) -> Result<(), DatagramError> {
    // SAFETY: `fcntl` is called with the valid descriptor and F_GETFL has no variadic argument.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(last_errno());
    }
    // SAFETY: `fcntl` uses F_SETFL with the flags returned for this live descriptor.
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(last_errno());
    }
    // SAFETY: `fcntl` is called with the valid descriptor and F_GETFD has no variadic argument.
    let descriptor_flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if descriptor_flags < 0 {
        return Err(last_errno());
    }
    // SAFETY: `fcntl` uses F_SETFD with flags returned for this live descriptor.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, descriptor_flags | libc::FD_CLOEXEC) } < 0 {
        return Err(last_errno());
    }
    Ok(())
}

fn poll_timeout_millis(remaining: Duration) -> libc::c_int {
    let millis = remaining.as_nanos().saturating_add(999_999) / 1_000_000;
    millis.clamp(1, libc::c_int::MAX as u128) as libc::c_int
}

fn last_errno() -> DatagramError {
    DatagramError::NativeErrno(last_errno_code())
}

fn last_errno_code() -> i32 {
    IoError::last_os_error().raw_os_error().unwrap_or(libc::EIO)
}

fn map_io_error(error: IoError) -> DatagramError {
    match error.kind() {
        ErrorKind::WouldBlock => DatagramError::WouldBlock,
        ErrorKind::TimedOut => DatagramError::TimedOut,
        ErrorKind::Interrupted => DatagramError::NativeErrno(libc::EINTR),
        _ => error
            .raw_os_error()
            .map(DatagramError::NativeErrno)
            .unwrap_or(DatagramError::BackendFailure),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IosDatagramSession;
    use core::num::NonZeroU32;
    use std::sync::mpsc;
    use std::thread;

    fn bind_loopback() -> IosDatagramBackend {
        IosDatagramBackend::bind(DatagramBindConfig::new(Ipv4Endpoint::new(
            [127, 0, 0, 1],
            0,
        )))
        .expect("loopback UDP bind")
    }

    #[test]
    fn portable_values_preserve_endpoint_options_and_owned_datagram_contract() {
        let endpoint = Ipv4Endpoint::new([192, 0, 2, 7], 22_000);
        assert_eq!(endpoint.address(), [192, 0, 2, 7]);
        assert_eq!(endpoint.port(), 22_000);
        assert!(!endpoint.is_unspecified());
        assert!(!endpoint.is_multicast());
        assert!(Ipv4Endpoint::new([255, 255, 255, 255], 1).is_limited_broadcast());
        assert!(Ipv4Endpoint::new([239, 1, 2, 3], 1).is_multicast());
        assert!(!Ipv4Endpoint::new([223, 255, 255, 255], 1).is_multicast());

        let config = DatagramBindConfig::new(Ipv4Endpoint::new([0, 0, 0, 0], 0));
        assert!(!config.reuse_address());
        assert!(!config.broadcast());
        let configured = config.with_reuse_address(true).with_broadcast(true);
        assert!(configured.reuse_address());
        assert!(configured.broadcast());
        assert_eq!(
            Datagram::new(Ipv4Endpoint::new([127, 0, 0, 1], 0), Vec::new()),
            Err(DatagramError::InvalidEndpoint)
        );
        assert_eq!(
            Datagram::new(
                Ipv4Endpoint::new([127, 0, 0, 1], 1),
                vec![0; MAX_DATAGRAM_PAYLOAD_BYTES as usize + 1]
            ),
            Err(DatagramError::PayloadTooLarge)
        );
    }

    fn timeout(milliseconds: u32) -> ReceiveWait {
        ReceiveWait::TimeoutMillis(NonZeroU32::new(milliseconds).expect("nonzero timeout"))
    }

    fn wait_until_receive_is_active(session: &IosDatagramBackend) {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if lock(&session.shared.receive)
                .expect("receive state lock")
                .active
            {
                return;
            }
            assert!(Instant::now() < deadline, "receive did not become active");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn loopback_send_receive_preserves_source_and_owned_payload() {
        let receiver = bind_loopback();
        let sender = bind_loopback();
        let source = sender.local_endpoint().expect("sender endpoint");
        let destination = receiver.local_endpoint().expect("receiver endpoint");

        sender
            .send_to(destination, b"owned datagram")
            .expect("send datagram");
        let datagram = receiver
            .receive(64, timeout(500))
            .expect("receive datagram");
        assert_eq!(datagram.source(), source);
        assert_eq!(datagram.payload(), b"owned datagram");
        assert_eq!(datagram.into_payload(), b"owned datagram");
    }

    #[test]
    fn empty_datagrams_are_preserved_as_messages() {
        let receiver = bind_loopback();
        let sender = bind_loopback();
        sender
            .send_to(receiver.local_endpoint().unwrap(), &[])
            .expect("send empty datagram");
        let received = receiver.receive(1, timeout(500)).expect("empty datagram");
        assert!(received.payload().is_empty());
        assert_ne!(received.source().port(), 0);
    }

    #[test]
    fn invalid_destinations_and_payload_bounds_fail_before_send() {
        let sender = bind_loopback();
        for destination in [
            Ipv4Endpoint::new([127, 0, 0, 1], 0),
            Ipv4Endpoint::new([0, 0, 0, 0], 53),
        ] {
            assert_eq!(
                sender.send_to(destination, b"x"),
                Err(DatagramError::InvalidEndpoint)
            );
        }
        assert_eq!(
            sender.send_to(Ipv4Endpoint::new([224, 0, 0, 1], 53), b"x"),
            Err(DatagramError::MulticastUnsupported)
        );
        assert_eq!(
            sender.send_to(Ipv4Endpoint::new([255, 255, 255, 255], 53), b"x"),
            Err(DatagramError::BroadcastNotEnabled)
        );
        let oversized = std::vec![0; MAX_DATAGRAM_PAYLOAD_BYTES as usize + 1];
        assert_eq!(
            sender.send_to(Ipv4Endpoint::new([127, 0, 0, 1], 9), &oversized),
            Err(DatagramError::PayloadTooLarge)
        );
        assert_eq!(
            sender.receive(0, ReceiveWait::NonBlocking),
            Err(DatagramError::InvalidReceiveBound)
        );
        assert_eq!(
            sender.receive(MAX_DATAGRAM_PAYLOAD_BYTES + 1, ReceiveWait::NonBlocking),
            Err(DatagramError::InvalidReceiveBound)
        );
    }

    #[test]
    fn invalid_local_broadcast_and_multicast_endpoints_are_rejected() {
        for address in [[255, 255, 255, 255], [239, 1, 2, 3]] {
            assert_eq!(
                IosDatagramSession::bind(DatagramBindConfig::new(Ipv4Endpoint::new(address, 0)))
                    .err(),
                Some(DatagramError::InvalidEndpoint)
            );
        }
    }

    #[test]
    fn receive_bound_detects_truncation_instead_of_returning_partial_data() {
        let receiver = bind_loopback();
        let sender = bind_loopback();
        let destination = receiver.local_endpoint().unwrap();
        sender.send_to(destination, &[7; 33]).unwrap();
        assert_eq!(
            receiver.receive(32, timeout(500)),
            Err(DatagramError::PayloadTooLarge)
        );
        sender.send_to(destination, b"next").unwrap();
        assert_eq!(
            receiver.receive(32, timeout(500)).unwrap().payload(),
            b"next"
        );
    }

    #[test]
    fn nonblocking_timeout_and_native_errno_are_distinct() {
        let receiver = bind_loopback();
        assert_eq!(
            receiver.receive(32, ReceiveWait::NonBlocking),
            Err(DatagramError::WouldBlock)
        );
        assert_eq!(
            receiver.receive(32, timeout(5)),
            Err(DatagramError::TimedOut)
        );

        let first = bind_loopback();
        let endpoint = first.local_endpoint().unwrap();
        let error = IosDatagramBackend::bind(DatagramBindConfig::new(endpoint))
            .err()
            .expect("second exclusive bind must fail");
        assert_eq!(error, DatagramError::NativeErrno(libc::EADDRINUSE));
        assert_eq!(
            map_io_error(IoError::from_raw_os_error(libc::EACCES)),
            DatagramError::NativeErrno(libc::EACCES)
        );
    }

    #[test]
    fn only_one_receive_may_wait_on_a_session() {
        let receiver = Arc::new(bind_loopback());
        let thread_receiver = Arc::clone(&receiver);
        let (started, ready) = mpsc::channel();
        let task = thread::spawn(move || {
            started.send(()).unwrap();
            thread_receiver.receive(32, ReceiveWait::UntilCancelled)
        });
        ready.recv().unwrap();
        wait_until_receive_is_active(&receiver);
        assert_eq!(
            receiver.receive(32, ReceiveWait::NonBlocking),
            Err(DatagramError::ConcurrentReceive)
        );
        receiver.cancel_receive().unwrap();
        assert_eq!(task.join().unwrap(), Err(DatagramError::Cancelled));
    }

    #[test]
    fn cancel_wakes_only_the_current_receive_and_socket_remains_usable() {
        let receiver = Arc::new(bind_loopback());
        let sender = bind_loopback();
        let destination = receiver.local_endpoint().unwrap();
        let thread_receiver = Arc::clone(&receiver);
        let (started, ready) = mpsc::channel();
        let task = thread::spawn(move || {
            started.send(()).unwrap();
            thread_receiver.receive(32, ReceiveWait::UntilCancelled)
        });
        ready.recv().unwrap();
        wait_until_receive_is_active(&receiver);
        receiver.cancel_receive().unwrap();
        assert_eq!(task.join().unwrap(), Err(DatagramError::Cancelled));
        sender.send_to(destination, b"still open").unwrap();
        assert_eq!(
            receiver.receive(32, timeout(500)).unwrap().payload(),
            b"still open"
        );
    }

    #[test]
    fn close_wakes_waiter_is_idempotent_and_rejects_later_calls() {
        let receiver = Arc::new(bind_loopback());
        let thread_receiver = Arc::clone(&receiver);
        let (started, ready) = mpsc::channel();
        let task = thread::spawn(move || {
            started.send(()).unwrap();
            thread_receiver.receive(32, ReceiveWait::UntilCancelled)
        });
        ready.recv().unwrap();
        wait_until_receive_is_active(&receiver);
        receiver.close().unwrap();
        assert_eq!(task.join().unwrap(), Err(DatagramError::Closed));
        assert_eq!(receiver.close(), Ok(()));
        assert_eq!(receiver.local_endpoint(), Err(DatagramError::Closed));
        assert_eq!(
            receiver.send_to(Ipv4Endpoint::new([127, 0, 0, 1], 53), b"x"),
            Err(DatagramError::Closed)
        );
    }
}
