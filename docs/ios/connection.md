# iOS TCP connections and listeners

`ios-connection` implements outbound secure TCP connections and inbound TCP listeners with public
Network.framework C APIs. It does not use URLSession, Swift, Objective-C classes, a global executor,
or D15's `NWPathMonitor`

## TLS and endpoint behavior

The backend uses `nw_parameters_create_secure_tcp` with Apple's default TLS and TCP configuration
and does not offer plaintext mode, custom trust callbacks, or certificate pinning. The peer must
present a normally trusted certificate whose subject alternative name matches the supplied DNS
name or IP address. The backend cannot override failed system trust evaluation

The endpoint is host/IP text plus a numeric nonzero port, not a URL. Network.framework performs
native name resolution. The backend does not preflight or gate the connection using a D15 path
snapshot and has no implicit connect timeout

## Queue and callback ownership

Each native connection uses a private serial Dispatch queue set before start. Network.framework
state, send, and receive callbacks enter that queue. The C shim owns copied blocks and an opaque
Rust callback reference until their callback completion; the persistent state block and native
context are released only after the final `nw_connection_state_cancelled` event. Rust detaches
future state before requesting cancellation. No user callback is invoked on the queue; a stored
waker may be woken there after Rust locks are released

Native send and receive operations are one-at-a-time. Receive data is copied into owned Rust
bytes, including bytes delivered alongside an error. EOF is reported only when Network.framework
marks a complete final TCP context. Each send and receive is capped at 1,048,576 bytes by
`MAX_CHUNK_BYTES`; this bounds adapter allocations and is not a TCP transport limit. Empty sends
are successful no-ops. If the C shim cannot map receive data or Rust cannot allocate the owned
copy, the operation returns an allocation error instead of a chunk; the C mapping error can retain
an accompanying raw Network.framework error without mislabeling the local allocation failure.
A successful send completion does not prove peer receipt or acknowledgement. Dropping an active
operation future requests cancellation of the whole stream; drop is not a synchronous close and
cannot retract sent bytes

`nw_error_get_error_domain` and `nw_error_get_error_code` are preserved exactly. Native
`waiting`/`preparing` states remain pending; a failed connect resolves with its native error when
provided. If no native error value exists, the backend reports `BackendFailure`

## Availability and local-network privacy

The selected endpoint, TLS/TCP, connection, queue, callback, data, and error APIs are declared for
iOS 12.0 in the inspected SDK. On iOS 14 and later, outgoing TCP to local-network addresses is
subject to Local Network privacy. Host applications that may connect to local-network endpoints
must provide `NSLocalNetworkUsageDescription`; this crate does not own or edit the host app's
Info.plist, request permission, or determine authorization ahead of a connection. A denied local
operation may remain in a waiting state until the caller cancels it. No Bonjour key or multicast
entitlement is used by this outbound-only slice

## Inbound TCP listener

B532 adds `TcpListenerBackend` and `AcceptedConnection` to the portable `framework-connection`
contract, with `IosTcpListenerBackend` backed by the public Network.framework `NWListener` and
`NWConnection` C APIs. Binding is lazy; port zero asks the OS to choose a port, returned after the
listener reaches ready state. The iOS accepted streams are plaintext TCP, not TLS. Use a caller-owned
protocol and keep its framing and authentication policy outside this transport crate

The iOS backend queues at most eight pending accepted streams and cancels excess arrivals. An
`accept` future has no implicit timeout; dropping it does not close the listener. `close` waits for
native listener cancellation and does not close streams already returned to the caller; `cancel` and
drop request nonblocking cancellation. Accepted streams reuse the existing bounded send/receive,
explicit close, and cancellation contract. The portable API requires no global executor

The listener binds a wildcard local endpoint, but does not add Bonjour advertising, service
discovery, multicast, UDP, TLS, or an application protocol. Host apps that access the local network
must provide `NSLocalNetworkUsageDescription` and handle the system's Local Network authorization,
per [Apple's local-network privacy guidance](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy);
this crate does not request permission or modify the host app's Info.plist. The selected API floor is
iOS 12.0; the registered device and Simulator link probes use iOS 12.0 and 14.0 deployment floors

## Evidence limits

Device and Simulator check, strict Clippy, and link/import probes validate compilation and linkage
only. They do not execute an outbound connection, TLS handshake, certificate validation, local-network
prompt, listener bind/accept, plaintext byte exchange, EOF, cancellation callback, or runtime parity test
