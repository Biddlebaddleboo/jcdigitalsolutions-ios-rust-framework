# PLAN_B532_TCP_LISTENER.md — Workstream B532: TCP Listener and Accept

## Status

Implementation in place; integration and registered-validator qualification remain with the root integrator

## Starting state

- Latest `origin/main` is `3447e3f feat(files): add native append operation`
- `framework-connection` exposes an executor-neutral `ConnectionBackend` for outbound secure TCP streams
- `ios-connection` implements the stream with public Network.framework C APIs and a crate-local C shim
- Capability row `027-networking-web-low-level-network-connection-listener-where-appropriate` marks TCP listener support partial
- The installed `ios-rust-validate` lists only `ios-datagram` among connection/network pilots; no `ios-connection` profile exists
- Xcode iOS 26.5 headers declare `nw_listener_create_with_port`, listener handlers, TCP options, and listener cancellation from iOS 12.0

## Objective

Add a reusable `no_std` portable TCP listener facade and an iOS `NWListener` backend. Keep accepted streams caller-owned, cancellation explicit, and callbacks executor-neutral. Do not add app protocols, service discovery, multicast, TLS identity configuration, or a global runtime

## Proposed portable contract

- `TcpListenerBackend` has lazy bind, accept, close, and nonblocking cancel operations
- Bind accepts port `0` for OS-selected ephemeral port and resolves with the actual nonzero bound port
- `TcpListener::accept` returns an `AcceptedConnection` with send, bounded receive, close, cancel, and drop semantics matching existing `Connection`
- An accepted backend is ready on return; no caller-supplied endpoint or second connect step is required
- Only one in-flight accept future may borrow a listener; dropping it detaches its waker and leaves the listener active
- Listener close is idempotent, resolves at native cancellation completion, drops unreturned accepted streams, and does not close streams already returned to callers
- iOS keeps a bounded pending-accept queue; overflow connections are immediately cancelled

## iOS transport and lifecycle

- Use public `nw_listener_create_with_port` with `nw_parameters_create`, `nw_tcp_create_options`, and `nw_protocol_stack_set_transport_protocol`
- Bind to the wildcard local endpoint; expose the actual port only after listener-ready state
- Accepted streams use public `nw_connection_t` send/receive APIs without TLS; document plaintext transport explicitly
- Reuse the existing per-stream connection state, operation callbacks, ownership, cancellation, and panic containment by adopting each unstarted listener connection into its own `IosConnectionBackend`
- Keep one private serial Dispatch queue per listener and per accepted connection; no user callback runs on a native queue
- Cancel incoming connections that arrive after close, when the bounded queue is full, or if accepted-backend allocation fails
- iOS minimum remains 12.0; Simulator minimum remains 14.0

## Implemented and verified acceptance criteria

- Host fake-backend tests cover lazy bind, port `0`, readiness and native errors, accept ownership, dropped accept future, pending queue order, close/cancel idempotence, listener drop, and accepted-stream operations
- Host tests cover queue overflow and rejection without consuming or corrupting the next accepted stream
- iOS device and Simulator compile and strict Clippy pass
- Release link/import probes verify the outbound and listener probes, selected listener and TCP symbols, exact deployment floors, public `Network`/system imports, and absence of Swift/Python/Objective-C class runtime, Bonjour, UDP, multicast, and unrelated Network APIs
- Existing `framework-connection` outbound tests remain unchanged and pass
- No live network, local listener runtime, peer byte delivery, device, or Simulator behavior claim

## Validation notes

- Host tests: `framework-connection` 18 tests; `ios-connection` 2 listener-queue tests
- Portable no-default-features check, device and Simulator cross-compiles, strict Clippy, Rustdoc, and the combined release link/import script pass
- Installed validator has no `ios-connection` capability profile yet. Root integrator owns registration and must run it after adding the B532 profile

## Owned paths

- `PLAN_B532_TCP_LISTENER.md`
- `crates/framework-connection/**`
- `platform/ios/ios-connection/**`

Root-owned `tools/validation/specs/validation-v1.json`, capability status, shared plans, workspace registration, generated reports, and repository push remain with the integrator

## Stop condition

Stop if the accepted-stream backend cannot reuse the existing ownership and operation contract without breaking `ConnectionBackend` source compatibility or introducing a mandatory executor, thread, or process-wide runtime
