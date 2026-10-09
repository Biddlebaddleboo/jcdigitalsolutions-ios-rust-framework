# iOS UIKit accessibility metadata

## Scope and status

`ios-accessibility` is a partial, iOS-only adapter for synchronous accessibility metadata on a borrowed, caller-owned `UIView`. It exposes a getter and setter for `isAccessibilityElement`, setters for `accessibilityLabel`, `accessibilityHint`, and `accessibilityValue`, property-presence queries for those text fields, a setter and presence query for `accessibilityAttributedLabel`, setters and property-presence queries for `accessibilityLanguage` and `accessibilityTextualContext`, a setter and non-empty query for `accessibilityUserInputLabels`, getters and setters for `accessibilityActivationPoint`, `accessibilityContainerType`, `accessibilityElementsHidden`, `accessibilityViewIsModal`, `accessibilityRespondsToUserInteraction`, and `shouldGroupAccessibilityChildren`, a getter and setter for navigation style, a membership query for known traits, and sixteen standard traits. UIKit remains the accessibility implementation; this crate does not create or retain views, build a view tree, or replace UIKit behavior when its setters are unused.

This is not a claim of full accessibility support. No live VoiceOver or screen-reader behavior was exercised. Announcements, focus movement, custom actions, custom `UIAccessibilityElement` objects, accessibility containers, dynamic accessibility blocks, SwiftUI, and a complete accessibility API are out of scope.

## Use

```rust,ignore
use ios_accessibility::{AccessibilityMetadata, AccessibilityTrait};
use ios_runtime::main_thread::MainThread;
use objc2_ui_kit::UIView;

fn label_close_button(view: &UIView) {
    let main_thread = MainThread::current().expect("UIKit access requires the main thread");
    let metadata = AccessibilityMetadata::new(view, main_thread);
    metadata.set_element(true);
    metadata.set_label(Some("Close"));
    metadata.set_hint(None);
    metadata.set_value(Some(""));
    metadata.set_traits(&[AccessibilityTrait::Button]);
}
```

`UIView` is the only generated UIKit type exposed at this boundary. `AccessibilityMetadata` holds a borrow, does not retain the view, and requires the integrated `ios_runtime::main_thread::MainThread` proof. Its stored objc2 marker and explicit non-send marker make it `!Send` and `!Sync`. Construct, call, and drop the adapter on UIKit's main thread. Each setter performs its UIKit property update synchronously on the calling thread; this API does not dispatch work.

## Property semantics

