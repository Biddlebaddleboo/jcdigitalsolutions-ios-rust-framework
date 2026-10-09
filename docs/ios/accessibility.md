# iOS UIKit accessibility metadata

## Scope and status

`ios-accessibility` is a partial, iOS-only adapter for synchronous accessibility metadata on a borrowed, caller-owned `UIView`. It exposes setters for `isAccessibilityElement`, `accessibilityLabel`, `accessibilityHint`, `accessibilityValue`, and fifteen standard traits. UIKit remains the accessibility implementation; this crate does not create or retain views, build a view tree, or replace UIKit behavior when its setters are unused.

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

## Setter semantics

- `set_element(bool)` replaces `isAccessibilityElement`; `false` disables the element flag.
- `set_label`, `set_hint`, and `set_value` replace the matching nullable native property. `None` passes `nil` and clears that property. `Some("")` sets an empty string; it is not treated as absent.
- `set_traits(&[AccessibilityTrait])` replaces the complete native trait mask. It does not read or merge the current mask, and can replace defaults that UIKit supplies for standard controls. Callers must pass the complete desired trait set. An empty slice sets UIKit's `UIAccessibilityTraitNone` constant.
- UIKit declares the three text properties as nullable, copied `NSString` properties. Each `Some(&str)` is converted to a temporary retained `NSString`; UIKit copies the assigned value. No custom normalization or localization is performed.

No setter returns an error: the selected UIKit setters are synchronous, void property updates. This adapter does not assert that a view is currently in a window or exposed to assistive technologies.

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

The Rust API does not accept a raw trait integer or pointer. The adapter ORs the selected UIKit constants; it does not hard-code their numeric values. Existing trait-mapping checks verify empty-set handling, combination, order independence, and that each update is derived only from the provided set. They do not model or simulate assistive technology.

## Availability and framework imports

The inspected host is macOS 26.6.2, Xcode 26.6 (build 17F113), with iPhoneOS and iPhoneSimulator SDK 26.5. This is below the repository's Xcode 27.x planning baseline. Both SDK `SDKSettings.plist` files set `MinimumDeploymentTarget` to iOS 12.0, so this host cannot verify a binary targeted below iOS 12.0. The exact inspected declarations are in:

- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibility.h`: category `NSObject (UIAccessibility)` declares `BOOL isAccessibilityElement`, nullable copied `NSString *accessibilityLabel`, nullable copied `NSString *accessibilityHint`, nullable copied `NSString *accessibilityValue`, and `UIAccessibilityTraits accessibilityTraits`. These declarations are marked unavailable on watchOS and have no iOS `API_AVAILABLE` version annotation in this header.
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibilityConstants.h`: `UIAccessibilityTraits` is `uint64_t`; the selected constants are declared there. `UIAccessibilityTraitAdjustable` and `UIAccessibilityTraitStartsMediaSession` have `API_AVAILABLE(ios(4.0), watchos(2.0))`; `UIAccessibilityTraitCausesPageTurn` and `UIAccessibilityTraitAllowsDirectInteraction` have `API_AVAILABLE(ios(5.0), watchos(2.0))`; `UIAccessibilityTraitHeader` has `API_AVAILABLE(ios(6.0), watchos(2.0))`. The other eleven exposed traits and `UIAccessibilityTraitNone` have no explicit iOS minimum annotation in this header. `UIAccessibilityTraitStartsMediaSession` is at line 84, `UIAccessibilityTraitAllowsDirectInteraction` at line 93, `UIAccessibilityTraitSummaryElement` at line 69, and `UIAccessibilityTraitCausesPageTurn` at line 100; its selector `accessibilityScroll:` is declared at `UIAccessibility.h:386` with iOS 4.2 availability and watchOS unavailability, while `UIAccessibilityScrollDirectionNext` is available from iOS 5.0.
- The same header says an adjustable element must implement `accessibilityIncrement` and `accessibilityDecrement`; Apple documents the same requirement for `UIAccessibilityTraitAdjustable` and `UIAccessibilityAction`.
- The same declarations are enabled by `objc2-ui-kit` 0.3.2 features `UIAccessibility`, `UIAccessibilityConstants`, `UIResponder`, and `UIView`; text conversion uses `objc2-foundation` 0.3.2 `NSString`.

The selected surface's lowest explicit iOS availability remains iOS 4.0, set by `UIAccessibilityTraitAdjustable` and `UIAccessibilityTraitStartsMediaSession`; `UIAccessibilityTraitCausesPageTurn` and `UIAccessibilityTraitAllowsDirectInteraction` are iOS 5.0, and `UIAccessibilityTraitHeader` is iOS 6.0. The other eleven traits and `UIAccessibilityTraitNone` have no explicit iOS minimum annotation. This API floor comes from active SDK declarations, not the binary floor for this host and not a crate deployment target: the crate does not set one. A prior temporary external `cdylib` consumer was linked with `IPHONEOS_DEPLOYMENT_TARGET=12.0` for both targets. `vtool -show-build` reported device `LC_BUILD_VERSION` `minos 12.0` / SDK 26.5 and simulator `minos 14.0` / SDK 26.5; the simulator Rust target did not lower its link minimum to 12.0. `otool -L` on both outputs listed UIKit, Foundation, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` showed the original seven selected trait constants, `UIAccessibilityTraitNone`, and `objc_msgSend`. That prior link audit did not include the B92 `UIAccessibilityTraitAdjustable`, B95 `UIAccessibilityTraitNotEnabled`, B98 `UIAccessibilityTraitKeyboardKey`, B100 `UIAccessibilityTraitUpdatesFrequently`, B102 `UIAccessibilityTraitPlaysSound`, B104 `UIAccessibilityTraitCausesPageTurn`, B106 `UIAccessibilityTraitStartsMediaSession`, B108 `UIAccessibilityTraitAllowsDirectInteraction`, or B111 `UIAccessibilityTraitSummaryElement` references; this follow-up makes no linked-symbol claim for them. The evidence proves compile/link imports only, not runtime or VoiceOver behavior. UIKit and Foundation are the required Apple framework imports; `objc2-ui-kit` supplies the UIKit framework link declaration.

No permission prompt, usage-description key, or entitlement is part of these property setters. The implementation uses public UIKit properties and generated objc2 bindings only; it adds no Swift source, private API, hand-written Objective-C runtime call, Blocks, or Swift ABI dependency.

## Validation and limits

Focused host tests cover optional text conversion and framework-owned trait combination. Device and simulator `cargo check`/Clippy validate compilation against the installed targets and SDK, not a live application. No simulator launch, physical-device run, VoiceOver session, accessibility audit, user study, or App Store review result is claimed.
