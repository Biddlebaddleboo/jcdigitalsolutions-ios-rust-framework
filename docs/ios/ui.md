# iOS Native UI Controls

`ios-ui` implements the portable `framework-ui` contract with public UIKit bindings. It creates a
native `UIView`, adds native `UILabel`, `UIButton`, `UIImageView`, `UIScrollView`, `UIStackView`,
`UITableView`, `UIContextMenuInteraction`, single-line `UITextField`, and multiline `UITextView`
children, and supports synchronous label, button, image, scroll-host, field, selectable-text,
editable-text, disclosure-group, reusable-list, and fixed context-menu operations. `Frame` maps to
`CGRect` in the parent view's local coordinate system; its logical units are UIKit points

## Main-thread and ownership rules

- Construct `IosUiBackend` with `ios_runtime::main_thread::MainThread`; all UIKit creation and
  updates require the main thread. A failed runtime main-thread proof returns
  `framework_core::ErrorKind::Unavailable`. Backend and handle values are `!Send`/`!Sync`
- `IosView`, `IosLabel`, `IosButton`, `IosImageView`, `IosScrollView`, `IosTextField`,
  `IosSelectableTextView`, `IosTextEditor`, `IosListView`, and `IosSectionedListView` retain
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
- `IosListView` also retains its Rust data-source/delegate target because UIKit holds those
  references weakly. Keep the list handle alive while its table should provide rows and selection;
  dropping it releases callback ownership and UIKit clears the weak target references

`native_view`, `native_label`, `native_button`, `native_image_view`, and `native_text_field` expose
borrowed typed UIKit references as
integration escape hatches. Any direct native mutation remains subject to UIKit's main-thread and
ownership rules

## System text styles and semantic colors

`IosLabel`, `IosButton`, and `IosTextField` accept the same `IosSystemTextStyle` through
`set_system_text_style`. The styles map to UIKit's public `UIFontTextStyle` values and
`UIFont.preferredFontForTextStyle`; each native text object enables UIKit content-size-category
adjustment. This keeps Dynamic Type sizing in UIKit and adds no custom font family or text engine.

`set_semantic_text_color` accepts `IosSemanticTextColor::{Primary, Secondary, Blue, Red, Green,
Orange}` on those same controls. Primary and secondary use UIKit's dynamic `labelColor` and
`secondaryLabelColor` on iOS 13 and later; the iOS 12 fallback is black and gray. Named accent
colors use UIKit's public system colors. Button title color applies to the normal control state.
The registered device/Simulator link probe verifies the required UIKit selectors at iOS 12.0 and
iOS 14.0; static checks do not establish visible appearance, dark-mode adaptation, or Dynamic Type
behavior at runtime

## Font weight and monospaced design

`IosLabel`, `IosButton`, and `IosTextField` expose `set_font_weight` with
`IosFontWeight::{Regular, Bold}` and `set_font_design` with
`IosFontDesign::{Default, Monospaced}`. These setters transform the control's current public
`UIFontDescriptor` using UIKit's symbolic bold and monospaced traits, preserving unrelated traits
and the current descriptor size. A missing font or a descriptor conversion UIKit cannot satisfy
returns `Unavailable` without replacing the current font. The symbolic traits and descriptor APIs
are available from iOS 7, so they fit the existing iOS 12.0 device floor; the implementation does
not call the iOS 13-only `fontDescriptorWithDesign:` API.

Set a preferred text style first when the font should track Dynamic Type, then apply weight or
design:

```rust,ignore
label.set_system_text_style(IosSystemTextStyle::Caption)?;
label.set_font_design(IosFontDesign::Monospaced)?;
count_label.set_font_weight(IosFontWeight::Bold)?;
```

UIKit's content-size-category adjustment remains enabled by the preferred-style setter. Device
and Simulator Release link probes verify descriptor selectors and deployment floors; no live font
rendering or Dynamic Type update behavior is claimed. Only regular/bold weight and default/mono
symbolic traits are exposed; other weight grades and font families remain unsupported

## Button accessibility metadata

