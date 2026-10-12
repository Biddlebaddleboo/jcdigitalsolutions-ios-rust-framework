# iOS Bonjour service discovery

`ios-service-discovery` implements the portable contract with Apple's public Network.framework
C `NWBrowser` API. It starts only when the first poll of the caller-owned browser's start future
occurs. The caller selects one `_service._tcp` or `_service._udp` type and optional domain; the
adapter copies exact service instance name, type, and domain values from each discovered endpoint.
It accepts base service types only; subtype syntax is rejected. It does not read TXT records,
enumerate interfaces, resolve a service to a socket endpoint, or implement an application protocol

The portable `BonjourAdvertisement` descriptor is applied by `TcpListener<IosTcpListenerBackend>`
to the same Network.framework `NWListener` that accepts connections. Register it before or after
binding; the listener's actual port, including an OS-selected port, is used for the Bonjour SRV
record. `update_advertisement` replaces the descriptor, and `stop_advertising` removes it without
closing the TCP listener. These methods report that the request was accepted or queued, not that
Bonjour publication completed or that a peer can connect

The adapter owns one private serial Dispatch queue and keeps the C callback context alive until the
final `nw_browser_state_cancelled` callback. `close` waits for that cancellation boundary; `cancel`
and drop request asynchronous cancellation. Membership changes use a fixed 32-event queue; a full
queue fails with `QueueOverflow` and stops native browsing instead of silently discarding an event.
Browser state is coalesced. No user callback is invoked by this crate; a stored Rust waker may be
woken on the native queue after locks are released

The Network.framework browser APIs used here are declared by the selected SDK for iOS 13.0 and
later. The registered device build/link target is iOS 13.0 and the Simulator target is iOS 14.0.
The listener advertisement descriptor and setter used by `ios-connection` are declared for iOS
12.0; its registered device target is iOS 12.0 and the Simulator target is iOS 14.0.
Compilation and static link/import inspection do not establish live discovery, cancellation timing,
or peer connectivity

Browsing or advertising on the local network may require Apple's Local Network authorization. The
host application, not this framework, must provide `NSLocalNetworkUsageDescription` and include
every browsed or advertised service type in `NSBonjourServices`. The framework neither requests nor
preflights permission and does not edit the host app's Info.plist. Apple states that Simulator does
not support Local Network privacy; no Simulator or device runtime discovery, registration,
permission, or peer-connectivity evidence is claimed. Apps that browse arbitrary service types or
use other multicast operations must separately review Apple's current multicast entitlement
requirements; these adapters do not request multicast entitlement or use raw multicast sockets

See [Apple's `NWBrowser`](https://developer.apple.com/documentation/network/nwbrowser),
[`nw_advertise_descriptor_create_bonjour_service`](https://developer.apple.com/documentation/network/nw_advertise_descriptor_create_bonjour_service%28_%3A_%3A_%3A%29),
[`nw_listener_set_advertise_descriptor`](https://developer.apple.com/documentation/network/nw_listener_set_advertise_descriptor%28_%3A_%3A%29),
[`nw_browse_descriptor_create_bonjour_service`](https://developer.apple.com/documentation/network/nw_browse_descriptor_create_bonjour_service%28_%3A_%3A%29),
[Network.framework C functions](https://developer.apple.com/documentation/network/network-functions),
[`NSBonjourServices`](https://developer.apple.com/documentation/BundleResources/Information-Property-List/NSBonjourServices),
[`NSLocalNetworkUsageDescription`](https://developer.apple.com/documentation/BundleResources/Information-Property-List/NSLocalNetworkUsageDescription),
and [TN3179: Understanding Local Network Privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)
