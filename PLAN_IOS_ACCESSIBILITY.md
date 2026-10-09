# PLAN_IOS_ACCESSIBILITY.md — Workstream B8: UIKit Accessibility Metadata

## Objective

Add a capability-scoped iOS extension for accessibility metadata on caller-owned UIKit `UIView` instances, using public `objc2` bindings and preserving native UIKit behavior

This is a bounded metadata adapter, not a reusable view system or a claim of full accessibility coverage

## Dependencies

- Foundation A and the iOS `ios-runtime` main-thread contract are integrated
- Use the active Xcode SDK and existing workspace `objc2` dependencies; do not add a Swift source layer
- This workstream does not depend on Swift ABI or other capability backends

## Read first

- `PLAN.md`
- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES.md`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/UNSAFE.md`
- `docs/ios/runtime.md`
- Current `objc2-ui-kit` generated bindings for the selected public accessibility properties

## Write scope

- `platform/ios/ios-accessibility/**`
- `docs/ios/accessibility.md`
- `PLAN_IOS_NATIVE.md` only to link this B8 decomposition

Do not edit examples, other platform crates, shared capability manifests/indexes, workspace dependency versions, or CI. The orchestrator owns shared manifest and CI updates

## Required behavior

- Provide a small iOS-only crate for setting caller-selected accessibility element state, label, hint, value, and a bounded set of standard traits on a borrowed `UIView`
- Use the public `objc2-ui-kit` accessibility bindings; keep generated dependency types and conversions behind the crate except for the unavoidable borrowed UIKit view boundary
- Require the existing typed main-thread proof, remain `!Send`, and perform synchronous property updates on the caller's main thread
- Define whether each setter replaces or clears the prior value. Nil/absent text must clear the matching native property; setting traits must have explicit replacement semantics
- Use framework-owned trait names/bit flags where practical; map only traits whose constants and availability are verified from the active public SDK. Do not expose arbitrary raw pointers or guessed integer masks
- Record the exact SDK declarations, framework imports, deployment floor, and whether device/simulator results are compile/link-only
- Use no private selectors, private APIs, hand-written Objective-C runtime calls, Swift source, UIAccessibility callback blocks, or accessibility runtime clone
- Mark the capability partial. Do not claim VoiceOver UX, announcements, focus movement, custom actions, custom accessibility elements/containers, SwiftUI, or complete accessibility coverage

## Non-goals

- Creating, laying out, retaining, or rendering UIKit views
- Native view/controller lifecycle or general presentation
- Dynamic accessibility blocks, announcements, focus APIs, custom actions, or custom `UIAccessibilityElement` implementations
- Portable accessibility contracts or changes to `framework-ui`
- Live VoiceOver/screen-reader behavior claims

## Status

B8's implementation and guide meet the bounded metadata contract. The public API borrows only a
`UIView` and the typed `MainThread` proof, stores the main-thread marker, and is `!Send`/`!Sync`.
Each setter is synchronous; `is_element()` reads the current native property synchronously; text
`None` clears through `nil`, `Some("")` remains an empty string, and `set_traits` replaces the full
trait mask, using UIKit's `UIAccessibilityTraitNone` for an empty set. B8's seven initial
framework-owned traits map to public constants from the active SDK; B92
adds `Adjustable` as the eighth trait, B95 adds `NotEnabled` as the ninth, B98 adds `KeyboardKey`
as the tenth, B100 adds `UpdatesFrequently` as the eleventh, B102 adds `PlaysSound` as the
twelfth, B104 adds `CausesPageTurn` as the thirteenth, B106 adds `StartsMediaSession` as the
fourteenth, B108 adds `AllowsDirectInteraction` as the fifteenth, and B111 adds `SummaryElement`
as the sixteenth. Focused tests
cover optional text, trait combination, empty-set behavior, order independence, and deriving masks
only from the passed traits.

