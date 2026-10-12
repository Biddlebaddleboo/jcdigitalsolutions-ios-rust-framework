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

B525 adds `IosDisclosureGroup` over native `UIStackView`, `UIButton`, and `UIView`. Its 44-point
header reports the title and `Expanded` / `Collapsed` through UIKit accessibility, owns a
main-thread-only synchronous callback, and toggles caller-owned content. Programmatic state updates
are silent; callbacks reuse the existing panic/reentrancy containment. Host policy tests cover
header-height rejection and idempotent state updates. Device / Simulator link checks are the only
native evidence; they do not prove VoiceOver, visual reflow, or live callback behavior

B527 adds a typed responder-chain keyboard action. It sends UIKit's public `resignFirstResponder`
action through `UIApplication` only after a valid main-thread proof; a host regression confirms a
missing proof does not send the action. Target link checks establish the selector only, not keyboard
dismissal at runtime

B528 adds `IosGridView`, an eager equal-column layout over native vertical and horizontal
`UIStackView` values. The caller supplies columns, spacing, frame, and child views; incomplete rows
use inert placeholders. It is a lower-level UIKit helper, not SwiftUI: no declarative tree, bindings,
state diffing, lazy creation, or cell reuse is provided. Host tests reject zero columns and invalid
spacing; native checks do not establish layout or accessibility behavior

B530 adds `IosUiBackend::add_system_symbol_image_view` over public UIKit `UIImage.systemImageNamed:`.
Unknown symbols map to `Unavailable`; empty names map to `InvalidInput`. Device / Simulator link
probes check imports, selector, and deployment floor, not system-symbol rendering at runtime

B534 adds `IosListView` over public `UITableView`, with owned title/detail row snapshots, unique
`u64` identities, explicit full-snapshot replacement, and stable-ID selection callbacks. UIKit
reuses subtitle cells; rejected duplicate-ID or unrepresentable-count updates preserve the prior
snapshot. The handle and retained delegate/data-source target remain main-thread-bound. Host tests
cover replacement policy, panic containment, and callback reentry. The registered profile includes
host regressions and Device/Simulator link probes; those do not establish live row rendering, selection,
scrolling, or accessibility behavior. This is a reusable native list primitive, not a declarative
SwiftUI `List` or state-observation system

B537 adds `IosContextMenuAction` and a fixed UIKit `IosContextMenu` over public
`UIContextMenuInteraction`, with unique stable IDs, owned nonempty titles, main-thread selection,
panic/reentry containment, and exact interaction removal on drop. It requires iOS 13 at runtime
while preserving the crate's iOS 12 compile floor. The registered profile covers host policy
regressions and Device/Simulator link probes; menu display and selection behavior remain
runtime-unverified. This does not add SwiftUI context-menu composition or clipboard policy

B538 adds `IosSelectableTextView` over public `UITextView`, with editing disabled, selection and
scrolling enabled, strict UTF-16 text snapshots, length-aware writes, and a typed native escape.
Host tests preserve embedded NUL/supplementary scalars and reject malformed surrogate data. Device
and Simulator compile/link checks establish UIKit selector availability at iOS 12.0 / 14.0, not
live selection or clipboard behavior. This is not SwiftUI text state binding; content and copy
policy remain caller-owned

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
  scene management, custom layout engine, general accessibility system, or virtual view tree.
  B525 adds only the UIKit disclosure header's label and expanded/collapsed accessibility value

## Documentation and tests

- Document frame validation and units, borrowed-string copy/lifetime semantics, control-handle
  ownership, callback thread/reentrancy/panic behavior, and native escape hatches
- Add portable tests for valid and invalid frame values and a fake static backend path. Do not
  simulate UIKit action delivery or claim live UI behavior
- Mark this as a minimal native-control slice. Text fields add no delegate, callback, keyboard, or
  runtime text-input contract. The crate does not claim navigation, scene lifecycle, general
  presentation, custom layout, a general accessibility system, or parity with a complete UI
  framework. B525's native accessibility state is a single-control exception

## Validation and handoff

- Do not run Cargo while G's portable no-std final-link proof is active; root will integrate the
  package and run workspace checks after this slice
- After integration, run portable no-default-features checks, iOS device/simulator checks, strict
  Clippy, formatting, docs-check, zero-Swift-source, and diff checks
- Until those gates run, report only source-level review and any direct non-Cargo checks. Do not
  claim Xcode compilation, UIKit linking, simulator launch, device behavior, callback execution, or
  visual/layout correctness

## B542 — editable multiline text

B542 adds the iOS-specific `IosTextEditor` over public `UITextView`, separate from the existing
read-only `IosSelectableTextView`. It owns strict UTF-16 snapshots, synchronous owned edit
callbacks, silent programmatic `set_text`, responder requests, and a retained Objective-C delegate
that detaches on handle drop. Reentrant callback calls are ignored, panic is contained, and active
callback replacement fails without removing the current callback. The API stays imperative and
UIKit-native; it does not add state binding, automatic observation, diffing, or declarative
composition. Host and registered device/Simulator static checks do not prove live text input,
keyboard behavior, focus, or callback delivery.

The integrated command `ios-rust-validate --workspace-root "$PWD" --spec
"$PWD/tools/validation/specs/validation-v1.json" --capability ios-ui --format json` reported 271
PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED of 273 at snapshot
`2b0a501c10b79e12943a818681f768ffac24b10bb3bb88567c94476585ef462b`. Skips are Simulator
execution and physical-device proof; no live UI behavior is claimed.

## B546 — stack layout controls

