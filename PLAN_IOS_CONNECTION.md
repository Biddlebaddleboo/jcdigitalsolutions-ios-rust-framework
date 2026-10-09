# PLAN_IOS_CONNECTION.md — Workstream B24: iOS Secure TCP Byte Streams

## Status

Implementation and scoped validation are complete. This slice adds one outbound TLS-over-TCP byte
stream through public Network.framework C APIs. It does not extend B3 URLSession HTTP or B18's D15
path snapshot

## Objective

Implement D19 `framework-connection::ConnectionBackend` in a separate `ios-connection` crate,
using only public `<Network/Network.h>` and Dispatch C APIs. Add no Swift or Objective-C source,
private API, third-party block binding, listener, UDP, or endpoint/path preflight

## Dependencies

- D19 `framework-connection` is the portable contract; see `PLAN_CAPABILITIES_CONNECTION.md`
- Foundation A and `framework-core` are integrated
- B3 `ios-network` remains the owner of foreground URLSession HTTP
- D15/B18 `framework-connectivity` / `ios-connectivity` remain the owner of one informational
  `NWPathMonitor` snapshot and are not modified
- The C shim uses the existing crate-local `cc = { version = "1.2", default-features = false }`
  pattern with `-fblocks`; no Rust block or Network.framework binding dependency is needed
- The portable `MAX_CHUNK_BYTES` cap is exactly 1,048,576 bytes per send or receive. It bounds
  memory copied by the C shim/Rust adapter; it is not a TCP or Network.framework transport limit

## Write scope

- `PLAN_IOS_CONNECTION.md`
- `platform/ios/ios-connection/**`
- `docs/ios/connection.md`
- `PLAN_VALIDATION_IOS_CONNECTION.md`

Do not edit B3/B18 source, shared capability JSON/counts, `PLAN_CAPABILITIES.md`,
`PLAN_IOS_NATIVE.md`, `PLAN_VALIDATION.md`, root `Cargo.toml`, CI, or documentation indexes. Root
owns central integration. `Cargo.lock` may receive only the mechanical entries for these two new
workspace packages and their already-used `cc` dependency

## Native API and availability

The local Xcode iOS 26.5 SDK declares these selected APIs available from iOS 12.0:

- `nw_endpoint_create_host`
- `nw_parameters_create_secure_tcp`
- `nw_connection_create`
- `nw_connection_set_state_changed_handler`
- `nw_connection_set_queue`, before start
- `nw_connection_start`, `nw_connection_send`, `nw_connection_receive`, and
  `nw_connection_cancel`
- `nw_content_context_get_is_final`, `nw_error_get_error_domain`, and
  `nw_error_get_error_code`

Create only hostname/IP plus numeric-port endpoints. Use
`nw_parameters_create_secure_tcp(NW_PARAMETERS_DEFAULT_CONFIGURATION,
NW_PARAMETERS_DEFAULT_CONFIGURATION)`; never pass `NW_PARAMETERS_DISABLE_PROTOCOL`. This uses
Apple's default TLS/TCP configuration. Do not expose custom trust, pinning, TLS-version, or
plaintext controls. Document the need for a normally trusted certificate chain and a certificate
subject alternative name matching the hostname/IP; no custom trust exception is supported

## Queue, callbacks, and ownership

- Create a private serial Dispatch queue per connection and set it with `nw_connection_set_queue`
  before `nw_connection_start`. Native connection callbacks and the C-to-Rust event bridge run on
  that queue; no user callback runs there. A waker may be woken there, outside Rust mutex guards.
- Copy every C block with `Block_copy` before passing it to Network.framework. Retain the state
  block through the final `nw_connection_state_cancelled` callback. Retain one-shot send/receive
  blocks until their exact-once callback has returned; schedule block release on the same serial
  queue after that callback
- The C shim owns a native-side Rust context reference after successful start. Rust detaches its
  future/operation state before requesting cancel; the context remains alive until the final
  cancelled-state event, after which it is released exactly once. Rust releases both its temporary
  `Arc` reference and this persistent callback reference inside panic boundaries
- A C lifecycle mutex serializes operation submission, cancellation requests, and final native
  resource detachment. One-shot operation references keep C state alive through callback cleanup,
  even after the facade releases its client handle
- `nw_connection_cancel` is asynchronous. Outstanding send/receive operations receive errors
  before the final cancelled-state event, and no pending send is guaranteed to reach the peer
- A dropped active connect/send/receive future requests cancellation of the whole stream. A
  dropped facade also requests cancellation. Neither drop waits for the native callback nor
  retracts bytes that may already be sent
- No panic may unwind across a C callback. Native callbacks update owned state only and wake the
  stored waker after releasing locks

## Send and receive mapping

- Copy send bytes into a `dispatch_data_t` allocation owned by the C shim before the async send;
  do not retain a Rust borrow in a native block. Empty sends are successful no-ops and oversized
  sends are rejected before native work. Use the default message context with
  `is_complete=true`; this completes that send context but does not close the TCP write direction
- Request one receive at a time with minimum length 1 and a caller maximum no greater than
  `MAX_CHUNK_BYTES`
- Copy discontiguous `dispatch_data_t` regions into owned Rust bytes before returning from the
  native callback
- Preserve content and error when both are present when owned-copy allocation succeeds. Mark EOF
  only when both the receive context is complete and `nw_content_context_get_is_final` is true;
  this maps TCP's final context to a peer FIN
- If `dispatch_data_create_map` cannot allocate a receive copy, return
  `ConnectionError::ReceiveDataUnavailable` with an optional raw native error rather than claiming
  a synthetic POSIX/Network.framework error. If Rust cannot allocate its owned copy after mapping,
  return `ResourceExhausted`. In either allocation-failure case, received bytes cannot be returned
  because no owned buffer exists
- Preserve `nw_error_get_error_domain` and `nw_error_get_error_code` exactly as signed 32-bit
  values. If a callback reports no `nw_error_t`, use `ConnectionError::BackendFailure`
- `nw_connection_state_waiting` and `preparing` remain pending; `ready` completes connect;
  `failed` carries its native error where present; `cancelled` closes the native lifecycle
- No operation timeout is built in. Callers must cancel their operation if their own deadline
  expires

## Local network privacy

The native API floor is iOS 12.0. On iOS 14 and later, outgoing TCP to a local-network address
requires the app's Local Network authorization. The host app, not this crate, must add
`NSLocalNetworkUsageDescription` when it may connect to local-network addresses. The crate does
not request or preflight permission. A denied local operation may remain in Network.framework's
waiting state, so connect can remain pending until a path change or caller cancellation. No
Bonjour, `NSBonjourServices`, multicast entitlement, listener, or inbound-access claim is included

## Link and validation

- Build and strict-Clippy `ios-connection` for `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Build a small Release link probe for both targets at device iOS 12.0 and simulator iOS 14.0;
  `vtool -show-build` must report those exact minimum OS versions
- Audit expected direct imports (`Network`, `libSystem.B.dylib`), Network connection/endpoint/
  parameter/error symbols, and reject Swift/Python runtime, Objective-C class symbols, listener,
  path-monitor, browser, and unrelated Network capability symbols
- Run formatting, rustdoc, docs-check, and zero-Swift-source checks
- Do not execute the probe, connect to any endpoint, or claim TLS verification, byte delivery,
  cancellation timing, local-network prompt behavior, or runtime parity from compile/link evidence

## Handoff

Report the actual SDK declarations and deployment targets, link imports, callback ownership
boundary, exact checks, deviations, and explicit runtime/privacy limits. Link/import evidence is
not runtime or parity evidence