B92 adds `AccessibilityTrait::Adjustable` through `UIAccessibilityTraitAdjustable`. The active
iOS 26.5 SDK declares it with `API_AVAILABLE(ios(4.0), watchos(2.0))`; this lowers the selected
surface API floor from iOS 6.0 to iOS 4.0, while the host's binary floor remains iOS 12.0 for device
and iOS 14.0 for Simulator. Apple requires an adjustable element to implement
`accessibilityIncrement` and `accessibilityDecrement`. The B8 adapter sets the trait only, so the
caller must use it only for a view that supplies both methods. No callback or action implementation
is added. This B92 pass added no tests and ran no tests; it updated the existing test fixture map
only so its exhaustive match includes the new variant

The B92 source audit confirms `UIAccessibilityTraitAdjustable` in `objc2-ui-kit` 0.3.2's generated
`UIAccessibilityConstants` feature; the existing manifest already enables that feature. Apple's
[trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/adjustable?language=objc)
and [`UIAccessibilityAction`](https://developer.apple.com/documentation/objectivec/uiaccessibilityaction?language=objc)
both require the two methods. The active SDK has the trait declaration at
`UIAccessibilityConstants.h:90` and the method requirements at `UIAccessibility.h:349-355`

B95 adds `AccessibilityTrait::NotEnabled` through `UIAccessibilityTraitNotEnabled`. The active
iOS 26.5 SDK declares the constant at `UIAccessibilityConstants.h:72` with
`API_AVAILABLE(watchos(2.0))` only and no explicit iOS minimum annotation. The exact static is in
`objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature; no dependency or feature
change is needed. Apple defines the trait as an accessibility element that is not enabled and does
not respond to user interaction ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/notenabled?language=objc)).
The adapter changes accessibility metadata only. The caller must use this trait only when the
underlying view or control is already disabled or otherwise does not respond to user interaction;
setting the trait does not disable or block interaction. The selected API floor remains iOS 4.0
from `Adjustable`. This B95 pass added no
tests and ran no tests; it updated the existing test fixture map so its exhaustive match includes
the new variant

B98 adds `AccessibilityTrait::KeyboardKey` through `UIAccessibilityTraitKeyboardKey`. The active
iOS 26.5 SDK declares the constant at `UIAccessibilityConstants.h:58` with
`API_AVAILABLE(watchos(2.0))` only and no explicit iOS minimum annotation. The exact static is in
`objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature; no dependency or feature
change is needed. Apple defines the trait as an accessibility element that behaves like a keyboard
key ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/keyboardkey?language=objc)).
The caller must use it only for an element that already provides the key behavior; this adapter
sets metadata only and adds no key event handling or keyboard. The selected API floor remains iOS
4.0 from `Adjustable`. This B98 pass added no tests and ran no tests; it updated the existing test
fixture map so its exhaustive match includes the new variant

B100 adds `AccessibilityTrait::UpdatesFrequently` through `UIAccessibilityTraitUpdatesFrequently`.
The active iOS 26.5 SDK declares it at `UIAccessibilityConstants.h:78` with
`API_AVAILABLE(watchos(2.0))` only and no explicit iOS minimum annotation. The exact static is in
`objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature; no dependency or feature
change is needed. Apple says to use this trait when the element's label or value changes too
frequently to send an update notification for every change, so an assistive app may poll instead
([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/updatesfrequently?language=objc)).
This adapter sets the trait only. The caller owns label/value updates and notification policy; this
API does not guarantee that an assistive app polls or when. The selected API floor remains iOS 4.0
from `Adjustable`. This B100 pass added no tests and ran no tests; it updated the existing test
fixture map so its exhaustive match includes the new variant

B102 adds `AccessibilityTrait::PlaysSound` through `UIAccessibilityTraitPlaysSound`. The active
iOS 26.5 SDK declares it at `UIAccessibilityConstants.h:55` with
`API_AVAILABLE(watchos(2.0))` only and no explicit iOS minimum annotation. The exact static is in
`objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature; no dependency or feature
change is needed. Apple says this trait describes an element that plays its own sound when the
user activates it ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/playssound?language=objc)).
The caller must provide that activation sound; this adapter sets metadata only and adds no
activation handler or audio behavior. The selected API floor remains iOS 4.0 from `Adjustable`.
This B102 pass added no tests and ran no tests; it updated the existing test fixture map so its
exhaustive match includes the new variant