`IosButton::set_accessibility_label`, `set_accessibility_hint`, and
`set_accessibility_identifier` set UIKit's public `UIAccessibility` and
`UIAccessibilityIdentification` metadata on a native button. Calls require the same main-thread
proof as other button updates. Rust UTF-8 values become length-aware `NSString` values; UIKit copies
them. `None` writes native `nil`, while `Some("")` remains an explicitly empty string. An
identifier is not user-facing and should be unique in the app's accessibility hierarchy; the
framework does not enforce uniqueness. These methods do not add a portable accessibility tree or
prove VoiceOver announcements. Host regressions cover nil versus empty and embedded NUL/Unicode
preservation; Device and Simulator link probes verify the three UIKit selectors at the existing
iOS 12.0 and iOS 14.0 floors. Live assistive-technology behavior remains unverified

## Enabled state for controls

`IosButton`, `IosToggle`, and `IosTextField` expose `is_enabled` and `set_enabled` through both
their typed handles and the portable `ButtonControl`, `ToggleControl`, and `TextFieldControl`
traits. The setters call UIKit's `UIControl.isEnabled` state on the main thread. A disabled native
control rejects user interaction according to UIKit; programmatic text/toggle updates remain
available and silent. Changing enabled state does not synthesize button, toggle, text-edit, or
submit callbacks. A backend without enabled-state support returns `Unsupported` from the trait's
default methods

```rust,ignore
button.set_enabled(!request_in_flight)?;
toggle.set_enabled(form_is_valid)?;
text_field.set_enabled(!saving)?;
```

This is explicit control state, not SwiftUI environment propagation: callers choose each control's
enabled value. The registered host regression checks disabled input and callback silence; Device and
Simulator probes compile and link the getter and setter selectors. They do not demonstrate runtime
UIKit interaction on a Simulator or physical device

## Native stack composition

`IosUiBackend::add_stack_view` adds a `UIStackView` child to a caller-owned `IosView`. Choose a
horizontal or vertical `IosStackAxis`; spacing is expressed in UIKit points and must be finite.
Finite negative spacing is passed through to UIKit and can overlap arranged children.
`IosStackView::add_arranged_subview` passes a native `UIView` to UIKit, which retains and reparents
it as an arranged child. The `ios-ui` control handle remains caller-owned; keep an `IosButton` handle
alive for as long as its action should remain active. Typed `set_alignment` and `set_distribution`
methods select UIKit's cross-axis alignment and main-axis distribution; baseline alignment is valid
only for horizontal stacks. `remove_arranged_subview` removes UIKit's layout management but leaves
the view visible in the stack's subview hierarchy. `set_spacing` updates UIKit's spacing, and
`native_stack_view` exposes the native stack for app-owned UIKit integration. These operations are
main-thread-bound.

