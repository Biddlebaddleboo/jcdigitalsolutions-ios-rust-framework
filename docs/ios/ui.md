# iOS Native UI Controls

`ios-ui` implements the portable `framework-ui` contract with public UIKit bindings. It creates a
native `UIView`, adds native `UILabel` and `UIButton` children, and supports synchronous label-text
and button-title replacement. `Frame` maps to `CGRect` in the parent view's local coordinate
system; its logical units are UIKit points

## Main-thread and ownership rules

- Construct `IosUiBackend` with `ios_runtime::main_thread::MainThread`; all UIKit creation and
  updates require the main thread. A failed runtime main-thread proof returns
  `framework_core::ErrorKind::Unavailable`. Backend and handle values are `!Send`/`!Sync`
- `IosView`, `IosLabel`, and `IosButton` retain their native UIKit objects. The caller owns
  hierarchy placement and must attach `IosView::native_view()` to a caller-owned controller or
  parent view
- UIKit retains added child views in its native hierarchy. The Rust `IosButton` handle also owns
  the Rust `NSObject` target and action callback. Keep that handle alive while the button action
  should remain active; dropping it unregisters its target/action and releases Rust callback
  ownership even if the parent still displays the native button. An in-flight target callback keeps
  its target alive until return
- Borrowed text and title values are converted to owned `NSString` values during each call; Rust
  borrows do not outlive the call. Empty text/title values set an empty string
- UIKit target/action invokes the Rust callback synchronously on the main thread for
  `UIControlEvents::TouchUpInside`. No executor or deferred callback is used. A reentrant activation
  while the same `FnMut` callback is already borrowed is ignored
- The target stores one boxed `FnMut` closure per button; target/action dispatch adds no per-event
  allocation or framework registry. Avoid a strong callback capture of shared state that owns the
  same button handle, which would form an ownership cycle
- Rust callback panics are caught before return through Objective-C. The panic hook still runs;
  with `panic=abort`, the process aborts. No callback result is returned to UIKit or the caller

`native_view`, `native_label`, and `native_button` expose borrowed typed UIKit references as
integration escape hatches. Any direct native mutation remains subject to UIKit's main-thread and
ownership rules

## CoreGraphics frame intersection

`ios_ui::geometry::intersection(first, second)` calls the public CoreGraphics C functions
`CGRectIntersection`, `CGRectIsNull`, and `CGRectIsEmpty` for finite `framework_ui::Frame` values.
It returns `Ok(None)` for an empty or touching result, a positive-area `Frame` only when the native
result exactly matches the portable `Frame::intersection` contract, and
`framework_core::ErrorKind::Platform` if CoreGraphics returns an invalid or nonmatching result.
Unrepresentable input extents retain the portable `InvalidInput` error

The extern declarations are private to `ios-ui` and use the existing
`objc2-core-foundation` `CFCGTypes` `CGRect` layout; there is no public C ABI/header and no Swift
source or Swift runtime dependency. The functions have an SDK availability floor of iOS 2.0, but
this repository does not set the package's effective minimum iOS version. The link probe builds at
iOS 12.0 device and iOS 14.0 Simulator deployment targets; those are probe settings, not a product
minimum

## Scope and evidence

This crate is a partial native-control backend, not a view hierarchy runtime. It does not own an
application, window, controller, scene, navigation stack, modal presenter, or layout engine. It
does not implement accessibility metadata, animations, gestures, tables, collections, text input,
or SwiftUI. The existing `ios-minimal` example demonstrates a separate app-entry vertical slice;
this crate does not alter that example

Portable no-std compilation passed in the recorded validation. That record reports seven portable
unit tests passed, but the current `framework-ui` source has nine `#[test]` cases; this audit did not
run tests, so pass status for the current full set is unresolved. Root integration passes the
workspace check, host no-std link probe, and locked device/simulator `cargo check` plus strict
Clippy for `ios-ui` on Xcode 26.6 / SDK 26.5, below the Xcode 27.x baseline. The geometry link/import
probe adds C and Rust `CGRect` layout assertions and links the three CoreGraphics symbols for
device and Simulator. With `-Wl,-dead_strip_dylibs`, the probe's exact direct imports are
`CoreGraphics` and `libSystem.B.dylib`; this is geometry-probe evidence, not a claim that the full
UIKit controls package has no UIKit/Foundation dependencies. The probe does not execute the native
intersection function or establish runtime parity. These checks also do not prove simulator launch,
device behavior, target-action execution, visual layout, accessibility, or runtime callback behavior
