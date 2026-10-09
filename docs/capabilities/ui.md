# Portable UI Controls

`framework-ui` is a `no_std` contract for a small set of native controls: a container view, text
label, and button. It defines validated `Frame` values, static `UiBackend`, `LabelControl`, and
`ButtonControl` traits, and the generic `UiClient<B>` facade. The facade holds the backend supplied
by the caller; it does not discover a backend or start a runtime

## Frames

`Frame::new(x, y, width, height)` uses a parent-local origin and backend-defined logical units.
All four values must be finite; width and height must be non-negative. Zero-size frames are valid.
An invalid frame returns `framework_core::ErrorKind::InvalidInput`. A frame does not define
auto-layout, pixel rounding, insets, scaling, or intrinsic content size

`Frame::intersection(other)` applies only when both frames use the same coordinate space and logical
units. It returns `Ok(None)` for zero-size frames and for disjoint, edge-touching, or corner-touching
frames; a returned frame always has positive width and height. The calculation uses each maximum
edge as `origin + dimension`. A non-finite maximum edge, a positive dimension that cannot advance
its origin at the available `f64` precision, or an unrepresentable result returns
`framework_core::ErrorKind::InvalidInput`. No coordinate transform or tolerance is applied

## Text and callbacks

Labels and button titles are borrowed only for each synchronous call. A backend must copy or
convert each string to its native representation before return. Empty strings request empty text or
titles

`UiBackend::add_button` takes a generic `FnMut() + 'static` action. On success the returned button
handle owns the action; dropping that handle ends callback ownership. If creation returns an error,
the consumed callback is dropped. Callback thread, reentrancy, and panic behavior are backend
contracts. The portable API has no callback executor, result future, global registry, or
platform-specific object type

## iOS extension

The UIKit implementation is a separate `ios-ui` crate. Its `IosUiBackend` needs a typed
`ios_runtime::main_thread::MainThread` proof. See the [iOS UI guide](../ios/ui.md) for its
main-thread, callback, handle-lifetime, and UIKit escape-hatch rules

This is not a full UI framework. Navigation, scenes, general presentation, layout, accessibility,
virtual views, and application/window lifecycle are outside this contract. The portable
intersection operation is scalar geometry; it does not provide the broader CoreGraphics or
QuartzCore APIs in capability row 055
