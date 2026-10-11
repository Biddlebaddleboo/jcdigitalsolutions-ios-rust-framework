# iOS Native UI Controls

`ios-ui` implements the portable `framework-ui` contract with public UIKit bindings. It creates a
native `UIView`, adds native `UILabel`, `UIButton`, `UIImageView`, and single-line `UITextField`
children, and supports synchronous label, button, image, and field operations. `Frame` maps to
`CGRect` in the parent view's local coordinate system; its logical units are UIKit points

## Main-thread and ownership rules

- Construct `IosUiBackend` with `ios_runtime::main_thread::MainThread`; all UIKit creation and
  updates require the main thread. A failed runtime main-thread proof returns
  `framework_core::ErrorKind::Unavailable`. Backend and handle values are `!Send`/`!Sync`
- `IosView`, `IosLabel`, `IosButton`, `IosImageView`, and `IosTextField` retain their native UIKit
  objects. The caller owns hierarchy placement and must attach `IosView::native_view()` to a
  caller-owned controller or parent view
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

`native_view`, `native_label`, `native_button`, `native_image_view`, and `native_text_field` expose
borrowed typed UIKit references as
integration escape hatches. Any direct native mutation remains subject to UIKit's main-thread and
ownership rules

## Native stack composition

`IosUiBackend::add_stack_view` adds a `UIStackView` child to a caller-owned `IosView`. Choose a
horizontal or vertical `IosStackAxis`; spacing is expressed in UIKit points and must be finite.
Finite negative spacing is passed through to UIKit and can overlap arranged children.
`IosStackView::add_arranged_subview` passes a native `UIView` to UIKit, which retains and reparents
it as an arranged child. The `ios-ui` control handle remains caller-owned; keep an `IosButton` handle
alive for as long as its action should remain active. `set_spacing` updates UIKit's spacing, and
`native_stack_view` exposes the native stack for app-owned UIKit integration.

UIKit arranges only the stack's child views. The caller remains responsible for placing and resizing
the stack in its parent; this API adds no custom constraint engine, screen/root ownership, navigation,
or virtual view tree. Static checks do not establish visual layout behavior at runtime

```rust
let mut stack = backend.add_stack_view(
    &mut parent,
    stack_frame,
    IosStackAxis::Vertical,
    8.0,
)?;
let title = backend.add_label(&mut parent, child_frame, "Settings")?;
stack.add_arranged_subview(title.native_label())?;
```

## Single-line text input

`IosUiBackend::add_text_field` adds a caller-owned `UITextField` child with optional initial text and
placeholder. `IosTextField::text` and `placeholder` return owned Rust strings; their setters accept
`Option<&str>`. `None` maps to native `nil`, while `Some("")` maps to a present empty NSString.
Encoding uses the explicit-length NSString initializer. Reads copy the exact UTF-16 range and reject
malformed surrogate sequences as `InvalidInput`; embedded NUL and supplementary Unicode scalars
are preserved. No UTF-8 C-string conversion is used

`request_first_responder` and `resign_first_responder` return UIKit's `Bool` result. These values do
not prove keyboard visibility or text-input runtime behavior. The handle is retained and
`!Send`/`!Sync`; all operations require the current main thread. This slice adds no delegate,
callback, keyboard management, controller/window ownership, or runtime UI proof

```rust
let mut field = backend.add_text_field(&mut parent, frame, None, Some("Name"))?;
field.set_text(Some("Ada"))?;
let text: Option<String> = field.text()?;
let accepted = field.request_first_responder()?;
```

## Image display

`IosUiBackend::add_image_view` adds a native `UIImageView` child for a `framework_ui::Frame` and an
optional `&UIImage` from the app. `IosImageView::set_image` sets the image; `clear_image` sets `nil`.
The image reference lasts only for the call; UIKit owns each non-null value of `UIImageView.image`, and
the Rust handle holds no second image retain. The handle owns its `UIImageView`, is `!Send`/`!Sync`,
and exposes `native_image_view()` for UIKit use. Create, set, and clear calls need the current UIKit
main thread. This slice has no image decode, image file I/O, or runtime visual proof

```rust
let mut image_view = backend.add_image_view(&mut parent, frame, Some(&image))?;
image_view.set_image(Some(&replacement))?;
image_view.clear_image()?;
```

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

The B489 image-view link fixture checks the public UIKit framework, `UIImageView` class,
`setImage:` selector, and iOS 12.0 device / iOS 14.0 Simulator deployment targets. The B493
text-field fixture checks `UITextField`, nullable text/placeholder selectors, both responder
selectors, and the same target floors. Link passes are not runtime image display, visual proof,
keyboard behavior, or text-input proof

## Scope and evidence

This crate is a partial native-control backend, not a view hierarchy runtime. It does not own an
application, window, controller, scene, navigation stack, modal presenter, or layout engine. It
does not implement accessibility metadata, animations, gestures, tables, collections, multi-line
editing, delegates, keyboard management, or SwiftUI. The existing `ios-minimal` example demonstrates a separate app-entry vertical slice;
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
