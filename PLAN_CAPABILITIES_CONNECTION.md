# PLAN_CAPABILITIES_CONNECTION.md — Workstream D19: Secure TCP Byte Streams

## Status

The portable contract and deterministic fake-backend tests are implemented in
`crates/framework-connection`. The separate B24 iOS backend and scoped compile/link validation are
complete. This contract is a bounded secure byte stream, not HTTP, endpoint reachability, or a
listener API

## Objective

Add a `no_std` static-backend facade for an outbound TLS-over-TCP byte stream. Keep it separate
from D1 `framework-network` foreground HTTP and D15 `framework-connectivity` path snapshots

## Scope

- Validate ASCII host/IP text and a nonzero 16-bit port without URL parsing, IDNA conversion,
  hostname canonicalization, or name resolution in the portable crate
- Establish one secure TCP connection lazily when its connect future is first polled
- Send one borrowed byte slice at a time and receive one bounded owned chunk at a time
- Preserve receive bytes when the native callback also reports an error; report continuation, EOF,
  and error as explicit outcomes
- Preserve a native error domain and code without translating or collapsing either value
- Keep the API executor-neutral, without `Send`, `Sync`, global state, or an async runtime

## Contract semantics

- A connection backend is statically selected and owned by `Connection<B>`
- Connect, send, receive, and close futures borrow the backend mutably; the first contract permits
  only one outstanding operation and does not promise simultaneous full-duplex reads and writes
- `MAX_CHUNK_BYTES` is 1,048,576 bytes for each send or receive. This is a bounded-allocation
  safety contract, not a TCP limit; the facade rejects larger requests before native work, and
  backend implementations must enforce the same rule when used directly
- An empty send is a successful no-op and does not start a native operation
- Network streams have no message framing. Each receive returns at most its nonzero byte bound;
  `ReadTerminal::More` must accompany nonempty data
- `ReadTerminal::EndOfStream` is a peer read-close only; the local send direction may remain open
- `ReadTerminal::Error` retains any bytes delivered with the native failure
- If a backend cannot allocate a copy for received bytes, it returns an allocation error rather
  than a chunk; the iOS backend preserves an accompanying raw native error when it cannot copy
- Send success means backend send completion only, not peer receipt or acknowledgement. Failed or
  cancelled sends may have transmitted an unknown prefix
- Dropping an active operation future must detach its waker and request whole-stream cancellation;
  native cancellation is asynchronous and cannot recall bytes already sent
- `cancel` is idempotent and nonblocking; `close` resolves at the backend's asynchronous cancel
  completion boundary; dropping `Connection` requests cancellation
- Connect can remain pending indefinitely while a backend is waiting or preparing. There is no
  implicit timeout, retry policy, or D15 path preflight

## Non-goals

- No listener, UDP, Bonjour/browser/service discovery, socket escape handle, path monitor, HTTP,
  WebSocket, custom TLS trust callback, certificate pinning, or plaintext transport mode
- No user callback invoked on a native queue
- No claim of concurrent read/write operation support in this first contract

## Write scope

- `PLAN_CAPABILITIES_CONNECTION.md`
- `crates/framework-connection/**`
- `docs/capabilities/connection.md`

Root-owned shared plan indexes, capability status, workspace manifests/lockfile reconciliation,
CI, and documentation indexes are intentionally excluded from this slice

## Validation

- Add deterministic fake-backend unit tests for endpoint validation, lazy connect, owned read
  outcomes including bytes plus error, cancellation/drop, send, close, native domain/code
  preservation, and the `MAX_CHUNK_BYTES` boundary
- Run package tests, no-default-features check, rustdoc, formatting, workspace docs-check, and the
  zero-Swift-source gate
- Do not use live network endpoints as correctness oracles

## Handoff

Report changed paths, exact commands, test results, API deviations, and any portable/native
semantic gap. The native implementation is owned by B24 in `PLAN_IOS_CONNECTION.md`
