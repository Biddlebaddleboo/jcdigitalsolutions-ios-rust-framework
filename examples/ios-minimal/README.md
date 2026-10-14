# Rust-owned UIKit app slice

This example enters UIKit through Rust-owned `IosApplicationHost`, creates a separate `UIWindow`, root controller, and state owner for each connected scene, and tears each scene down on disconnect. Its default scene demonstrates UIKit buttons, a native alert, staged local-file sharing and document export, and `IosCustomRenderSession` with Rust-owned RGBA8 frames, touch, text input, accessibility, and display-link lifecycle. These are imperative native primitives, not SwiftUI, a widget system, or a declarative renderer. The app runtime uses no Swift source or generated Swift. A minimal Xcode project is used only for unsigned archive packaging; the example uses no permission prompt, entitlement, or private API.

## Build

From this directory:

```sh
./build.sh simulator
```

The script builds an arm64 iOS Simulator Release executable with Cargo and packages it as an unsigned `.app` under `target/ios-minimal/simulator/ios-minimal.app`. To compile for an iOS device target without signing:

```sh
./build.sh device
```

The device command only builds and bundles an unsigned app. Device installation, signing, provisioning, and physical-device validation are not verified here. The standard simulator command builds the required arm64 `aarch64-apple-ios-sim` target; this Intel host cannot run that executable.

For a supplemental Intel-host simulator run, install the `x86_64-apple-ios` Rust target and build its separate bundle:

```sh
rustup target add x86_64-apple-ios
./build.sh simulator-x86
codesign --force --sign - --timestamp=none ../../target/ios-minimal/simulator-x86/ios-minimal.app
xcrun simctl install booted ../../target/ios-minimal/simulator-x86/ios-minimal.app
xcrun simctl launch booted dev.jcdigitalsolutions.ios-minimal
```

The current scene sample launched on the iOS 18.0 iPhone 16 Pro x86_64 Simulator. The captured screen showed the scene-hosted buttons, RGBA8 view, text-input view, and a live display-link status update. This establishes one scene connection and visible root content only; no button action, alert, text edit, share/export interaction, multi-scene transition, arm64 Simulator run, or physical-device behavior was exercised.

The older single-window demo is isolated behind Cargo feature `legacy-app-demo`; its
`--exercise-rust-button-callback` argument tests programmatic UIKit target/action dispatch only,
not physical touch or arm64/device behavior.

The default scene app paints a 64×32 RGBA8 frame and changes its color field after touch events; its display link pauses while the scene is inactive and detaches at disconnect. The legacy/custom-Metal demo also retains `--custom-render-host-demo` and `--exercise-custom-render-host-lifecycle` modes. See [the custom-render host contract](../../docs/ios/custom-render-host.md).

The default scene app's Text button requests focus on its native `UITextView` surface. UIKit owns the active text buffer and IME composition; the Rust callback receives complete text plus selection and marked ranges in UTF-16 code units. A successful focus request or compile/link check does not prove software-keyboard display, typed edits, callback delivery, or IME composition. See [the custom-render text-input contract](../../docs/ios/custom-render-text-input.md).

The touch and accessibility adapters remain independently usable and are also composed by `IosCustomRenderSession`. Their compile/link gates and deterministic state/tree regressions do not prove native touch delivery or VoiceOver runtime behavior; this Simulator has no `simctl io` touch-injection command, and no live VoiceOver interaction was performed. See the [touch contract](../../docs/ios/custom-render-touch.md) and [accessibility contract](../../docs/ios/custom-render-accessibility.md).

Metal support is opt-in. Build it with `./build.sh simulator-x86-metal` (or `simulator-metal` for arm64 Simulator, `device-metal` for arm64 Device); ordinary builds remain on the CPU path. Launch with `--custom-metal-render-demo` to show a Metal-rendered frame. Launch with `--custom-metal-render-benchmark` to compare CPU and Metal full/sparse presentation at requested 30/60/120-Hz pacing; the results are saved in the app data container under `tmp/custom-render-metal-benchmark.txt`. The benchmark is Simulator-only evidence and its requested pacing is not actual display refresh. See the [custom-render host and Metal notes](../../docs/ios/custom-render-host.md).

The Rust target components are `aarch64-apple-ios-sim` and `aarch64-apple-ios`. The current execution host recorded by the backend guide is macOS 26.6.2, Xcode 26.6 (17F113), iOS and Simulator SDK 26.5. The planning baseline assumes Xcode 27.x; that toolchain is not present on this host.

## Unsigned Xcode archive

Run from the repository root on macOS with the iPhoneOS SDK installed:

```sh
cargo xtask archive-smoke
```

