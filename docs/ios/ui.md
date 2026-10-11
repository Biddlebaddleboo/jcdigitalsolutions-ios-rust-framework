# iOS Native UI Controls

`ios-ui` implements the portable `framework-ui` contract with public UIKit bindings. It creates a
native `UIView`, adds native `UILabel`, `UIButton`, `UIImageView`, `UIScrollView`, and single-line
`UITextField` children, and supports synchronous label, button, image, scroll-host, and field
operations. `Frame` maps to
`CGRect` in the parent view's local coordinate system; its logical units are UIKit points

## Main-thread and ownership rules

- Construct `IosUiBackend` with `ios_runtime::main_thread::MainThread`; all UIKit creation and
  updates require the main thread. A failed runtime main-thread proof returns
  `framework_core::ErrorKind::Unavailable`. Backend and handle values are `!Send`/`!Sync`
- `IosView`, `IosLabel`, `IosButton`, `IosImageView`, `IosScrollView`, and `IosTextField` retain
  their native UIKit objects. The caller owns hierarchy placement and must attach
  `IosView::native_view()` to a caller-owned controller or parent view
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

## Native scrolling

`IosUiBackend::add_scroll_view` adds a public UIKit `UIScrollView` and an owned zero-origin `UIView`
content host to a caller-owned parent. Add labels, fields, stacks, and other existing controls to
`IosScrollView::content_view_mut()` through the same backend methods used for other `IosView`
values. `set_content_size(width, height)` accepts finite non-negative UIKit point dimensions only;
it returns `InvalidInput` for negative or non-finite values and does not clamp accepted values. It
sets both the content host frame at `(0, 0)` and UIKit `contentSize` to the exact dimensions. The
caller positions child views and chooses the total content size. `native_scroll_view` exposes a
typed UIKit escape. All creation and mutation require the main thread, and the retained handle is
`!Send`/`!Sync`

```rust
let mut scroll = backend.add_scroll_view(&mut parent, scroll_frame)?;
let title = backend.add_label(
    scroll.content_view_mut(),
    child_frame,
    "Status",
)?;
scroll.set_content_size(240.0, 1024.0)?;
let native_scroll_view = scroll.native_scroll_view();
```

`enable_intrinsic_vertical_layout()` is a separate one-shot mode. It pins the retained host to
UIKit's `contentLayoutGuide` and its width to `frameLayoutGuide`, allowing a vertical stack and its
arranged subviews to determine the content height through UIKit Auto Layout. Pin an already-added
stack with `IosStackView::pin_edges_to_parent(content_view)`. The stack must still be a direct child
of that exact parent, and its autoresizing-mask translation must not already be disabled. Both
helpers disable autoresizing-mask translation on the constrained view. Give arranged children
intrinsic or explicit heights and a complete vertical constraint chain; this API adds no custom
layout engine. Repeated constraint setup and switching between intrinsic and explicit sizing return
`InvalidInput`; repeated explicit `set_content_size` updates remain valid. Native escape-hatch
constraints remain caller-managed and must not duplicate or conflict with these constraints

The native compile/link probe establishes public class and selector linkage for device and Simulator
targets. It does not establish runtime scrolling, clipping, gesture behavior, indicator behavior, or
visual layout. Its intrinsic-layout additions also do not establish runtime content measurement or
visual constraint resolution

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
keyboard management, controller/window ownership, or runtime UI proof

`IosView::end_editing()` forces UIKit to end editing in that view or one of its descendants through
the public `UIView.endEditing(true)` API. It searches only the passed view hierarchy and is not a
process-wide responder search. `Ok(true)` means UIKit reports that the scoped view resigned first
responder status; `Ok(false)` means it did not. This result does not establish that an on-screen
keyboard was visible or disappeared. The method requires the current main thread; the existing
`IosTextField::resign_first_responder()` remains the narrower, field-specific operation

```rust
let mut field = backend.add_text_field(&mut parent, frame, None, Some("Name"))?;
field.set_text(Some("Ada"))?;
let text: Option<String> = field.text()?;
let accepted = field.request_first_responder()?;
let ended = parent.end_editing()?;
```

`framework_ui::TextFieldControl` and `UiClient::add_text_field` add a portable user-edit callback.
Each callback receives `Result<Option<String>>` with a fresh owned snapshot for that event; `None`
and `Some("")` remain distinct, and embedded NUL and supplementary Unicode values are preserved.
Malformed native UTF-16 reaches the callback as `InvalidInput`, never replacement text. Calling
`set_text` is programmatic and must not invoke the callback

On iOS the callback uses public `UITextField` target/action for
`UIControlEvents::EditingChanged`, synchronously on UIKit's main thread. The callback is not
`Send`; panics are caught before returning through Objective-C and reentrant calls while the same
`FnMut` is active are ignored. The returned `IosTextField` handle retains the target and unregisters
only its exact selector and event on drop, even if the parent still retains the native field. The
existing `IosUiBackend::add_text_field` no-callback API remains available and keeps plain-text
behavior

`UiClient::add_text_field_with_mode` adds `TextFieldInputMode::{Text, Password, Number, Decimal}`.
The iOS backend applies `UITextInputTraits::setSecureTextEntry` and `setKeyboardType` before
attaching the field: `Text` uses the default keyboard without secure entry, `Password` uses secure
entry with the default keyboard, and `Number` / `Decimal` select `UIKeyboardType::NumberPad` /
`DecimalPad`. These modes are hints, not numeric parsers or validators; initial and edited values
remain exact text, and creating a field does not emit an edit callback. Other backends may return
`Unsupported` for modes they do not implement. No submit/return handling, autoformatting, keyboard
management, or application state is added. Host regressions and device / Simulator compile-link
checks do not prove live keyboard selection, secure rendering, or callback delivery

## Boolean toggle

`framework_ui::ToggleControl` exposes the current boolean state and a programmatic `set_state` operation; `UiBackend::add_toggle` accepts an initial value and a callback that receives the new value after a user/native change. A programmatic `set_state` must not invoke that callback. Callback ownership lasts as long as the returned control handle, while callback thread, panic, and reentrancy policy are backend-defined

On iOS, `IosUiBackend::add_toggle` creates and attaches a public UIKit `UISwitch`. `IosToggle::is_on` reads its state, `set_state` updates without animation, and `set_on(on, animated)` forwards the caller's animation choice to UIKit `setOn:animated:`. UIKit does not send a `ValueChanged` action for this programmatic setter. A user change invokes the callback synchronously on the main thread with `UISwitch.isOn`; Rust panics are caught before returning through Objective-C, and a reentrant callback is ignored

`IosToggle` retains the switch and is `!Send`/`!Sync`. Keep its handle alive while the callback should remain active. Dropping the handle unregisters only its own `toggleChanged:` action for `UIControlEvents::ValueChanged`; the parent may continue to retain and display the native switch. `native_switch` provides a borrowed typed UIKit escape; direct native changes remain subject to UIKit's main-thread and ownership rules

```rust
let mut toggle = backend.add_toggle(&mut parent, frame, false, |on| {
    // Apply application-owned state
})?;
let enabled = toggle.is_on()?;
toggle.set_on(true, true)?;
```

Host contract tests and target compile/link checks do not prove UIKit interaction, switch rendering, animation, callback delivery, or runtime behavior

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
text-field fixture checks `UITextField`, nullable text/placeholder selectors, both field responder
selectors, the view-scoped `endEditing:` selector, and the same target floors. Link passes are not
runtime image display, visual proof, keyboard behavior, or text-input proof

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
