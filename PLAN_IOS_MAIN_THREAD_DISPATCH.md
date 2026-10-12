# iOS main-thread dispatch

## B561 — cancellable queued handoff

### Scope

Extend the optional `ios-runtime/dispatch` feature with `run_on_main_cancellable`. Preserve the existing `run_on_main` API and callback contract. Calls already on main run inline and return `RanInline`; off-main calls enqueue one callback and return a `MainThreadTaskHandle`.

### Contract

- `MainThreadTaskHandle::cancel` atomically races callback start and returns true only if it prevents a pending callback from starting.
- Cancellation is best-effort: it does not remove the block from libdispatch and cannot stop a callback after its start claim.
- Dropping the handle does not cancel the callback.
- A cancelled block is a no-op when dequeued. The queued callback otherwise receives the existing `MainThread` proof and retains panic containment.
- Do not add Swift, a dependency, a global registry, or an executor. Do not claim Simulator or device runtime behavior from compile/link evidence.

### Acceptance

- Host tests prove cancel-before-start suppression, start-before-cancel rejection, and one atomic winner under concurrent cancel/start.
- The registered `ios-runtime` capability runs those host tests and its native link script.
- Device iOS 10.0 and Simulator iOS 14.0 Release probes import public libdispatch async symbols, contain no Swift runtime imports, and retain their declared deployment floors.
- Docs distinguish cancellation of callback execution from removal of queued libdispatch work.

### Status

Implementation is in the dedicated B561 workstream. Runtime delivery and cancellation timing on Apple platforms remain unverified.
