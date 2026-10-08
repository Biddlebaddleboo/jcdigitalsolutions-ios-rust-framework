# iOS UIKit accessibility metadata

## Scope and status

`ios-accessibility` is a partial, iOS-only adapter for synchronous accessibility metadata on a borrowed, caller-owned `UIView`. It exposes setters for `isAccessibilityElement`, `accessibilityLabel`, `accessibilityHint`, `accessibilityValue`, and seven standard traits. UIKit remains the accessibility implementation; this crate does not create or retain views, build a view tree, or replace UIKit behavior when its setters are unused.

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

The Rust API does not accept a raw trait integer or pointer. The adapter ORs the selected UIKit constants; it does not hard-code their numeric values. Trait-mapping checks verify empty-set handling, combination, order independence, and that each update is derived only from the provided set. They do not model or simulate assistive technology.

## Availability and framework imports

The inspected host is macOS 26.6.2, Xcode 26.6 (build 17F113), with iPhoneOS and iPhoneSimulator SDK 26.5. This is below the repository's Xcode 27.x planning baseline. Both SDK `SDKSettings.plist` files set `MinimumDeploymentTarget` to iOS 12.0, so this host cannot verify a binary targeted below iOS 12.0. The exact inspected declarations are in:

- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibility.h`: category `NSObject (UIAccessibility)` declares `BOOL isAccessibilityElement`, nullable copied `NSString *accessibilityLabel`, nullable copied `NSString *accessibilityHint`, nullable copied `NSString *accessibilityValue`, and `UIAccessibilityTraits accessibilityTraits`. These declarations are marked unavailable on watchOS and have no iOS `API_AVAILABLE` version annotation in this header.
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/UIKit.framework/Headers/UIAccessibilityConstants.h`: `UIAccessibilityTraits` is `uint64_t`; the selected constants are declared there. `UIAccessibilityTraitHeader` has an explicit `API_AVAILABLE(ios(6.0), watchos(2.0))` annotation. The other six exposed traits and `UIAccessibilityTraitNone` have no explicit iOS minimum annotation in this header.
- The same declarations are enabled by `objc2-ui-kit` 0.3.2 features `UIAccessibility`, `UIAccessibilityConstants`, `UIResponder`, and `UIView`; text conversion uses `objc2-foundation` 0.3.2 `NSString`.

The selected surface's lowest explicit iOS availability is iOS 6.0, set only by `UIAccessibilityTraitHeader`. This is an API floor derived from the active SDK declarations, not the binary floor for this host and not a crate deployment target: the crate does not set one. A temporary external `cdylib` consumer was linked with `IPHONEOS_DEPLOYMENT_TARGET=12.0` for both targets. `vtool -show-build` reported device `LC_BUILD_VERSION` `minos 12.0` / SDK 26.5 and simulator `minos 14.0` / SDK 26.5; the simulator Rust target did not lower its link minimum to 12.0. `otool -L` on both outputs listed UIKit, Foundation, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` showed the seven selected trait constants, `UIAccessibilityTraitNone`, and `objc_msgSend`. This proves compile/link imports only, not runtime or VoiceOver behavior. UIKit and Foundation are the required Apple framework imports; `objc2-ui-kit` supplies the UIKit framework link declaration.

No permission prompt, usage-description key, or entitlement is part of these property setters. The implementation uses public UIKit properties and generated objc2 bindings only; it adds no Swift source, private API, hand-written Objective-C runtime call, Blocks, or Swift ABI dependency.

## Validation and limits

Focused host tests cover optional text conversion and framework-owned trait combination. Device and simulator `cargo check`/Clippy validate compilation against the installed targets and SDK, not a live application. No simulator launch, physical-device run, VoiceOver session, accessibility audit, user study, or App Store review result is claimed.
