# Rust-owned UIKit app slice

This example enters UIKit from Rust, registers a Rust-defined `UIApplicationDelegate`, creates a `UIWindow` and native `UIViewController`, `UIView`, `UILabel`, and `UIButton`, then routes `UIControl` target/action back to a Rust method. It also embeds `IosCustomRenderHost`, presents Rust-owned RGBA framebuffer data, and provides opt-in touch, `IosCustomTextInput`, semantic accessibility, and Metal consumers. These are imperative native view primitives; they are not SwiftUI, a widget system, or a declarative renderer. The app runtime uses no Swift source or generated Swift. A minimal Xcode project is used only for unsigned archive packaging; the example uses no permission prompt, entitlement, or private API.

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

This host launched that bundle on the iOS 18.0 iPhone 16 Pro simulator. The label and button were legible in both light and dark appearance. The supplemental x86_64 run does not validate the required arm64 simulator target or a user touch.

To exercise the Rust target/action callback without a user tap, launch the simulator bundle with this argument:

```sh
xcrun simctl launch --terminate-running-process booted dev.jcdigitalsolutions.ios-minimal --exercise-rust-button-callback
```

The app then calls `UIControl::sendActionsForControlEvents(UIControlEvents::TouchUpInside)`, and the Rust callback changes the label to `Rust callback: 1 tap(s)`. This verifies programmatic UIKit target/action dispatch only; it does not simulate a physical touch or establish arm64/device behavior.

The same callback updates the 64×32 framebuffer from its initial frame, so the launch above submits two distinct images. To display only a Rust-painted framebuffer in a native `UIImageView`, pass `--custom-render-host-demo`; it submits two changing frames without creating a label or button, then leaves the second frame visible. For a create → present → update → resize → detach → drop lifecycle smoke, pass `--exercise-custom-render-host-lifecycle`; that mode runs the two submissions and removes the host at launch. See [the custom-render host contract](../../docs/ios/custom-render-host.md).

To attach a native `UITextView` input surface to the custom-render host, have the app request first-responder status at launch, and mirror only unmarked snapshots into its Rust-owned text model, pass `--exercise-custom-text-input`. UIKit owns the active text buffer and IME composition; the Rust callback receives complete text plus selection and marked ranges in UTF-16 code units. On the iOS 18.0 iPhone 16 Pro Max x86_64 Simulator, the rebuilt demo showed the native text view and caret after the launch focus request. The software keyboard did not appear in the captured state; typed edits, callback delivery, and IME composition remain unverified. See [the custom-render text-input contract](../../docs/ios/custom-render-text-input.md).

The `IosCustomRenderTouchInput` and `IosCustomRenderAccessibility` APIs are separate host adapters. Their compile/link gates and deterministic state/tree regressions do not prove native touch delivery or VoiceOver runtime behavior; this Simulator has no `simctl io` touch-injection command, and no live VoiceOver interaction was performed. See the [touch contract](../../docs/ios/custom-render-touch.md) and [accessibility contract](../../docs/ios/custom-render-accessibility.md).

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
- `UIApplication` owns the delegate lifecycle. The app delegate strongly retains the window and button target in Rust ivars; UIKit owns the view/controller hierarchy. `UIControl` target/action is non-closure based.
- The button target holds the label and compact tap count. There is no `Arc`, shared global registry, or Rust mirror of the UIKit hierarchy. Retained UI objects are not `Send` and do not cross threads.
- The app delegate owns the target because target/action registration does not define Rust ownership. This keeps the selector receiver alive for as long as the window is live. On delegate teardown, Rust ivars release their `Retained` Objective-C objects and UIKit tears down the native hierarchy.
- Selector methods catch Rust panics before returning through Objective-C. A panic may advance the Rust tap count before label text updates; `panic=abort` remains a process abort and cannot unwind into UIKit.
- UIKit may synchronously reenter app code. The callback uses `Cell<u32>` and only short, non-overlapping borrows; it does not retain a mutable global reference.

## Capability status

| Surface | Status | Apple API / linkage | Permissions and limitations |
|---|---|---|---|
| App entry and UI | Implemented minimal vertical slice | Public UIKit `UIApplicationMain` surface via `UIApplication::main`, `UIApplicationDelegate`, `UIWindow`, `UIViewController`, `UIView`, `UILabel`, `UIButton`, and `UIControl` | No permission or entitlement. Single-window app-delegate lifecycle; no scene lifecycle, layout engine, accessibility abstraction, pasteboard, share sheet, or broad UI facade. |
| Runtime main-thread proof | Implemented | `objc2::MainThreadMarker` adapter in `ios-runtime` | iOS-only backend API; no portable type or scheduling facility. |
| Custom-render text input | Implemented UIKit adapter | Public `UITextView`, `UITextInput`, `UITextViewDelegate`, and keyboard notifications | Full UTF-16 text/selection/marked-range snapshots, explicit composition cancellation, and per-handle teardown. Software keyboard/IME runtime behavior is not verified. |
| Custom-render touch | Implemented UIKit adapter | Public `UIView` responder touch callbacks and `UITouch` metadata | Bounded multi-contact/coalesced event delivery and cancellation; runtime touch injection not verified. |
| Custom-render accessibility | Implemented UIKit adapter | Public `UIAccessibilityElement`, `UIAccessibilityCustomAction`, and notifications | Ordered caller-owned semantic nodes and stale-action protection; VoiceOver runtime behavior not verified. |
| Custom-render Metal | Optional UIKit/Metal adapter | Public `CAMetalLayer`, Metal textures, command buffers, and drawable presentation | BGRA8 full/damage updates; CPU fallback stays available; one Simulator comparison is documented, with no device-performance claim. |
| Foreign callback panic containment | Implemented helper and used by both Rust callbacks | Rust `catch_unwind` around `define_class!` callback bodies | Panic hook still runs; abort-mode panics terminate the process. No panic payload is exposed. |
| Other capability families | Deferred | No additional Apple frameworks linked by this example | Clipboard, sharing, files/preferences, security, network, background, sensors, camera/audio, graphics, ML, personal stores, cloud/accounts, maps/AR, and extensions remain unimplemented. No Swift-only residual was assessed or passed to Workstream C here. |

The `objc2` / `objc2-ui-kit` / `objc2-foundation` dependencies are kept in the iOS example/backend only; `objc2-ui-kit` disables default features and enables the small UIKit surface used here. This uses generated bindings rather than custom messaging and keeps the dependency substitution point at the iOS adapter. No portable contract has an Objective-C type.