B104 adds `AccessibilityTrait::CausesPageTurn` through `UIAccessibilityTraitCausesPageTurn`. The
active iOS 26.5 SDK declares the trait at `UIAccessibilityConstants.h:100` with
`API_AVAILABLE(ios(5.0), watchos(2.0))`; the exact static is in `objc2-ui-kit` 0.3.2's generated
`UIAccessibilityConstants` feature. The existing `UIAccessibility` feature also exposes
`accessibilityScroll:` and `UIAccessibilityScrollDirectionNext`; no dependency or feature change
is needed. The SDK declares `accessibilityScroll:` at `UIAccessibility.h:386` for iOS 4.2 and
unavailable on watchOS, and the `Next` direction at iOS 5.0. Apple says this trait describes a page
in a sequence: VoiceOver calls `accessibilityScroll:` with `Next` after reading the page and stops
when new content is unchanged ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/causespageturn?language=objc)).
The caller must provide this page-turn behavior; this adapter sets metadata only. The separate
`UIAccessibilityReadingContent` protocol supports continuous reading and is not implemented by this
crate ([protocol documentation](https://developer.apple.com/documentation/uikit/uiaccessibilityreadingcontent?language=objc)).
The selected API floor remains iOS 4.0 from `Adjustable`. This B104 pass added no tests and ran no
tests; it updated the existing test fixture map so its exhaustive match includes the new variant

B106 adds `AccessibilityTrait::StartsMediaSession` through `UIAccessibilityTraitStartsMediaSession`.
The active iOS 26.5 SDK declares the constant at `UIAccessibilityConstants.h:84` with
`API_AVAILABLE(ios(4.0), watchos(2.0))`; the exact static is in `objc2-ui-kit` 0.3.2's generated
`UIAccessibilityConstants` feature. Apple describes it for an element whose activation starts a
media session that should not be interrupted by assistive-app audio, such as audio recording
([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/startsmediasession?language=objc)).
The caller must use it only when activation actually starts such a media session; this adapter sets
metadata only and adds no media or audio behavior. No assistive-audio behavior or timing is claimed.
The selected API floor remains iOS 4.0. This B106 pass added no tests and ran no tests; it updated
the existing test fixture map so its exhaustive match includes the new variant

B108 adds `AccessibilityTrait::AllowsDirectInteraction` through
`UIAccessibilityTraitAllowsDirectInteraction`. The active iOS 26.5 SDK declares it at
`UIAccessibilityConstants.h:93` with `API_AVAILABLE(ios(5.0), watchos(2.0))`; the exact static is in
`objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature. Apple defines it for an
element that represents an object users interact with directly, such as a piano keyboard
([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/allowsdirectinteraction?language=objc)).
The caller must provide the touch-interactive region and input behavior. This adapter sets metadata
only, routes no touch events, and configures no `UIAccessibility.DirectTouchOptions`. The selected
API floor remains iOS 4.0. This B108 pass added no tests and ran no tests; it updated the existing
test fixture map so its exhaustive match includes the new variant

B111 adds `AccessibilityTrait::SummaryElement` through `UIAccessibilityTraitSummaryElement`. The
active iOS 26.5 SDK declares the constant at `UIAccessibilityConstants.h:69` with only
`API_AVAILABLE(watchos(2.0))`, so it adds no explicit iOS minimum to the selected surface. The exact
static is in `objc2-ui-kit` 0.3.2's generated `UIAccessibilityConstants` feature. Apple defines it
for an element that provides a summary of current conditions, settings, or state when the app
starts ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/summaryelement?language=objc)). The caller must supply that summary; this setter does not control when UIKit or assistive technology reads or presents it. The selected API floor remains iOS 4.0. This B111 pass added no tests and ran no tests; it updated the existing test fixture map so its exhaustive match includes the new variant

B114 adds `AccessibilityMetadata::is_element()`, a synchronous read of the current
`isAccessibilityElement` property on the already-borrowed `UIView`. The iOS 26.5 SDK declares this
property in `UIAccessibility.h:52` without an explicit iOS minimum annotation; the exact
`NSObjectUIAccessibility::isAccessibilityElement(&self, MainThreadMarker) -> bool` binding is in
`objc2-ui-kit` 0.3.2's existing `UIAccessibility` feature. Apple documents the property as an
accessibility-element flag; this getter does not report current visibility, focus, or assistive-app
reachability. The API floor remains iOS 4.0. No dependency or feature change was needed, and this
pass added or ran no tests.

B114 also audited `UIAccessibilityTraitSupportsZoom`: the SDK marks the constant at
`UIAccessibilityConstants.h:114` as iOS 17.0+, and `UIAccessibility.h:358-367` requires
`accessibilityZoomInAtPoint:` and `accessibilityZoomOutAtPoint:` callbacks. The generated
`objc2-ui-kit` 0.3.2 binding exposes the static through the already-enabled
`UIAccessibilityConstants` feature, but the current safe trait setter has no per-trait availability
result and this crate implements no zoom callbacks. The trait is therefore not exposed; adding it
would need a bounded iOS-17 availability/error contract and caller callback responsibility rather
than an unconditional enum variant.

B117 adds `AccessibilityMetadata::has_label()`, `has_hint()`, and `has_value()` as synchronous
checks of the corresponding current UIKit property getters. The getters return optional retained
`NSString` values through the existing `NSObjectUIAccessibility` bindings in `objc2-ui-kit` 0.3.2's
`UIAccessibility` feature; these methods drop the returned objects after the nil check and expose no
text or raw Objective-C type. `UIAccessibility.h:64`, `:79`, and `:95` declare the nullable
properties without an explicit iOS minimum annotation. An empty string counts as non-nil, and
UIKit may supply a default label, hint, or value, so `true` does not prove the caller explicitly set
the property. The methods query the properties only and do not invoke dynamic accessibility
callback blocks. The API floor remains iOS 4.0; no dependency or feature change was needed. This
pass added and ran no tests.

B120 adds `AccessibilityMetadata::has_trait(AccessibilityTrait)`, a synchronous check against
UIKit's current `accessibilityTraits` mask. It reuses the same framework-owned constant mapping as
`set_traits`, returns only membership for the requested known trait, and exposes no raw mask or
unknown native flags. UIKit controls may contribute default traits, so a positive result does not
prove the caller explicitly set the trait or predict assistive-technology behavior. The SDK declares
the mask getter at `UIAccessibility.h:113`; the generated binding is
`NSObjectUIAccessibility::accessibilityTraits(&self, MainThreadMarker) -> UIAccessibilityTraits`
in the existing `objc2-ui-kit` 0.3.2 `UIAccessibilityConstants` feature. This adds no framework
symbol or API floor; per-trait availability caveats remain those already recorded for the enum. This
pass added and ran no tests.

B123 adds `accessibility_elements_hidden()` and `set_accessibility_elements_hidden(bool)` for
UIKit's `accessibilityElementsHidden` property. The iOS 26.5 SDK declares it at
`UIAccessibility.h:160` with `API_AVAILABLE(ios(5.0))`; `objc2-ui-kit` 0.3.2 exposes the getter and
setter through the existing `NSObjectUIAccessibility` binding and `UIAccessibility` feature. Each
method checks its exact selector with `NSObjectProtocol::respondsToSelector` before dispatch and
returns `AccessibilityApiUnavailable` if unsupported, so this addition does not raise the crate's
iOS 4.0 floor. The property marks contained accessible elements as hidden; it does not set the
receiver's own element flag, visual hidden state, or interaction state. No VoiceOver result is
claimed, and no runtime availability path was exercised. This pass added and ran no tests.

B126 adds `should_group_accessibility_children()` and
`set_should_group_accessibility_children(bool)` for UIKit's `shouldGroupAccessibilityChildren`
property. The iOS 26.5 SDK declares it at `UIAccessibility.h:176` with `API_AVAILABLE(ios(6.0))`;
`objc2-ui-kit` 0.3.2 exposes the getter and setter through the existing `NSObjectUIAccessibility`
binding and `UIAccessibility` feature. Both methods check their exact selectors with
`NSObjectProtocol::respondsToSelector` before dispatch and return `AccessibilityApiUnavailable`
when unsupported, preserving the crate's iOS 4.0 floor. Apple describes the property as a request
to group children regardless of screen position; callers should use it on a parent whose children
form a logical group. The API reads or replaces the property only and does not guarantee an
assistive application's grouping or traversal result. This pass added and ran no tests.

B129 adds `accessibility_view_is_modal()` and `set_accessibility_view_is_modal(bool)` for UIKit's
`accessibilityViewIsModal` property. The iOS 26.5 SDK declares it at `UIAccessibility.h:167` with
`API_AVAILABLE(ios(5.0))`; `objc2-ui-kit` 0.3.2 exposes the getter and setter through the existing
`NSObjectUIAccessibility` binding and `UIAccessibility` feature. Both methods check their exact
selectors with `NSObjectProtocol::respondsToSelector` before dispatch and return
`AccessibilityApiUnavailable` when unsupported, preserving the crate's iOS 4.0 floor. Apple says a
true property value makes elements outside the view ignored for accessibility; callers should set
it only on a view that represents currently modal accessible content. This API reads or replaces
the property only and does not present, dismiss, or establish app-level modality. This pass added
and ran no tests.

B132 adds `set_accessibility_language(Option<&str>)` and
`has_accessibility_language()` for UIKit's `accessibilityLanguage` property. The iOS 26.5 SDK declares
the nullable `NSString *` property at `UIAccessibility.h:154`, documents BCP 47 language tags at
`:148-152`, and gives it no explicit iOS minimum annotation; `objc2-ui-kit` 0.3.2 exposes the typed
getter and setter through the existing `NSObjectUIAccessibility` binding and `UIAccessibility`
feature. Both methods check their exact selectors with `NSObjectProtocol::respondsToSelector`
before dispatch and return `AccessibilityApiUnavailable` if absent, keeping the crate's iOS 4.0
floor without assuming unguarded availability. `None` sends `nil`; non-`None` UTF-8 is converted to
`NSString` and passed through without BCP 47 validation or normalization. The presence query
examines only the property getter, not the iOS 17 `accessibilityLanguageBlock` or any language an
assistive application uses. This pass added and ran no tests.

B135 adds `accessibility_responds_to_user_interaction()` and
`set_accessibility_responds_to_user_interaction(bool)` for UIKit's
`accessibilityRespondsToUserInteraction` property. The iOS 26.5 SDK declares it at
`UIAccessibility.h:193` with `API_AVAILABLE(ios(13.0), tvos(13.0))`; the header says the default is
derived from other accessibility properties. `objc2-ui-kit` 0.3.2 exposes the typed getter and
setter through the existing `NSObjectUIAccessibility` binding and `UIAccessibility` feature. Both
methods check their exact selectors with `NSObjectProtocol::respondsToSelector` before dispatch
and return `AccessibilityApiUnavailable` if absent, preserving the crate's iOS 4.0 floor. The
getter reports the scalar property only, not whether the caller explicitly set it. The setter
changes no native action or input behavior; callers must describe the real action behavior. Neither
method invokes the separate iOS 17 `accessibilityRespondsToUserInteractionBlock`. This pass added
and ran no tests.

B138 adds `accessibility_navigation_style()` and
`set_accessibility_navigation_style(AccessibilityNavigationStyle)` for UIKit's
`accessibilityNavigationStyle` property. The iOS 26.5 SDK declares the property at
`UIAccessibility.h:185` with `API_AVAILABLE(ios(8.0))`; `UIAccessibilityConstants.h:207-224` defines
the `Automatic`, `Separate`, and `Combined` enum values with an iOS 8.0 floor. `objc2-ui-kit` 0.3.2
provides the typed property methods and enum through the existing `UIAccessibility` and
`UIAccessibilityConstants` features. Both methods check their exact selectors with
`NSObjectProtocol::respondsToSelector` before dispatch and return `AccessibilityApiUnavailable`
if absent, preserving the crate's iOS 4.0 floor. The Rust enum maps only the three generated
constants; the getter returns `None` for an unknown native value and exposes no raw `NSInteger`.
The SDK says this property currently affects Switch Control only, not VoiceOver or other assistive
technologies. This adapter only reads or replaces the scalar property and claims no navigation
result. This pass added and ran no tests.

B141 adds `set_accessibility_textual_context(Option<AccessibilityTextualContext>)` and
`has_accessibility_textual_context()` for UIKit's `accessibilityTextualContext` property. The iOS
26.5 SDK declares the nullable property at `UIAccessibility.h:227` with `API_AVAILABLE(ios(13.0),
tvos(13.0))`; the header describes named contexts as input that may help assistive technologies
choose output and gives source-code punctuation as an example. `UIAccessibilityConstants.h:249-255`
and `objc2-ui-kit` 0.3.2 expose the seven public `NSString` constants and typed getter/setter through
the existing `UIAccessibility` and `UIAccessibilityConstants` features. Both methods check their
exact selectors with `NSObjectProtocol::respondsToSelector` before dispatch and return
`AccessibilityApiUnavailable` if absent, preserving the crate's iOS 4.0 floor. The Rust enum maps
only generated constants; `None` clears the property. The presence query reads only the scalar
property and neither method invokes the separate iOS 17 `accessibilityTextualContextBlock` or
claims an assistive application's output. This pass added and ran no tests.

B144 adds `accessibility_container_type()` and
`set_accessibility_container_type(AccessibilityContainerType)` for UIKit's
`UIAccessibilityContainer.accessibilityContainerType` property. The iOS 26.5 SDK declares the
property at `UIAccessibilityContainer.h:58` with `API_AVAILABLE(ios(11.0))`; the generated
`UIAccessibilityContainerType` enum is in `UIAccessibilityConstants.h:229-235`. `objc2-ui-kit`
0.3.2 provides the typed category methods and enum; the package enables its existing
`UIAccessibilityContainer` feature. Both methods selector-check before dispatch and return
`AccessibilityApiUnavailable` if absent, preserving the crate's iOS 4.0 floor. The Rust enum maps
only `None`, `List`, and `Landmark`; the getter returns `None` for unknown values and exposes no raw
`NSInteger`. `DataTable` is excluded because Apple requires `UIAccessibilityContainerDataTable`;
`SemanticGroup` is excluded because that enum member has a separate iOS 13.0 floor and this slice
does not add a version check. The setter marks the container type only and does not implement
container enumeration or create children. This pass added and ran no tests.

B147 audited UIKit's `accessibilityDirectTouchOptions` property and did not add it. The iOS 26.5
SDK declares the property at `UIAccessibility.h:230` and option flags at
`UIAccessibilityConstants.h:237-244`, with an iOS 17.0 floor; `objc2-ui-kit` 0.3.2 generates typed
property methods and flags under the existing `UIAccessibility` and `UIAccessibilityConstants`
features. Apple documents that the options change VoiceOver touch passthrough and audio:
`SilentOnTouch` silences VoiceOver in the direct-touch area and is appropriate only when the app
provides its own conflicting audio feedback, while `RequiresActivation` gates touch passthrough on
user activation ([Apple API reference](https://developer.apple.com/documentation/uikit/uiaccessibility/directtouchoptions)).
This metadata-only crate cannot establish that a real direct-touch area exists or provide or verify
its touch and audio behavior, and no runtime accessibility validation is in scope. Exposing this as
a generic setter would surface behavior with preconditions this adapter cannot represent. Revisit
only with a dedicated direct-touch contract and caller-owned interaction/audio responsibilities.
This pass added and ran no tests.

B150 adds `set_accessibility_user_input_labels(&[&str])` and
`has_accessibility_user_input_labels()` for UIKit's `accessibilityUserInputLabels` property. The iOS
26.5 SDK declares the `NSArray<NSString *>` property at `UIAccessibility.h:204` with
`API_AVAILABLE(ios(13.0), tvos(13.0))`; its header specifies primary-first ordering, descending
alternative importance, an empty-array default, and fallback to `accessibilityLabel` for empty or
invalid values. `objc2-ui-kit` 0.3.2 exposes the typed getter and an unsafe setter through the
existing `UIAccessibility` feature. The package enables the direct `NSArray` feature on
`objc2-foundation` for conversion. Both methods selector-check and return
`AccessibilityApiUnavailable` when absent. The Rust setter always constructs a non-null typed
`NSArray<NSString>` (including for an empty input slice) and passes `Some`, satisfying the generated
setter's `None` safety restriction; it does not expose raw Objective-C objects. The getter returns
only whether the native array is non-empty; UIKit controls may supply a default, and the iOS 17
`accessibilityUserInputLabelsBlock` is not invoked. No recognition or match guarantee is claimed.
This pass added and ran no tests.

B153 adds `accessibility_activation_point()` and
`set_accessibility_activation_point(CGPoint)` for UIKit's `accessibilityActivationPoint`
property. The active iOS 26.5 SDK declares it at `UIAccessibility.h:145` with an iOS 5.0 floor;
the header specifies screen coordinates and defaults to the midpoint of `accessibilityFrame`.
`objc2-ui-kit` 0.3.2 generates typed `CGPoint` getter/setter methods when its
`objc2-core-foundation` feature is enabled; the package adds the matching direct
`objc2-core-foundation` `CFCGTypes` feature. Both methods check their exact selectors and return
`AccessibilityApiUnavailable` when absent, preserving the crate's iOS 4.0 API floor. The getter
reports the native property only. The setter passes the caller's screen point unchanged, performs
no coordinate conversion, and requires callers to recompute after layout or screen-position
changes; no assistive-application activation result is claimed. Apple documents the property in
its [UIKit accessibility reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityactivationpoint?language=objc).
This pass added and ran no tests.

B156 adds `set_accessibility_attributed_label(Option<&NSAttributedString>)` and
`has_accessibility_attributed_label()` for UIKit's `accessibilityAttributedLabel` property. The
active iOS 26.5 SDK declares the nullable copied property at `UIAccessibility.h:70` with an iOS
11.0 floor. `objc2-ui-kit` 0.3.2 generates typed getter/setter methods on
`NSObjectUIAccessibility`; its `UIAccessibility` feature enables the generated API and the
package adds direct `objc2-foundation/NSAttributedString` feature access. Both methods check their
exact selectors and return `AccessibilityApiUnavailable` when absent, preserving the crate's iOS
4.0 floor. The setter passes the caller-owned attributed string through as a borrow, or `None` to
clear it; UIKit copies it and documents that setting it changes `accessibilityLabel` and vice versa.
The presence query reports only whether the property getter is non-nil, and neither method
invokes the iOS 17 `accessibilityAttributedLabelBlock`. This API does not validate attributed-string
keys or claim speech or assistive-application output. Apple documents the property and the broader
[UIKit text-attribute surface](https://developer.apple.com/documentation/uikit/text-attributes-for-attributed-strings).
This pass added and ran no tests.

On Xcode 26.6 build 17F113 with iPhoneOS and iPhoneSimulator SDK 26.5, these checks pass:

- `cargo test --locked -p ios-accessibility` (3 focused unit tests pass; 0 doc tests)
- `cargo check --locked -p ios-accessibility`
- `cargo check --locked -p ios-accessibility --target aarch64-apple-ios`
- `cargo check --locked -p ios-accessibility --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `cargo xtask zero-swift-source`
- `git diff --check`

Inspection of `UIAccessibility.h` confirms the selected nullable copied text properties and
synchronous accessibility setters; `UIAccessibilityConstants.h` declares `UIAccessibilityTraits`
as `uint64_t` and the seventeen referenced public constants: sixteen traits plus `UIAccessibilityTraitNone`.
`UIAccessibilityTraitAdjustable` and `UIAccessibilityTraitStartsMediaSession` have explicit iOS 4.0
floors, `UIAccessibilityTraitCausesPageTurn` and `UIAccessibilityTraitAllowsDirectInteraction` have
explicit iOS 5.0 floors, and `UIAccessibilityTraitHeader` has an explicit iOS 6.0 floor. The other
eleven traits plus `UIAccessibilityTraitNone` have no explicit iOS minimum annotation. The SDK's
`SupportedTargets.iphoneos.MinimumDeploymentTarget` and
`SupportedTargets.iphonesimulator.MinimumDeploymentTarget` values are 12.0.
`objc2-ui-kit` 0.3.2 uses the direct features `UIAccessibility`, `UIAccessibilityConstants`,
`UIAccessibilityContainer`, `UIResponder`, `UIView`, and `objc2-core-foundation`;
`objc2-foundation` uses `NSArray`, `NSAttributedString`, and `NSString`, and
`objc2-core-foundation` uses `CFCGTypes`.

A temporary external Rust `cdylib` consumer linked for device and simulator with
`IPHONEOS_DEPLOYMENT_TARGET=12.0`. `vtool -show-build` reports device minimum iOS 12.0 / SDK 26.5
and simulator minimum iOS 14.0 / SDK 26.5. `otool -L` reports UIKit, Foundation,
`/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` reports the original seven
trait constants, `UIAccessibilityTraitNone`, and `objc_msgSend`. This is
compile/link evidence only; the B92 `UIAccessibilityTraitAdjustable`, B95
`UIAccessibilityTraitNotEnabled`, B98 `UIAccessibilityTraitKeyboardKey`, B100
`UIAccessibilityTraitUpdatesFrequently`, B102 `UIAccessibilityTraitPlaysSound`, B104
`UIAccessibilityTraitCausesPageTurn`, B106 `UIAccessibilityTraitStartsMediaSession`, B108
`UIAccessibilityTraitAllowsDirectInteraction`, and B111 `UIAccessibilityTraitSummaryElement`
references have no new link evidence in their respective passes.
The B153 `CGPoint` accessors and B156 attributed-label accessors were not included in a link audit;
this evidence pass makes no linked-symbol claim for them.
The host remains below the Xcode 27.x planning baseline; no simulator launch, device run, live
VoiceOver session, announcement, focus movement, accessibility audit, or UX behavior is claimed.
No CI/G8 files were changed.

## Validation and handoff

- Add focused conversion/trait-mapping checks where behavior is framework-owned; do not simulate VoiceOver
- Run `cargo fmt --all -- --check`, the crate checks, device/simulator `cargo check`, Clippy with `-D warnings`, SDK/link inspection, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Report changed files, commit SHA, exact SDK/API availability, checks and evidence limits, deviations, and unresolved assumptions. Do not push