- `is_element()` returns UIKit's current `isAccessibilityElement` property value. It does not report whether a view is on screen, focusable, or currently reachable through an assistive application.
- `set_element(bool)` replaces `isAccessibilityElement`; `false` disables the element flag.
- `accessibility_elements_hidden()` reads and `set_accessibility_elements_hidden(bool)` replaces UIKit's iOS 5.0 `accessibilityElementsHidden` property for accessible descendants contained by the view ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityelementshidden?language=objc)). Both check the needed selector first and return `Err(AccessibilityApiUnavailable)` when the view does not respond. This property does not set the view's own `isAccessibilityElement`, visual `hidden` state, or interaction state; it does not prove how an assistive application presents descendants.
- `accessibility_view_is_modal()` reads and `set_accessibility_view_is_modal(bool)` replaces UIKit's iOS 5.0 `accessibilityViewIsModal` property ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityviewismodal)). Both check the needed selector first and return `Err(AccessibilityApiUnavailable)` when the view does not respond. Set it only for a view that represents currently modal accessible content; this property does not present or dismiss a view or establish app-level modality.
- `accessibility_responds_to_user_interaction()` reads and `set_accessibility_responds_to_user_interaction(bool)` replaces UIKit's iOS 13.0 `accessibilityRespondsToUserInteraction` property ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityrespondstouserinteraction)). Both check the needed selector first and return `Err(AccessibilityApiUnavailable)` when the view does not respond. UIKit may derive the getter default from other accessibility properties; the caller must set `true` only when the accessible element actually performs an action in response to interaction. This metadata adds no action handler or interaction behavior, and the getter does not invoke the iOS 17 `accessibilityRespondsToUserInteractionBlock`.
- `accessibility_navigation_style()` reads and `set_accessibility_navigation_style(AccessibilityNavigationStyle)` replaces UIKit's iOS 8.0 `accessibilityNavigationStyle` property ([Apple property reference](https://developer.apple.com/documentation/uikit/uiaccessibilitynavigationstyle?language=objc)). The getter returns `None` for native values outside the three named styles. Both methods check their needed selector and return `Err(AccessibilityApiUnavailable)` when the view does not respond. UIKit's SDK header says this property currently affects Switch Control only, not VoiceOver or other assistive technologies; the API makes no navigation-result claim.
- `should_group_accessibility_children()` reads and `set_should_group_accessibility_children(bool)` replaces UIKit's iOS 6.0 `shouldGroupAccessibilityChildren` property ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/shouldgroupaccessibilitychildren?language=objc)). Both check the needed selector first and return `Err(AccessibilityApiUnavailable)` when the view does not respond. Set it only on a parent whose children form a logical group; the property does not guarantee a particular assistive-application grouping or traversal result.
- `set_label`, `set_hint`, and `set_value` replace the matching nullable native property. `None` passes `nil` and clears that property. `Some("")` sets an empty string; it is not treated as absent.
- `set_accessibility_attributed_label(Option<&NSAttributedString>)` replaces UIKit's iOS 11.0 `accessibilityAttributedLabel`; `None` clears it, and UIKit copies the supplied string. Apple says setting it also changes `accessibilityLabel` and vice versa. `has_accessibility_attributed_label()` reports whether the property getter is non-nil; neither method invokes the iOS 17 `accessibilityAttributedLabelBlock`. They check their needed selector and return `Err(AccessibilityApiUnavailable)` when absent. This adapter does not validate attributed-string keys or promise speech attributes or assistive-application output ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityattributedlabel?language=objc), [UIKit text-attribute reference](https://developer.apple.com/documentation/uikit/text-attributes-for-attributed-strings)).
- `set_accessibility_language(Option<&str>)` replaces the nullable `accessibilityLanguage` property with the caller's language tag; `None` clears it. Apple specifies BCP 47 language tags. The string is passed through without validation or normalization, so the caller owns tag correctness. The SDK header has no explicit iOS minimum annotation for this property; the selector is checked first and a missing selector returns `Err(AccessibilityApiUnavailable)`. `has_accessibility_language()` reports whether the property getter is non-nil, but does not invoke `accessibilityLanguageBlock` or report the language used by an assistive application ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilitylanguage)).
- `set_accessibility_textual_context(Option<AccessibilityTextualContext>)` replaces the iOS 13.0 `accessibilityTextualContext` property with one of this crate's named UIKit constants; `None` clears it. `has_accessibility_textual_context()` reports only whether the scalar property getter is non-nil, and neither method invokes the iOS 17 `accessibilityTextualContextBlock` or claims an assistive application's output. Both check their needed selector and return `Err(AccessibilityApiUnavailable)` when absent ([Apple property reference](https://developer.apple.com/documentation/uikit/uiaccessibilitytextualcontext?language=objc)).
- `set_accessibility_user_input_labels(&[&str])` replaces the iOS 13.0 `accessibilityUserInputLabels` array with caller-supplied labels; the primary label belongs first, followed by alternatives in descending importance. An empty slice assigns an empty array. Strings pass through without validation or normalization. `has_accessibility_user_input_labels()` reports whether the property array is non-empty; UIKit controls may supply a default, so this does not prove that the caller assigned labels. Both check the needed selector and return `Err(AccessibilityApiUnavailable)` when absent; the query does not invoke the iOS 17 callback block ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityuserinputlabels)). Apple says an empty or invalid array falls back to `accessibilityLabel`; this API does not validate input, recognize speech, or guarantee matching ([UIAccessibility reference](https://developer.apple.com/documentation/uikit/uiaccessibility-protocol?language=objc)).
- `accessibility_activation_point()` reads and `set_accessibility_activation_point(CGPoint)` replaces the iOS 5.0 `accessibilityActivationPoint` property using `objc2_core_foundation::CGPoint`. Points are in screen coordinates; UIKit defaults to the midpoint of `accessibilityFrame`. The setter passes the point through unchanged and does not convert coordinates; callers must recompute after layout or screen-position changes. Both check the needed selector and return `Err(AccessibilityApiUnavailable)` when absent. They report or set the property only and do not guarantee an assistive application's activation behavior ([Apple property reference](https://developer.apple.com/documentation/objectivec/nsobject-swift.class/accessibilityactivationpoint?language=objc)).
- `accessibility_container_type()` reads and `set_accessibility_container_type(AccessibilityContainerType)` replaces the iOS 11.0 `accessibilityContainerType` property ([Apple property reference](https://developer.apple.com/documentation/uikit/uiaccessibilitycontainertype?language=objc)). Both check the needed selector and return `Err(AccessibilityApiUnavailable)` when absent. The getter returns `None` for native values outside the supported subset. This property changes container metadata only; it does not create or enumerate accessible children.
- `has_label()`, `has_hint()`, and `has_value()` report whether the matching UIKit property getter returns a non-nil string; an empty string counts as present. UIKit may provide a default label, hint, or value, so these methods do not prove that the caller explicitly set a property. They read the native properties only and do not invoke dynamic accessibility callback blocks.
- `set_traits(&[AccessibilityTrait])` replaces the complete native trait mask. It does not read or merge the current mask, and can replace defaults that UIKit supplies for standard controls. Callers must pass the complete desired trait set. An empty slice sets UIKit's `UIAccessibilityTraitNone` constant.
- `has_trait(AccessibilityTrait)` checks whether the current native trait mask contains that framework-owned flag. UIKit defaults are included; unknown flags are not exposed, and this query does not report assistive-technology behavior.
- UIKit declares the three text properties as nullable, copied `NSString` properties. Each `Some(&str)` is converted to a temporary retained `NSString`; UIKit copies the assigned value. No custom normalization or localization is performed.

Except for the selector-guarded optional properties, the selected UIKit setters are synchronous, void property updates and return no error. The `accessibilityLanguage`, `accessibilityUserInputLabels`, `accessibilityActivationPoint`, `accessibilityAttributedLabel`, `accessibilityTextualContext`, `accessibilityContainerType`, `accessibilityElementsHidden`, `accessibilityViewIsModal`, `accessibilityRespondsToUserInteraction`, `accessibilityNavigationStyle`, and `shouldGroupAccessibilityChildren` methods return `AccessibilityApiUnavailable` if their needed selector is absent. This adapter does not assert that a view is currently in a window or exposed to assistive technologies.

## Traits

The framework-owned `AccessibilityTrait` enum maps only to UIKit constants declared by the active public SDK:

| Rust trait | UIKit constant |
|---|---|
| `Button` | `UIAccessibilityTraitButton` |
| `Link` | `UIAccessibilityTraitLink` |
| `Header` | `UIAccessibilityTraitHeader` |
| `Image` | `UIAccessibilityTraitImage` |
| `Selected` | `UIAccessibilityTraitSelected` |
| `StaticText` | `UIAccessibilityTraitStaticText` |
| `SearchField` | `UIAccessibilityTraitSearchField` |
| `Adjustable` | `UIAccessibilityTraitAdjustable` |
| `NotEnabled` | `UIAccessibilityTraitNotEnabled` |
| `KeyboardKey` | `UIAccessibilityTraitKeyboardKey` |
| `UpdatesFrequently` | `UIAccessibilityTraitUpdatesFrequently` |
| `PlaysSound` | `UIAccessibilityTraitPlaysSound` |
| `CausesPageTurn` | `UIAccessibilityTraitCausesPageTurn` |
| `StartsMediaSession` | `UIAccessibilityTraitStartsMediaSession` |
| `AllowsDirectInteraction` | `UIAccessibilityTraitAllowsDirectInteraction` |
| `SummaryElement` | `UIAccessibilityTraitSummaryElement` |

`Adjustable` tells assistive technology that the element supports value changes across a range. Apple requires an element with this trait to implement `accessibilityIncrement` and `accessibilityDecrement` through UIKit's `UIAccessibilityAction` informal protocol ([trait reference](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/adjustable?language=objc), [action reference](https://developer.apple.com/documentation/objectivec/uiaccessibilityaction?language=objc)). The caller must ensure the borrowed view provides both methods; this crate sets the trait only and adds no action callback or implementation.

`NotEnabled` tells assistive technology that the element is not enabled and does not respond to user interaction ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/notenabled?language=objc)). This crate only sets the accessibility trait; the caller must use it only when the underlying view or control is already disabled or otherwise does not respond to user interaction. The trait does not disable a control or change its interaction behavior.

`KeyboardKey` tells assistive technology that the accessibility element behaves like a keyboard key ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/keyboardkey?language=objc)). The caller must use it only for an element that provides the corresponding key behavior; this crate only sets metadata and adds no key event handling or keyboard.

