# Optional iOS CallKit call-observer C API

The opt-in `framework-c-api` Cargo feature `ios-call-observer` enables
`bindings/c/include/framework_ios_call_observer.h`. It wraps D76's one-shot
`active_call_snapshot()` and returns only the number of values in
`CXCallObserver.calls` plus its aggregate state flags.

## Output contract

`framework_ios_call_observer_active_call_snapshot` writes a `uint64_t` call count and a
`uint32_t` state bitmask. The four public bits mean that any returned call is outgoing,
connected, on hold, or ended. They are independent ORed facts, not correlated to one call.
The wrapper passes D76's state mask through unchanged, including any future bits.

Both outputs are required, distinct, writable, and aligned. Each non-null output is set to
zero before validation or work. A null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`;
on success, read both values only after `FRAMEWORK_STATUS_OK`. The host stub returns
`FRAMEWORK_STATUS_UNSUPPORTED` with zero outputs. A caught Rust panic returns
`FRAMEWORK_STATUS_PANIC` with zero outputs.

## CallKit behavior and limits

This synchronous call runs on the caller's thread and imposes no main-thread rule. CallKit's
`calls` read may block while the system retrieves initial state, so keep this API off
UI-critical work. The snapshot is point-in-time and may be stale as soon as it returns.

D76 creates and releases its `CXCallObserver` and `CXCall` values inside the call. The C API
exposes no CallKit object, UUID, caller data, phone number, history, callback, provider,
controller, audio, PushKit path, or call-control action. It does not request permission or
register a delegate. These limits do not establish host requirements for a full VoIP or
default-calling-app product.

The API floor is iOS 10.0 from D76's header audit. This F12 gate targets device iOS 12.0 and
arm64 Simulator iOS 14.0 based on the current SDK/repository validation floor; these do not
raise the API floor. The gate compiles and links consumers but never executes them and makes
no live-call visibility claim.

See [`docs/ios/call-observer.md`](../ios/call-observer.md) for D76's backend contract and
[`PLAN_BINDINGS_CALL_OBSERVER.md`](../../PLAN_BINDINGS_CALL_OBSERVER.md) for F12 validation
and integration evidence.