UIKit arranges only the stack's child views. The caller remains responsible for placing and resizing
the stack unless it selects one of the one-shot edge helpers: `pin_edges_to_parent` aligns all four
edges with the direct parent's bounds, while `pin_edges_to_safe_area` aligns them with that parent's
public `safeAreaLayoutGuide`. Both require the stack to remain a direct child with autoresizing-mask
translation enabled, and only one may be used. Invalid preflight returns `InvalidInput` before
constraints are activated. This adds no custom constraint engine, screen/root ownership, navigation,
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
stack.set_alignment(IosStackAlignment::Leading)?;
stack.set_distribution(IosStackDistribution::FillProportionally)?;
stack.pin_edges_to_safe_area(&parent)?;
```

## Equal-column grid

`IosUiBackend::add_grid_view` creates a UIKit `UIStackView` containing horizontal row stacks. The
caller chooses a nonzero column count, finite nonnegative row and column spacing, frame, and native
child views. `IosGridView::add_arranged_subview` adds each view eagerly in row-major order; UIKit
retains it, and the caller keeps any behavior-owning handle alive. Incomplete final rows use
noninteractive, non-accessibility placeholder views to preserve equal column widths. The caller
still owns parent placement and sizing.

This is a lower-level UIKit layout helper, not a SwiftUI-style declarative UI. It has no view tree,
state diffing, bindings, `ForEach`, lazy creation, or cell reuse. App composition and state updates
remain caller-owned. Host tests reject zero columns and negative or non-finite spacing; device and
Simulator checks establish compilation and linkage, not runtime layout or accessibility behavior

## Reusable UIKit list

`IosUiBackend::add_list_view` adds a native `UITableView` with an owned row snapshot. Each
`IosListRow` has a unique `u64` identity, a title, and optional detail text. UIKit dequeues and
reuses subtitle cells; the detail label is reset on each bind. A synchronous selection callback
receives the current row's stable identity, not its temporary index. Call `replace_rows` to validate
and replace the full snapshot; duplicate identities or a count UIKit cannot represent fail without
changing the existing rows. `native_table_view` exposes the UIKit table for additional configuration.

```rust,ignore
let mut list = backend.add_list_view(
    &mut parent,
    list_frame,
    vec![IosListRow::new(7, "Peer").with_detail("Connected")],
    |row_id| { /* handle selection */ },
)?;
list.replace_rows(updated_rows)?;
```

This adds native cell reuse, which `IosGridView` does not provide, but it is still not SwiftUI's
declarative `List`: callers own state, update timing, row meaning, and selection policy, and
`replace_rows` reloads a complete snapshot rather than diffing observed state. There are no
bindings, automatic observation, or declarative view tree. Host regressions cover identity,
transactional replacement, panic containment, and callback reentry. Device/Simulator compile-link
checks verify UIKit symbols and deployment floors only; they do not establish live row rendering,
selection, scrolling, or accessibility behavior

## Reusable sectioned UIKit list

`IosUiBackend::add_sectioned_list_view` adds a native `UITableView` whose `IosListSection` values
own optional nonempty header titles and ordered `IosListRow` snapshots. `IosListSection::new` adds a
native text header; `IosListSection::untitled` creates a group without header text. Row IDs remain
unique across the complete snapshot, and the synchronous selection callback
receives the selected row's stable `u64` ID. UIKit reuses the same subtitle cells as `IosListView`.

```rust,ignore
let mut list = backend.add_sectioned_list_view(
    &mut parent,
    frame,
    vec![IosListSection::new(
        "Available",
        vec![IosListRow::new(7, "Peer").with_detail("Nearby")],
    )],
    |row_id| select_peer(row_id),
)?;
list.replace_sections(updated_sections)?;
```

Replacement validates all section titles, UIKit-representable section and per-section row counts,
and globally unique row IDs before mutation. A failed replacement preserves the previous snapshot;
the accepted replacement reloads the full table rather than diffing. The existing `IosListView` and
`replace_rows` API remain the flat-list path. This is still an imperative UIKit snapshot primitive,
not SwiftUI `List`/`Section`: there are no view builders, bindings, automatic state observation, or
declarative row composition. Host tests cover duplicate IDs across sections, invalid titles, and
transactional replacement; native compile/link checks do not prove section-header rendering,
selection, scrolling, or accessibility at runtime

## Reusable sectioned UIKit collection

`IosUiBackend::add_collection_grid` creates a native `UICollectionView` with fixed-column,
adaptive-column, or horizontal-carousel sections. Each section owns an ordered set of items with
unique `u64` identities. `IosCollectionGridItem` supports reusable typed card, metric, action, and
toggle content; UIKit reuses one native cell class with labels, a button, and a `UISwitch`. Action
and switch enabled state is explicit. IDs remain unsigned 64-bit values through UIKit diffable
snapshots, and selection/action/toggle callbacks report stable IDs rather than index paths

`IosCollectionGridSectionHeader::new(section_id, title)` adds an optional text-only section label
without changing `IosCollectionGridSection`. `add_collection_grid_with_headers` creates a collection
with headers keyed by stable section ID, and `replace_snapshot_with_headers` replaces them with the
data snapshot. Duplicate or unknown section IDs and empty titles return `InvalidInput` before native
mutation. A title change reloads its stable section's supplementary view; item IDs and event semantics
remain unchanged

```rust,ignore
let mut dashboard = backend.add_collection_grid(&mut parent, frame, sections, |event| match event {
    IosCollectionGridEvent::Selected { item_id } => select(item_id),
    IosCollectionGridEvent::ActionTriggered { item_id } => activate(item_id),
    IosCollectionGridEvent::ToggleChanged { item_id, is_on } => request_toggle(item_id, is_on),
})?;
dashboard.replace_snapshot(updated_sections, true)?;
```

```rust,ignore
let mut dashboard = backend.add_collection_grid_with_headers(
    &mut parent,
    frame,
    sections,
    vec![IosCollectionGridSectionHeader::new(1, "Recent trips")],
    |_| {},
)?;
dashboard.replace_snapshot_with_headers(
    updated_sections,
    vec![IosCollectionGridSectionHeader::new(1, "Today's trips")],
    true,
)?;
```

The callback is an event request, not an automatic state binding. A toggle reverts to the current
snapshot value after user input; the caller applies a new snapshot to commit updated state. Snapshot
replacement validates all section/item IDs and layout dimensions before changing state, and existing
IDs with changed typed content reload. UIKit applies one diff per call. To avoid overlapping UIKit
updates, a replacement while any prior apply is in flight returns `Unavailable` without mutation;
retry after its completion. The handle exposes `native_collection_view` for configuration but callers
must not replace its data source, delegate, layout, or registered cell identifier

This uses typed data and explicit snapshots over UIKit's native composition; it is not SwiftUI-style
declarative view composition. The caller owns app state and decides when to submit a complete snapshot;
there is no arbitrary view builder or custom section-header composition, `Binding`, automatic state observation, or
virtual view tree. `IosGridView` remains the simpler eager `UIStackView` helper. Compositional layout
and diffable data sources require iOS 13; the package's iOS 12 device and iOS 14 Simulator compile
floors remain, and runtime creation below iOS 13 returns `Unavailable`. Host policy tests and static
device/Simulator link checks do not establish live rendering, animation, input, or accessibility
behavior

## Fixed UIKit context menus

`IosUiBackend::add_context_menu` attaches a `UIContextMenuInteraction` to a caller-owned `UIView`
with an owned, fixed list of `IosContextMenuAction { id, title }` values. IDs must be unique and
titles nonempty; invalid input returns `InvalidInput` before attachment. The selection callback
receives the selected `u64` ID synchronously on UIKit's main thread. Panics are caught before the
Objective-C callback returns, and reentry while the `FnMut` callback is borrowed is ignored

Context menus require iOS 13.0. The package retains its iOS 12.0 compile floor; on iOS 12 the
runtime availability guard returns `Unavailable` before creating or attaching an interaction.
`IosContextMenu` retains the interaction and delegate target, holds only a weak parent reference,
and on drop disables callbacks and removes that exact interaction if the parent still exists. Keep
the handle alive while its menu should remain active; `native_interaction` is a typed UIKit escape

This is a fixed native menu, not SwiftUI `.contextMenu`: it has no dynamic action replacement,
custom preview, bindings, or state observation. Action meaning and operations such as copying text
remain app-owned. Host regressions cover invalid menus/IDs, action identity, callback panic, and
reentry. Device/Simulator link probes verify selectors and deployment floors but do not establish
menu presentation, selection, or runtime behavior

## Selectable multiline text

`IosUiBackend::add_selectable_text_view` adds a caller-framed, scrollable `UITextView` with editing
disabled and selection enabled. `IosSelectableTextView::text` returns a strict UTF-16 snapshot;
`set_text` replaces the content without enabling editing. Embedded NUL and supplementary Unicode
scalars are preserved, while malformed native UTF-16 returns `InvalidInput`. The handle is
main-thread-bound and `native_text_view` provides a typed UIKit escape. Content and copy behavior
remain caller-owned

This is a native text-selection primitive, not SwiftUI `.textSelection` state binding. It does not
observe application state, provide a custom copy menu, or define clipboard policy. Host tests cover
length-aware text and malformed-surrogate regressions. Device/Simulator compilation and link probes
verify the UIKit type, selectors, imports, and deployment floors; they do not establish live
selection, menu, or clipboard behavior

`IosUiBackend::add_text_editor` adds an editable, selectable, scrollable `UITextView` with an
initial Rust string and synchronous edit callback. `IosTextEditor::text` returns an owned strict
UTF-16 snapshot; `set_text` replaces it without an edit callback. The callback receives exact
embedded NUL and supplementary Unicode values, while malformed UTF-16 reaches it as
`InvalidInput`. Panics are contained, reentrant callback delivery is ignored, and replacing an
active callback returns `Unavailable`. The main-thread-bound handle owns the native view and its
delegate, detaches its delegate on drop, and exposes typed UIKit integration through
`native_text_view`. This is an imperative UIKit control, not a SwiftUI state-binding or rendering
system; layout, application state, and editor lifetime remain caller-owned. Device/Simulator
compile-link evidence verifies `UITextViewDelegate` and its edit selector but does not prove live
keyboard, text-edit, or callback behavior

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

`IosTextField::set_accessibility_label`, `set_accessibility_hint`, and
`set_accessibility_identifier` set the field's public UIKit accessibility metadata on the main
thread. Each accepts `Option<&str>`: `None` clears the native value, while `Some("")` sets an empty
string. UTF-8 text, including embedded NUL, is copied through length-aware `NSString` conversion.
Identifiers should be unique in the accessibility hierarchy; uniqueness is not enforced. Host
conversion tests cover clear/empty distinction and Unicode/NUL preservation, and the existing
device/Simulator link fixture checks the three selectors at iOS 12.0 / iOS 14.0. These static checks
do not prove VoiceOver announcements or other live assistive-technology behavior

`IosView::end_editing()` forces UIKit to end editing in that view or one of its descendants through
the public `UIView.endEditing(true)` API. It searches only the passed view hierarchy and is not a
process-wide responder search. `Ok(true)` means UIKit reports that the scoped view resigned first
responder status; `Ok(false)` means it did not. This result does not establish that an on-screen
keyboard was visible or disappeared. The method requires the current main thread; the existing
`IosTextField::resign_first_responder()` remains the narrower, field-specific operation

`IosUiBackend::send_resign_first_responder_action()` asks `UIApplication` to send the public
[`UIResponder.resignFirstResponder`](https://developer.apple.com/documentation/uikit/uiresponder/resignfirstresponder%28%29?language=objc)
action to a nil target, so UIKit searches the current responder chain without a view or text-field
handle. It requires the current main thread and returns `Result<bool>` containing
[`UIApplication.sendAction:to:from:forEvent:`](https://developer.apple.com/documentation/uikit/uiapplication/sendaction%28_%3Ato%3Afrom%3Afor%3A%29?language=objc)'s
Boolean unchanged. That Boolean does not report keyboard visibility or disappearance. This
app-level responder-chain action is distinct from `IosView::end_editing()`, which searches one view
subtree, and `IosTextField::resign_first_responder()`, which targets one field

## Activity indicator

`IosUiBackend::add_activity_indicator` adds a medium `UIActivityIndicatorView` to a caller-owned
`IosView`, using the supplied frame and initial animation state. `IosActivityIndicator::start` and
`stop` map to UIKit's public animation methods; the wrapper sets `hidesWhenStopped(true)`, so stop
hides the indicator and start shows it unless an ancestor view is hidden. `is_animating` reports
UIKit's current state. Dropping the main-thread-only handle stops animation, while UIKit may retain
the now-stopped child view in its parent hierarchy. `native_activity_indicator` is a typed escape
hatch; direct UIKit changes can alter the wrapper's visibility semantics. Compile/link checks do not
prove runtime animation or visual behavior

```rust
let mut indicator = backend.add_activity_indicator(&mut parent, frame, false)?;
indicator.start()?;
let active = indicator.is_animating()?;
indicator.stop()?;
```

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
`Unsupported` for modes they do not implement

`TextFieldControl::set_submit_action` and `UiClient::set_submit_action` install or replace a
submit callback. iOS uses the public `UIControlEvents::EditingDidEndOnExit` event and passes a new
strict UTF-16 text snapshot, preserving nil and empty text; malformed UTF-16 reaches the callback
as `InvalidInput`. The callback runs synchronously on the main thread, is not `Send`, and uses the
same panic-containment and reentrancy policy as the edit callback. The field handle retains the
separate submit target and removes only its `textFieldSubmitted:` action on drop. Replacing an
action while it is active returns `Unavailable` and leaves it unchanged

Submit handling does not change the keyboard's Return key, resign first responder, or manage
application state. Number and decimal keyboards may not expose a Return key and therefore may not
emit this event. Host regressions and device / Simulator compile-link checks do not prove live
keyboard selection, secure rendering, Return-key presentation, or callback delivery

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

`IosUiBackend::add_system_symbol_image_view` creates a `UIImageView` from UIKit's public
`UIImage.systemImageNamed:` API, so Rust callers can use system symbol names without constructing a
`UIImage`. UIKit added this constructor in iOS 13.0; `ios-ui` retains its iOS 12.0 device floor and
returns `Unsupported` below the API floor. An empty name returns `InvalidInput`; a name UIKit does
not recognize returns `Unavailable`. Names are passed through without a framework-maintained
allowlist, so future public symbol names remain usable. Pass an optional native `UIColor` to set a
tint; `None` preserves UIKit's inherited tint. `IosImageView::set_tint_color` can update it later

```rust
let blue = UIColor::blueColor();
let checkmark = backend.add_system_symbol_image_view(
    &mut parent,
    frame,
    "checkmark.circle.fill",
    Some(&blue),
)?;
```

The B530 link/import probe checks the `UIImage.systemImageNamed:` and `UIView.setTintColor:`
selectors, UIKit linkage, and unchanged iOS 12.0 device / iOS 14.0 Simulator deployment targets.
It does not execute the probe or prove symbol lookup, tint appearance, or runtime rendering

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

## Native disclosure group

`IosUiBackend::add_disclosure_group` adds a `UIStackView` with a native 44-point `UIButton` header
and a caller-owned `UIView` content host. The supplied `Frame` is the expanded group size and must
be at least 44 points high; shorter frames return `InvalidInput`. The content host receives the
remaining height and can accept existing labels, fields, and controls through the standard backend
methods. `IosDisclosureGroup::set_expanded` hides or shows the content, updates the native frame,
and changes the button's visual chevron without invoking the user callback. A user tap commits the
new state before a synchronous callback. The handle owns its Rust target, callback, and native
objects; it is `!Send`/`!Sync`, and dropping it unregisters only its own target/action

The header exposes its title as the UIKit accessibility label and `Expanded` or `Collapsed` as the
accessibility value. This is a narrow state description on the native button, not a general
accessibility tree or accessibility framework. Add the disclosure stack as an arranged subview of a
caller-owned vertical `UIStackView` when following content should reflow on collapse; otherwise the
caller owns sibling layout. The device and Simulator link fixture checks public UIKit classes,
selectors, Objective-C linkage, and the iOS 12.0 / iOS 14.0 deployment floors. It does not execute
the probe or prove visual, VoiceOver, or runtime callback behavior

```rust
let mut disclosure = backend.add_disclosure_group(
    &mut parent,
    expanded_frame,
    "Raw Status",
    false,
    |expanded| { let _ = expanded; },
)?;
let _label = backend.add_label(
    disclosure.content_view_mut(),
    content_frame,
    raw_status,
)?;
disclosure.set_expanded(true)?;
```

## Scope and evidence

This crate is a partial native-control backend, not a view hierarchy runtime. It does not own an
application, window, controller, scene, navigation stack, modal presenter, or general layout engine.
It exposes narrow native `UITableView` and `UICollectionView` primitives, including `IosCollectionGrid`,
but does not provide general table/collection composition, a virtual view tree, or SwiftUI-style
declarative views. It does not implement a general accessibility model, animations, gestures,
multi-line editing, delegates, or keyboard management. The disclosure header's accessible expanded
state is the narrow exception described above. The existing `ios-minimal` example demonstrates a
separate app-entry vertical slice; this crate does not alter that example

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