`UpdatesFrequently` marks an element whose label or value changes too frequently to send an update notification for every change. Apple says assistive apps may poll for current information instead ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/updatesfrequently?language=objc)). This crate only sets the trait: the caller owns label/value updates and notification policy, and this API does not guarantee that an assistive app polls or when.

`PlaysSound` tells assistive technology that the element plays its own sound when the user activates it ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/playssound?language=objc)). The caller must provide that activation sound; this crate only sets the trait and adds no activation handler, audio playback, or sound effect.

`CausesPageTurn` marks an accessibility element that represents a page in a set of pages. Apple documents that VoiceOver calls `accessibilityScroll:` with `UIAccessibilityScrollDirectionNext` after reading the current page, and stops when the new content does not differ ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/causespageturn?language=objc)). The caller must provide this page behavior; this crate only sets the trait and does not implement `accessibilityScroll:`. The separate `UIAccessibilityReadingContent` protocol supports continuous reading and is not implemented here ([protocol documentation](https://developer.apple.com/documentation/uikit/uiaccessibilityreadingcontent?language=objc)).

`StartsMediaSession` describes an element whose activation starts a media session that should not be interrupted by assistive-app audio ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/startsmediasession?language=objc)). The caller must use it only when activation actually starts such a media session; this crate only sets the trait and adds no media or audio behavior. This guide makes no claim that assistive audio is silenced in a particular way or at a particular time.

