# PLAN_IOS_UI.md — Workstream D14: Reusable Native UI Controls

## Status

D14 adds a portable no-std frame/control contract and a bounded UIKit backend for native views,
labels, and buttons. The iOS backend uses a Rust-defined UIControl target/action receiver and keeps
its callback in the returned button handle. The historical validation records list three portable
tests here and seven passing tests in `docs/ios/ui.md`, while the current
`crates/framework-ui/src/lib.rs` has nine `#[test]` cases. This audit did not run tests, so the pass
status for the current full set remains unresolved. Root integration also records
`cargo +1.94.1 check --workspace`, the 20-crate host no-std link probe, device/simulator `cargo
check` and strict Clippy for `ios-ui`, docs-check, and format checks. The target checks establish
compile/lint only, not UIKit link or runtime. The API exposes a callback, but no live callback
execution, visual behavior, scene lifecycle, navigation, or general layout system is claimed

B489 adds `IosImageView` to `ios-ui`: `add_image_view` takes an app `UIImage` ref; `set_image` and
`clear_image` update `UIImageView.image`; `native_image_view` gives the typed UIKit escape. UIKit
owns each non-null image. No decode or file I/O is in scope. The registered `ios-ui` profile passes
64 checks with zero failures or blocks and two skips for simulator execution and physical-device
proof. Device and Simulator link probes use iOS 12.0 / 14.0 floors. A negative mutation that removes
the image-specific main-thread proof fails its source guard. No runtime image proof is claimed

B493 adds a caller-owned `UITextField` child. `IosTextField` reads and writes nullable text and
placeholder values without C-string conversion, copies NSString's explicit UTF-16 range, preserves
embedded NUL and supplementary Unicode values, and rejects malformed surrogate data. Its typed
native escape and UIKit first-responder requests remain main-thread-bound; responder methods return
UIKit's `Bool` only. Host tests and the registered profile cover conversion, API guards, target
compilation, and selector linkage. No delegate, callback, keyboard, simulator execution, or physical
device text-input behavior is claimed

B500 adds a thin `IosStackView` wrapper for UIKit's built-in `UIStackView`, with horizontal or
vertical axis, finite point spacing, arranged `UIView` insertion, and a typed native escape. UIKit
owns arrangement of those children; callers still place and size the stack in their parent and keep
behavior-owning control handles alive. This is not a general layout engine: UIKit applies its native
stack layout, while this API adds no custom layout engine or parent constraints. Navigation,
controller/root ownership, virtual tree, and runtime visual claims remain out of scope. Host spacing
tests cover finite values and reject NaN/infinities. The registered `ios-ui` profile passes 152 checks
with 0 failures, 0 blocked, and 2 skipped for simulator execution and physical-device proof. Device
and Simulator compile/link checks verify API use, symbols, imports, and deployment floors only; they
do not prove runtime visual layout

B507 adds an opt-in intrinsic vertical sizing mode to `IosScrollView` using UIKit's public
`contentLayoutGuide` and `frameLayoutGuide`, plus `IosStackView::pin_edges_to_parent` to constrain a
stack to its direct `IosView` parent. Explicit content sizing remains unchanged; sizing modes cannot
be mixed, Auto Layout setup is one-shot, and host policy tests cover invalid transitions. The
registered device and Simulator link probes check public guides, anchors, and activation selectors;
they do not prove runtime constraint resolution, scrolling, or visual layout

## Objective

Provide a small reusable Rust API for native container views, labels, buttons, image views, and
single-line text fields, with a Rust target-action callback for buttons. Keep portable semantics in
`framework-ui` and iOS objects in `ios-ui`

## Dependencies

- `framework-core` for stable portable errors
- `ios-runtime` for the typed iOS main-thread proof and foreign-callback panic containment
- Existing workspace-pinned `objc2`, `objc2-foundation`, `objc2-core-foundation`, and `objc2-ui-kit`
  dependencies; do not add or change workspace versions

## Write scope

