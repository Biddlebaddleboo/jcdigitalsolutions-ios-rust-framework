# Portable UI Controls

`framework-ui` is a `no_std` contract for a small set of native controls: a container view, text
label, button, toggle, and single-line text field. It defines validated `Frame` values, static
`UiBackend`, `LabelControl`, `ButtonControl`, `ToggleControl`, and `TextFieldControl` traits, and the
generic `UiClient<B>` facade. The facade holds the backend supplied by the caller; it does not
discover a backend or start a runtime

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

`UiClient::add_text_field_with_mode` accepts `TextFieldInputMode::{Text, Password, Number, Decimal}`.
These are input/keyboard hints only: they do not parse, validate, or format text. The existing
`add_text_field` remains plain text, and backends that do not implement non-`Text` modes return
`Unsupported`

`TextFieldControl::set_submit_action` replaces the callback for a native submit event. The callback
receives a fresh owned `Result<Option<String>>` snapshot; `None` and `Some("")` remain distinct.
Backends that cannot report submit events return `Unsupported` and drop the new callback without
changing a prior action. Event source, thread, panic, and reentrancy behavior remain backend-defined

## iOS extension

The UIKit implementation is a separate `ios-ui` crate. Its `IosUiBackend` needs a typed
`ios_runtime::main_thread::MainThread` proof. See the [iOS UI guide](../ios/ui.md) for its
main-thread, callback, handle-lifetime, and UIKit escape-hatch rules. The iOS extension also exposes
UIKit's native `UIStackView` for vertical or horizontal arranged-subview composition. Its one-shot
edge helpers pin an attached stack either to its direct parent's bounds or to that parent's safe-area
guide; this adds no portable layout contract or custom layout engine

This is not a full UI framework. Navigation, scenes, general presentation, layout, accessibility,
virtual views, and application/window lifecycle are outside this contract. The portable
intersection operation is scalar geometry; it does not provide the broader CoreGraphics or
QuartzCore APIs in capability row 055

The iOS backend also exposes `IosScrollView` over public UIKit `UIScrollView`. It owns a zero-origin
content host that accepts existing iOS control handles and maps finite non-negative point dimensions
to the host frame and native `contentSize`. Its optional intrinsic vertical mode uses UIKit's
`contentLayoutGuide` and `frameLayoutGuide`; a one-shot `IosStackView` helper pins native stack edges
to a caller-owned parent so UIKit can size arranged dashboard content. This is an iOS extension only;
it adds no portable scroll contract or general layout system

The iOS backend also exposes a fixed-action `IosContextMenu` over public
`UIContextMenuInteraction`. It requires iOS 13 at runtime while preserving the crate's iOS 12
compile floor, and its handle removes the exact interaction on drop when the weakly held parent
still exists. This UIKit-only menu adds no portable menu contract, dynamic state binding, or
declarative view composition

The iOS backend also exposes a caller-framed `IosSelectableTextView` over public `UITextView`.
It is read-only, selectable, scrollable, and main-thread-bound; text snapshots preserve embedded
NUL and supplementary Unicode scalars and reject malformed UTF-16. Its native escape remains
available. This does not add a portable text-view contract or SwiftUI state binding; content and
copy policy remain with the app

The iOS backend also exposes `IosCollectionGrid`, a typed section/snapshot API backed by native
`UICollectionView` compositional layouts and diffable snapshots. Its reusable cell content covers
cards, metrics, actions, and switches; stable `u64` IDs feed typed caller-owned events. It remains an
iOS-only UIKit extension rather than a portable view, binding, or observation contract. Optional
`IosCollectionGridSectionHeader` values provide reusable text-only labels keyed by stable section ID;
they do not add arbitrary header composition or automatic state observation

For text-first content, `IosSectionedListView` reuses native `UITableView` subtitle cells across
caller-owned sections and optional UIKit header titles. Its full-snapshot API validates globally
unique stable row IDs before mutation; it is an imperative native list, not SwiftUI `List` or
automatic state observation