`AllowsDirectInteraction` describes an element for direct touch interaction by VoiceOver users, such as a piano keyboard view ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/allowsdirectinteraction?language=objc)). The caller must provide the touch-interactive region and its input behavior; this crate only sets the trait, routes no touch events, and does not configure `UIAccessibility.DirectTouchOptions`.

`SummaryElement` marks an element that provides a summary of current conditions, settings, or state when the app starts ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/summaryelement?language=objc)). The caller must supply a real app-start summary; this crate only sets the trait and does not control when UIKit or assistive technology reads or presents it.

`UIAccessibilityTraitSupportsZoom` is not mapped. The active SDK marks it as iOS 17.0+, and Apple requires an element with this trait to implement both zoom callbacks ([trait documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytraits/supportszoom)). The current trait setter has no per-trait availability result and this crate does not implement the callbacks, so it does not expose that trait.

## Navigation style

`AccessibilityNavigationStyle` maps the three named `UIAccessibilityNavigationStyle` values:

| Rust value | UIKit constant |
|---|---|
| `Automatic` | `UIAccessibilityNavigationStyleAutomatic` |
| `Separate` | `UIAccessibilityNavigationStyleSeparate` |
| `Combined` | `UIAccessibilityNavigationStyleCombined` |

The iOS 8.0 `accessibilityNavigationStyle` property defaults to `Automatic`. The active SDK explicitly says this property currently affects Switch Control only, not VoiceOver or other assistive technologies. The getter maps only the three known values and returns `None` for any other native value; it exposes no raw `NSInteger`. The setter replaces the property with a framework-owned value and does not claim a particular navigation result.

