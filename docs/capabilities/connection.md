# Secure TCP connections

`framework-connection` provides a `no_std` facade for one caller-selected outbound secure TCP
backend. It is intentionally separate from `framework-network` foreground HTTP and
`framework-connectivity` path snapshots. The facade does not resolve names, parse URLs, create a
listener, or decide whether a path can reach an endpoint

## Endpoint and stream model

Create an `Endpoint` from ASCII hostname/IP text and a nonzero port. The portable type rejects an
empty host, non-ASCII input, embedded NUL, whitespace/control bytes, URL delimiters, and port zero;
it does not canonicalize DNS names, convert IDNA, or fully parse IP/DNS syntax. Resolution and
remaining endpoint validation belong to the selected backend

After `connect` completes, call `send`, `receive`, and `close` through the statically selected
backend. Operations borrow the connection mutably and are serialized in this first contract. Each
send and receive is capped at `MAX_CHUNK_BYTES` (1,048,576 bytes) to bound copied memory; that is
not a TCP transport limit. An empty send is a successful no-op. TCP is a byte stream: a receive
returns an owned chunk up to the requested nonzero bound, not a message. `ReadTerminal`
distinguishes more data, peer EOF, and receive error; bytes accompanying an error remain available
in the returned chunk unless an allocation failure makes an owned copy unavailable. EOF closes
the peer's sending direction only

The contract has no executor, `Send`/`Sync` promise, timeout, or retry policy. A connect can remain
pending while the native backend waits or prepares. A caller with a deadline must cancel the
operation when that deadline expires

## Send and cancellation

A successful send reports backend/native send completion, not that the peer received or
acknowledged the data. A failed or cancelled send may have transmitted an unknown prefix. Dropping
an active operation future detaches its waker and requests cancellation of the whole stream;
dropping the facade also requests cancellation. Native cancellation can finish asynchronously,
and neither cancellation nor drop recalls bytes already sent

Native error domains and codes are preserved as raw signed values. Their interpretation is
backend-specific

This guide describes API semantics only. The iOS backend's local-network privacy, TLS defaults,
callback queue, API floor, compile/link evidence, and runtime limits are documented separately in
the [iOS connection guide](../ios/connection.md)