This builds the Release device executable and creates `target/ios-minimal/archive/ios-minimal.xcarchive` with Xcode's archive action. It disables signing and does not validate provisioning, archive export, installation, launch, or physical-device behavior. See [iOS Build and Packaging Direction](../../docs/IOS_BUILD.md) for the exact `xcodebuild` command and host result.

## Ownership and thread rules

- UIKit entry and every object construction/action run on the main thread. `MainThread` carries `objc2::MainThreadMarker`; UIKit types retain their `MainThreadOnly` restrictions.
- `UIApplication` owns the delegate lifecycle. Each connected scene gets a separate controller, presentation/render/input sessions, and target; UIKit owns the view/controller hierarchy.
- The per-scene Rust owner retains its button target, which holds only a weak reference to scene state. There is no global scene registry or Rust mirror of the UIKit hierarchy. Retained UI objects are not `Send` and do not cross threads.
- Scene disconnect pauses and detaches the render session, cancels active share/export callbacks, and releases the scene owner. UIKit target/action registration remains non-closure based.
- Selector methods catch Rust panics before returning through Objective-C; `panic=abort` remains a process abort and cannot unwind into UIKit. Reentrant mutable scene actions are rejected with short `try_borrow_mut` scopes.

## Capability status

| Surface | Status | Apple API / linkage | Permissions and limitations |
|---|---|---|---|
| App entry and scene UI | Implemented low-level scene-host slice | Public UIKit `UIApplicationMain`, `UIApplicationDelegate`, `UISceneDelegate`, `UIWindowScene`, `UIWindow`, `UIViewController`, `UIView`, `UILabel`, `UIButton`, and `UIControl` | Requires the documented scene manifest and startup setup. Per-scene sample state is Rust-owned; multi-scene runtime and reconnect behavior remain unverified. |
| Native prompts | Implemented session API | Public `UIAlertController`, `UIAlertAction`, and `UITextField` | Sample demonstrates action and bounded text-field setup; live presentation, keyboard behavior, and action delivery are not established by compile/link checks. |
| Staged file share/export | Implemented opt-in APIs | Public `UIActivityViewController` and iOS 14 `UIDocumentPickerViewController` | Sharing stages at most 16 MiB in app temporary storage; export uses copy semantics. Recipient delivery and provider durability are not guaranteed. |
| Sandbox backup exclusion | Implemented explicit setter/getter | Foundation `NSURLIsExcludedFromBackupKey` | Applies only to a validated sandbox file/directory; backup services' eventual inclusion or omission is not guaranteed. |
| Composed custom render session | Implemented opt-in lifecycle composition | Public UIKit/CoreGraphics/QuartzCore APIs and existing custom-render adapters | Composes RGBA8 rendering, touch, text, accessibility, and display scheduling without a renderer or SwiftUI model. Runtime input and accessibility behavior remain unverified. |
| Runtime main-thread proof | Implemented | `objc2::MainThreadMarker` adapter in `ios-runtime` | iOS-only backend API; no portable type or scheduling facility. |
| Custom-render text input | Implemented UIKit adapter | Public `UITextView`, `UITextInput`, `UITextViewDelegate`, and keyboard notifications | Full UTF-16 text/selection/marked-range snapshots, explicit composition cancellation, and per-handle teardown. Software keyboard/IME runtime behavior is not verified. |
| Custom-render touch | Implemented UIKit adapter | Public `UIView` responder touch callbacks and `UITouch` metadata | Bounded multi-contact/coalesced event delivery and cancellation; runtime touch injection not verified. |
| Custom-render accessibility | Implemented UIKit adapter | Public `UIAccessibilityElement`, `UIAccessibilityCustomAction`, and notifications | Ordered caller-owned semantic nodes and stale-action protection; VoiceOver runtime behavior not verified. |
| Custom-render Metal | Optional UIKit/Metal adapter | Public `CAMetalLayer`, Metal textures, command buffers, and drawable presentation | BGRA8 full/damage updates; CPU fallback stays available; one Simulator comparison is documented, with no device-performance claim. |
| Foreign callback panic containment | Implemented helper and used by both Rust callbacks | Rust `catch_unwind` around `define_class!` callback bodies | Panic hook still runs; abort-mode panics terminate the process. No panic payload is exposed. |
| Other capability families | Not exercised by this example | No additional Apple frameworks linked by this executable | Existing framework crates provide separate capabilities; this sample does not link clipboard, preferences, security, networking, background work, sensors, camera/audio, ML, personal stores, cloud/accounts, maps/AR, or extensions. |

The `objc2` / `objc2-ui-kit` / `objc2-foundation` dependencies are kept in the iOS example/backend only; `objc2-ui-kit` disables default features and enables the small UIKit surface used here. This uses generated bindings rather than custom messaging and keeps the dependency substitution point at the iOS adapter. No portable contract has an Objective-C type.