## Textual context

`AccessibilityTextualContext` maps only to UIKit's generated public textual-context constants. Apple describes a textual context as a named classification that may help assistive technology choose how to output text, such as speaking punctuation for source code ([Apple documentation](https://developer.apple.com/documentation/uikit/uiaccessibilitytextualcontext?language=objc)). The iOS 13.0 property is nullable; `None` clears it. This crate exposes no arbitrary context string and does not promise speech or another assistive-technology result.

| Rust value | UIKit constant |
|---|---|
| `WordProcessing` | `UIAccessibilityTextualContextWordProcessing` |
| `Narrative` | `UIAccessibilityTextualContextNarrative` |
| `Messaging` | `UIAccessibilityTextualContextMessaging` |
| `Spreadsheet` | `UIAccessibilityTextualContextSpreadsheet` |
| `FileSystem` | `UIAccessibilityTextualContextFileSystem` |
| `SourceCode` | `UIAccessibilityTextualContextSourceCode` |
| `Console` | `UIAccessibilityTextualContextConsole` |

## Container type

`AccessibilityContainerType` maps a bounded subset of UIKit's public container types:

| Rust value | UIKit constant |
|---|---|
| `Unspecified` | `UIAccessibilityContainerTypeNone` |
| `List` | `UIAccessibilityContainerTypeList` |
| `Landmark` | `UIAccessibilityContainerTypeLandmark` |

The property is declared on UIKit's `UIAccessibilityContainer` interface. Use it only when the borrowed view represents the corresponding data-based container; this crate does not implement the container protocol or supply child elements. `DataTable` is omitted because Apple requires the `UIAccessibilityContainerDataTable` protocol. `SemanticGroup` is also omitted: that enum value has a separate iOS 13.0 availability, while this adapter guards only the iOS 11.0 property selector. Unknown native values are returned as `None` and no raw integer is exposed.

The Rust API does not accept a raw trait integer or pointer. The adapter ORs the selected UIKit constants; it does not hard-code their numeric values. Existing trait-mapping checks verify empty-set handling, combination, order independence, and that each update is derived only from the provided set. They do not model or simulate assistive technology.

## Availability and framework imports

The inspected host is macOS 26.6.2, Xcode 26.6 (build 17F113), with iPhoneOS and iPhoneSimulator SDK 26.5. This is below the repository's Xcode 27.x planning baseline. Both SDK `SDKSettings.plist` files set `MinimumDeploymentTarget` to iOS 12.0, so this host cannot verify a binary targeted below iOS 12.0. The exact inspected declarations are in:

- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibility.h`: category `NSObject (UIAccessibility)` declares `BOOL isAccessibilityElement`, nullable copied `NSString *accessibilityLabel`, nullable copied `NSString *accessibilityHint`, nullable copied `NSString *accessibilityValue`, and `UIAccessibilityTraits accessibilityTraits`. `accessibilityAttributedLabel` is a nullable copied `NSAttributedString *` with an iOS 11.0 floor and changes `accessibilityLabel` and vice versa. `accessibilityActivationPoint` is a `CGPoint` in screen coordinates with an iOS 5.0 floor and defaults to the midpoint of `accessibilityFrame`. These declarations are marked unavailable on watchOS; the first five have no iOS `API_AVAILABLE` version annotation in this header.
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibilityConstants.h`: `UIAccessibilityTraits` is `uint64_t`; the selected constants are declared there. `UIAccessibilityTraitAdjustable` and `UIAccessibilityTraitStartsMediaSession` have `API_AVAILABLE(ios(4.0), watchos(2.0))`; `UIAccessibilityTraitCausesPageTurn` and `UIAccessibilityTraitAllowsDirectInteraction` have `API_AVAILABLE(ios(5.0), watchos(2.0))`; `UIAccessibilityTraitHeader` has `API_AVAILABLE(ios(6.0), watchos(2.0))`. The other eleven exposed traits and `UIAccessibilityTraitNone` have no explicit iOS minimum annotation in this header. `UIAccessibilityTraitStartsMediaSession` is at line 84, `UIAccessibilityTraitAllowsDirectInteraction` at line 93, `UIAccessibilityTraitSummaryElement` at line 69, and `UIAccessibilityTraitCausesPageTurn` at line 100; its selector `accessibilityScroll:` is declared at `UIAccessibility.h:386` with iOS 4.2 availability and watchOS unavailability, while `UIAccessibilityScrollDirectionNext` is available from iOS 5.0.
- The same header says an adjustable element must implement `accessibilityIncrement` and `accessibilityDecrement`; Apple documents the same requirement for `UIAccessibilityTraitAdjustable` and `UIAccessibilityAction`.
- The same declarations are enabled by `objc2-ui-kit` 0.3.2 features `UIAccessibility`, `UIAccessibilityConstants`, `UIAccessibilityContainer`, `UIResponder`, `UIView`, and `objc2-core-foundation`; `objc2-foundation` 0.3.2 uses `NSArray`, `NSAttributedString`, and `NSString`, and `objc2-core-foundation` uses `CFCGTypes` for `CGPoint`.

The selected surface's lowest explicit iOS availability remains iOS 4.0, set by `UIAccessibilityTraitAdjustable` and `UIAccessibilityTraitStartsMediaSession`; `UIAccessibilityTraitCausesPageTurn` and `UIAccessibilityTraitAllowsDirectInteraction` are iOS 5.0, and `UIAccessibilityTraitHeader` is iOS 6.0. The other eleven traits and `UIAccessibilityTraitNone` have no explicit iOS minimum annotation. This API floor comes from active SDK declarations, not the binary floor for this host and not a crate deployment target: the crate does not set one. A prior temporary external `cdylib` consumer was linked with `IPHONEOS_DEPLOYMENT_TARGET=12.0` for both targets. `vtool -show-build` reported device `LC_BUILD_VERSION` `minos 12.0` / SDK 26.5 and simulator `minos 14.0` / SDK 26.5; the simulator Rust target did not lower its link minimum to 12.0. `otool -L` on both outputs listed UIKit, Foundation, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` showed the original seven selected trait constants, `UIAccessibilityTraitNone`, and `objc_msgSend`. That prior link audit did not include the B92 `UIAccessibilityTraitAdjustable`, B95 `UIAccessibilityTraitNotEnabled`, B98 `UIAccessibilityTraitKeyboardKey`, B100 `UIAccessibilityTraitUpdatesFrequently`, B102 `UIAccessibilityTraitPlaysSound`, B104 `UIAccessibilityTraitCausesPageTurn`, B106 `UIAccessibilityTraitStartsMediaSession`, B108 `UIAccessibilityTraitAllowsDirectInteraction`, or B111 `UIAccessibilityTraitSummaryElement` references; this follow-up makes no linked-symbol claim for them. The evidence proves compile/link imports only, not runtime or VoiceOver behavior. UIKit and Foundation are the required Apple framework imports; `objc2-ui-kit` supplies the UIKit framework link declaration.

No permission prompt, usage-description key, or entitlement is part of these property setters. The implementation uses public UIKit properties and generated objc2 bindings only; it adds no Swift source, private API, hand-written Objective-C runtime call, Blocks, or Swift ABI dependency.

## Validation and limits

Focused host tests cover optional text conversion and framework-owned trait combination. Device and simulator `cargo check`/Clippy validate compilation against the installed targets and SDK, not a live application. No simulator launch, physical-device run, VoiceOver session, accessibility audit, user study, or App Store review result is claimed.