- `PLAN_IOS_UI.md`
- `crates/framework-ui/**`
- `platform/ios/ios-ui/**`
- `docs/capabilities/ui.md`
- `docs/ios/ui.md`

Do not edit root `Cargo.toml`, `Cargo.lock`, `PLAN.md`, `PLAN_CAPABILITIES.md`, capability status
manifest or README, CI, the minimal example, B8 accessibility, B9 presentation, or unrelated crates.
Root owns workspace, lockfile, docs-index, capability-matrix, and CI integration

## Portable contract

- Add a genuine `#![no_std]` crate with no unsafe code and only a `framework-core` dependency
- Define a finite `Frame` with non-negative width and height, local origin, and backend-defined
  logical units. Reject non-finite fields and negative dimensions as `InvalidInput`
- Define static `UiBackend`, `LabelControl`, and `ButtonControl` contracts plus a thin generic
  `UiClient<B>` facade. Do not use runtime backend discovery, boxed backend traits, or a registry
- Keep labels and button titles borrowed for each synchronous call; backends must copy or retain
  their own native representation before the call returns
- Make button actions generic `FnMut() + 'static` callbacks. The returned button handle owns the
  action for its lifetime; dropping the handle ends its callback ownership. A failed add consumes
  and drops the supplied callback
- Do not put UIKit, Objective-C, Swift, Android, C ABI, view-controller, executor, or global
  runtime types in the portable contract

## iOS backend contract

- Add `ios-ui`, compiled only for iOS, and require `ios_runtime::main_thread::MainThread` at backend
  construction. All UIKit create/update/drop and action-callback work stays on the main thread;
  backend and handles remain `!Send`/`!Sync`
- Map `Frame` to `CGRect` in the parent view's local coordinate system using UIKit points
- Create a native `UIView`; add native `UILabel` and `UIButton` children; support text/title updates
- Add `IosTextField` over public `UITextField` with nullable text/placeholder, explicit-length
  NSString encoding, strict UTF-16 decoding, and main-thread first-responder requests returning
  UIKit's Boolean result
- Implement button actions with a Rust-defined `NSObject` target and `UIControl` target/action. The
  returned `IosButton` strongly retains both the native button and its Rust target; the target
  catches callback panics before return through Objective-C. The panic hook still runs, and
  `panic=abort` remains fatal. Dropping `IosButton` unregisters only its own target/selector/event
  before target release; an in-flight callback retains its target through return. Store one boxed
  `FnMut` closure per button; do not add a per-event allocation or process registry
- A callback runs synchronously on UIKit's main thread when UIKit sends `TouchUpInside`. No
  executor, deferred dispatch, or result value exists; after handle drop, later activations cannot
  call the removed target/action
- Permit access to the typed native view/control references for integration with caller-owned
  UIKit controllers. Do not own a controller, window, scene, app delegate, or view hierarchy root
- No Swift source, global framework registry/runtime, generic navigation, modal presentation,
  scene management, auto-layout engine, accessibility system, or virtual view tree

## Documentation and tests

- Document frame validation and units, borrowed-string copy/lifetime semantics, control-handle
  ownership, callback thread/reentrancy/panic behavior, and native escape hatches
- Add portable tests for valid and invalid frame values and a fake static backend path. Do not
  simulate UIKit action delivery or claim live UI behavior
- Mark this as a minimal native-control slice. Text fields add no delegate, callback, keyboard, or
  runtime text-input contract. The crate does not claim navigation, scene lifecycle, general
  presentation, layout, accessibility, or parity with a complete UI framework

## Validation and handoff

- Do not run Cargo while G's portable no-std final-link proof is active; root will integrate the
  package and run workspace checks after this slice
- After integration, run portable no-default-features checks, iOS device/simulator checks, strict
  Clippy, formatting, docs-check, zero-Swift-source, and diff checks
- Until those gates run, report only source-level review and any direct non-Cargo checks. Do not
  claim Xcode compilation, UIKit linking, simulator launch, device behavior, callback execution, or
  visual/layout correctness