B546 extends the existing imperative `IosStackView` with typed UIKit alignment and distribution,
vertical-stack baseline rejection, and arranged-subview removal that ends stack layout management
while leaving the view in the stack hierarchy. The registered `ios-ui` profile reported 263 PASS,
0 FAIL, 0 BLOCKED,
and 2 SKIPPED of 265 at snapshot
`ad5d8b51bba3d0b8ffc204415570cac73a757c5dc5d193414d63b2500613a7ae`. Device and Simulator
compile, Clippy, Rustdoc, and link/import checks passed; Simulator execution and physical-device
proof remain skipped, so runtime layout and visual behavior are not claimed.

## B551 — safe-area stack pinning

B551 adds `IosStackView::pin_edges_to_safe_area` using UIKit's public `safeAreaLayoutGuide` and
existing typed anchors. It shares one-shot state with `pin_edges_to_parent`, validates exact direct
parent identity and autoresizing-mask state before activation, and preserves the iOS 12 device and
iOS 14 Simulator floors. Host regressions cover preflight rejection and one-shot policy; registered
device/Simulator link checks cover the public UIKit symbols. No runtime layout behavior is claimed.

## B554 — reusable sectioned collection grid

B554 adds `IosCollectionGrid` over UIKit `UICollectionView` compositional layouts and diffable full
snapshots, while leaving the eager `IosGridView` and portable `framework-ui` contract unchanged.
Sections support fixed-column, adaptive-column, and horizontal-carousel layouts; reusable native cells
render typed card, metric, action, and toggle content. Stable IDs remain lossless across the full
`u64` range, events report stable IDs and requested toggle values, and caller snapshots remain
authoritative. Invalid or overlapping snapshot replacement preserves current state. Host tests report
62 passed; device/Simulator compile and strict device Clippy pass, and the collection-grid link probe
verifies UIKit imports, diffable/compositional symbols, selectors, and deployment floors at iOS 12.0 /
14.0. Formatting and probe syntax checks pass. These are compile/link/static checks only; Simulator
execution, live cell rendering, scrolling, selection, animation, and physical-device behavior remain
unverified.

## B555 — typed collection section headers

B555 adds optional typed text headers to UIKit `IosCollectionGrid` without changing
`IosCollectionGridSection` or the existing no-header API. `IosCollectionGridSectionHeader`,
`IosUiBackend::add_collection_grid_with_headers`, and
`IosCollectionGrid::replace_snapshot_with_headers` validate duplicate/unknown section IDs and empty
titles before mutation. Stable-section title changes reload supplementary content without changing
item identity or event semantics. The implementation uses public reusable UIKit supplementary
views, preserves the iOS 13 runtime guard and iOS 12 device / iOS 14 Simulator link floors, and
registers header API, negative regression, UIKit selectors, and link checks in `ios-ui`. Host tests
pass 64/64; iOS device and Simulator target compiles, strict device Clippy, formatting, and the
two-target collection-grid link probe pass. The registered `ios-ui` validator passed 281 checks
with 0 FAIL, 0 BLOCKED, and 2 SKIPPED for Simulator execution and physical-device proof at commit
`503b35c5a213ee26d4dae2c3126b477484c75923`. `framework-ui`, `IosGridView`, arbitrary view
composition, bindings, and SwiftUI-style state observation remain unchanged. Simulator execution,
header rendering/accessibility, and physical-device behavior remain unverified.

## B562 — button accessibility metadata

B562 adds main-thread-only `IosButton::set_accessibility_label`,
`IosButton::set_accessibility_hint`, and `IosButton::set_accessibility_identifier` through public
UIKit accessibility properties. Each accepts `Option<&str>`: `None` writes native `nil`, `Some("")`
is an empty string, and valid UTF-8 including embedded NUL is passed to length-aware `NSString`
conversion. Host regressions cover nil/empty distinction and exact text preservation. The existing
`ios-ui` profile's list link probe compiles and links all three selectors on Device iOS 12.0 and
Simulator iOS 14.0; live VoiceOver announcements and identifier behavior remain unverified. This
closes the metadata API gap used by Rideshare Bot's `BinaryLogViewerView` row copy button but does
not add portable accessibility or SwiftUI state APIs.

## B566 — text-field accessibility metadata

B566 adds main-thread `IosTextField::set_accessibility_label`, `set_accessibility_hint`, and
`set_accessibility_identifier` through public UIKit metadata setters. Each accepts `Option<&str>`:
`None` clears the native value, `Some("")` preserves an explicitly empty string, and UTF-8,
including embedded NUL, uses the existing length-aware `NSString` conversion. Existing host
`AccessibilityText` regressions cover clearing, empty strings, Unicode, and embedded NUL; the
registered `ios-ui` text-field link fixture now compiles the setters and verifies their selectors at
iOS 12.0 device and iOS 14.0 Simulator floors. No portable accessibility tree is added, and live
VoiceOver or identifier behavior remains unverified.

## B584 — explicit native control enabled state

B584 adds `is_enabled` and `set_enabled` to `ButtonControl`, `ToggleControl`, and `TextFieldControl` and the existing UIKit button, switch, and text-field handles. UIKit setters are main-thread-only and do not emit action, value-change, edit, or submit callbacks. Its first isolated `ios-ui` profile reported 271 PASS, 1 source-guard FAIL, and 2 SKIPPED for Simulator execution and physical-device proof. The combined B584/B585 profile is pending; native interaction remains unverified.

## B585 — UIKit system text style and semantic text color

B585 adds one typed `IosSystemTextStyle` surface for labels, button titles, and text fields, backed
by UIKit's preferred system fonts and content-size-category adjustment. `IosSemanticTextColor`
provides primary, secondary, blue, red, green, and orange text colors; primary/secondary use dynamic
UIKit colors on iOS 13+ and black/gray fallbacks on the iOS 12 device floor. Host policy tests cover
both availability branches and ensure named system colors do not change. Full combined `ios-ui`
qualification and runtime behavior remain pending.
