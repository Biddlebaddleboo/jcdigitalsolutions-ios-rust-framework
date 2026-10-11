# iOS IPv4 UDP datagrams

`framework-datagram` supplies no-std IPv4 endpoint, bind configuration, receive policy, owned
datagram, and error types. `ios-datagram` supplies `IosDatagramSession`, a statically selected
Rust-native backend over public BSD sockets. It is separate from `ios-connection`; UDP users do
not link the TLS-over-TCP Network.framework shim.

Bind an explicit IPv4 address and port. Port zero asks the OS to select a local port, which the
session returns through `local_endpoint()`. `SO_REUSEADDR` and `SO_BROADCAST` are applied before
bind and default off. Send calls target one explicit IPv4 endpoint; empty datagrams are valid and
payloads above `MAX_DATAGRAM_PAYLOAD_BYTES` are rejected. Receive returns an owned payload and its
source endpoint, rejects a zero or over-limit receive bound, and detects datagrams larger than the
requested bound instead of returning a truncated payload as complete.

Receive policy is explicit: `NonBlocking` returns `WouldBlock`, `TimeoutMillis` returns
`TimedOut` at the finite deadline, and `UntilCancelled` waits until data, cancellation, or close.
Only one receive may wait on a session; sends can proceed while it waits. `cancel_receive()` wakes
only the active receive and leaves the socket usable. `close()` is idempotent, wakes a pending
receive, and rejects later operations. Dropping `DatagramSession` closes its backend. Native POSIX
errors preserve the raw `errno` in `DatagramError::NativeErrno`.

This slice supports IPv4 unicast and optional limited or directed broadcast when the socket
broadcast option is enabled, subject to OS and network policy. It rejects IPv4 multicast
destinations and does not join multicast groups.

On iOS 14 and later, communication with local-network destinations is subject to Local Network
privacy. A host app that directly communicates on the local network must include
`NSLocalNetworkUsageDescription`. Apple's Local Network privacy rules apply to BSD sockets:
outgoing UDP unicast to local-network destinations requires Local Network access, while incoming
UDP unicast alone does not trigger the same check. Apple's multicast entitlement documentation
requires `com.apple.developer.networking.multicast` for broadcast and multicast traffic; this API
defaults broadcast off, rejects multicast destinations, and does not join groups. The framework
does not inspect entitlements, prompt, preflight authorization, or infer permission from a
successful compile. Public references:
[TN3179: Understanding Local Network Privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy),
[`NSLocalNetworkUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocalnetworkusagedescription),
and the [multicast entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.multicast).

The registered link probe imports `libSystem.B.dylib` and the public iOS
`libiconv.2.dylib` install name emitted by the workspace `libc` crate's target link metadata; it
has no undefined `iconv` symbols. It checks the public BSD socket imports and deployment floors.
Device/Simulator compilation and link/import checks establish only target compatibility; they do
not prove live Wi-Fi delivery, Local Network prompt behavior, broadcast entitlement, background
execution, or physical-device cancellation timing.
