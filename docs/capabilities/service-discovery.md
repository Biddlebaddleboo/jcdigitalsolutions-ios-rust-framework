# Portable service discovery contract

`framework-service-discovery` defines a `no_std` caller-selected Bonjour query, exact owned
service identity, state snapshots, membership events, and a statically selected asynchronous
backend contract. It contains no platform types, service payloads, TXT record values, transport
handles, app protocol, or Rideshare-specific model

The query accepts one ASCII `_service._tcp` or `_service._udp` type and an optional ASCII DNS
domain. `None` requests the backend's default browse domains. Input and returned identity strings
are retained exactly, without Unicode normalization or lossy conversion. The contract caps each
identity component at 255 UTF-8 bytes and applies DNS's 63-byte label and 255-byte wire-name limits
to domains (at most 253 dotted ASCII bytes without a trailing dot). Backends queue at most 32
membership events; state events may coalesce to the latest snapshot. A full queue is a terminal
`QueueOverflow`, not a silent loss of discovery changes

`BonjourBrowserBackend` is statically selected. Start is lazy, `next_event` is executor-neutral,
`cancel` and drop are nonblocking, and `close` resolves only after the backend's native cancellation
boundary. Dropping a pending event future detaches its waiter without closing the browser. The
portable contract does not perform discovery itself or imply that all platforms support Bonjour

`BonjourAdvertisement` is a bounded, portable descriptor for one `_service._tcp` instance. Its
optional name and domain are copied exactly; the TCP listener supplies the advertised port. The
descriptor contains no TXT data or application protocol. `TcpListener` backends opt in to
`TcpListenerAdvertisementBackend`; registration, descriptor replacement, and removal do not
promise that the service has become visible on the network
