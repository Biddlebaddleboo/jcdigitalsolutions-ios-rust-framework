# Rust-owned UIKit app slice

This example enters UIKit from Rust, registers a Rust-defined `UIApplicationDelegate`, creates a `UIWindow` and native `UIViewController`, `UIView`, `UILabel`, and `UIButton`, then routes `UIControl` target/action back to a Rust method. The app runtime uses no Swift source or generated Swift. A minimal Xcode project is used only for unsigned archive packaging; the example uses no permission prompt, entitlement, or private API.

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
| Foreign callback panic containment | Implemented helper and used by both Rust callbacks | Rust `catch_unwind` around `define_class!` callback bodies | Panic hook still runs; abort-mode panics terminate the process. No panic payload is exposed. |
| Other capability families | Deferred | No additional Apple frameworks linked by this example | Clipboard, sharing, files/preferences, security, network, background, sensors, camera/audio, graphics, ML, personal stores, cloud/accounts, maps/AR, and extensions remain unimplemented. No Swift-only residual was assessed or passed to Workstream C here. |

The `objc2` / `objc2-ui-kit` / `objc2-foundation` dependencies are kept in the iOS example/backend only; `objc2-ui-kit` disables default features and enables the small UIKit surface used here. This uses generated bindings rather than custom messaging and keeps the dependency substitution point at the iOS adapter. No portable contract has an Objective-C type.
