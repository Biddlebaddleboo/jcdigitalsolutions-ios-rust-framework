# Optional iOS SpriteKit position C API

The opt-in `framework-c-api` Cargo feature `ios-spritekit` enables
`bindings/c/include/framework_ios_spritekit.h`. It wraps D64/B70's detached `SKNode` facade; the C
API exposes only create, get, set, and destroy for one node's parent-local position.

## Handle and thread contract

On iOS, every operation must run on the main thread. Create checks the thread before any SpriteKit
call; get and set check before reading the opaque handle. Off-main create/get/set returns
`FRAMEWORK_STATUS_UNAVAILABLE` after required output initialization for otherwise valid calls.
Invalid output pointers and non-finite create/set coordinates are rejected first. A successful
create returns one unique handle; do not copy it, use it concurrently, race destroy, or destroy an
alias.

Destroy is status-returning. Off-main it returns `FRAMEWORK_STATUS_UNAVAILABLE` without reading or
changing the pointer slot, so the host can retry on main. On main, a null slot or null handle is a
no-op with `FRAMEWORK_STATUS_OK`; a live slot is cleared before the Rust owner is dropped. The
handle owns one `IosSpriteNode`; no `SKNode`, `NativeSkNode`, `Retained<T>`, or Objective-C pointer
crosses the C boundary.

## Coordinates and statuses

Coordinates are finite parent-local doubles whose unit comes from the host scene. Create and set
reject NaN and infinity with `FRAMEWORK_STATUS_INVALID_ARGUMENT`. Get requires two distinct output
pointers, initializes each non-null output to `0.0`, and writes values only on
`FRAMEWORK_STATUS_OK`. Create initializes its output handle to null before validation. Rust panics
are contained as `FRAMEWORK_STATUS_PANIC`; backend errors map through `FrameworkStatus::from_error`.

On non-iOS targets, valid operations return `FRAMEWORK_STATUS_UNSUPPORTED`; the destroy stub does
not inspect or change its slot. There is no mock node and no SpriteKit dependency in the host
feature graph. Invalid required outputs and non-finite coordinates are rejected before the
unsupported result. The feature is off by default.

The B70 API floor is iOS 7.0. F11's SDK-specific Release link probes target device 12.0 and
Simulator 14.0, based on the active SDK and repository Simulator baseline. Those probe floors do not
raise the API floor. The probes establish symbol/import and deployment metadata only; they are
compiled and linked, never run. The API does not create or attach an `SKScene`/`SKView`, render,
load assets, animate, access physics, or claim parity or performance.

See [`docs/ios/spritekit-node-position.md`](../ios/spritekit-node-position.md) for B70's Rust facade
and platform evidence, and [`PLAN_BINDINGS_SPRITEKIT.md`](../../PLAN_BINDINGS_SPRITEKIT.md) for the
F11 ABI and validation record.
