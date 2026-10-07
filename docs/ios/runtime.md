# iOS native runtime and UIKit slice

## Scope and capability status

Workstream B adds `ios-runtime` as a narrow native adapter, plus the `ios-minimal` app example. The implemented surfaces are:

| Surface | Status | Public API | Permissions / entitlements |
|---|---|---|---|
| Rust-owned UIKit entry and one window | Minimal vertical slice implemented | UIKit [`UIApplicationMain`](https://developer.apple.com/documentation/uikit/uiapplicationmain%28_%3A_%3A_%3A_%3A%29-1yub7) through `objc2_ui_kit::UIApplication::main`, `UIApplicationDelegate`, `UIWindow`, `UIViewController`, `UIView`, `UILabel`, `UIButton`, and [`UIControl.addTarget:action:forControlEvents:`](https://developer.apple.com/documentation/uikit/uicontrol/addtarget%28_%3Aaction%3Afor%3A%29?changes=l_9&language=objc) | None for this UI. |
| Main-thread proof | Implemented | `objc2::MainThreadMarker`, wrapped by `ios_runtime::main_thread::MainThread` | iOS-only; this is a typed proof, not a dispatcher. |
| Rust panic boundary | Implemented | Rust `catch_unwind` in `ios_runtime::ffi` around native callback bodies | Panics are contained when the Rust panic strategy unwinds; `panic=abort` aborts as usual. |
| All other capability families | Deferred | No backend crate in this change | No permissions or entitlements are requested. |

The UIKit slice is intentionally a single-window app-delegate launch, not a general scene-aware app lifecycle. It does not add a portable UI contract, view tree, custom renderer, share/pasteboard API, layout system, accessibility facade, or broad native wrapper surface. Apple owns the UIKit hierarchy and compositor; framework-owned UI state remains ordinary Rust state.

## Build and toolchain record

Execution host: macOS 26.6.2, build 25G83, x86_64. Xcode 26.6, build 17F113. `xcodebuild -showsdks` reports iOS 26.5 and iOS Simulator 26.5. Rust 1.94.1 (`e408947bfd200af42db322daf0fadfe7e26d3bd1`, LLVM 21.1.8), Cargo 1.94.1. This is below the Xcode 27.x assumption in `PLAN.md`; no Xcode 27 claim is made.

Build a Release simulator app bundle (arm64):

```sh
./examples/ios-minimal/build.sh simulator
```

Build and bundle for iOS device (arm64), unsigned:

```sh
./examples/ios-minimal/build.sh device
```

The device command proves only compile/link/package on the selected SDK. It does not sign, provision, install, or launch on a device. No simulator launch is claimed: this x86_64 host cannot run the arm64 simulator executable produced by the required `aarch64-apple-ios-sim` target. Xcode archive validation is also not claimed. The bundle contains no `.swift` file; the application executable is Rust.

## Ownership and callback contract

- `UIApplication::main` enters UIKit from Rust and names the Rust-defined Objective-C delegate class. UIKit calls the launch delegate method on the main thread.
- The `MainThread` wrapper owns `objc2::MainThreadMarker`; consuming it supplies the marker required by generated UIKit constructors. It cannot be manufactured on a worker thread.
- The app delegate owns one `Retained<UIWindow>` and one `Retained<ButtonTarget>` in `OnceCell`s. UIKit owns the controller/view hierarchy once attached to the window. `Retained<T>` is native retain/release ownership, not an `Arc` wrapper.
- The target retains only the label and a `Cell<u32>` tap count. Apple documents that `UIControl` does not retain its target, so the application delegate keeps the action receiver alive; no callback registry or universal closure box exists. Teardown drops Rust ivars and releases native strong references.
- The target/action selector enters a Rust-defined Objective-C method and calls a Rust callback body. The body catches Rust panic before returning to UIKit. A panic can advance the tap count before the label text update, so callback state is not transactional. Callback execution is synchronous on the main thread; repeated taps are allowed, callback reentrancy is possible, and the demonstration does not claim exactly-once semantics.
- UI objects and state remain on the main thread. No cross-thread ownership, async operation, cancellation API, or error mapping is part of this slice.

## Dependency and substitution notes

The example uses `objc2` 0.6.5, `objc2-ui-kit` 0.3.2, `objc2-foundation` 0.3.2, and `objc2-core-foundation` 0.3.2. It needs Objective-C class/protocol definition and generated public UIKit/Foundation APIs; portable core primitives do not provide that platform ABI. The selected crates preserve native retain/release and UIKit main-thread semantics. UIKit default features are disabled; only the classes and geometry used by this slice are enabled. This adds no third-party runtime beyond the narrow binding graph, and no binding type enters portable contracts. `ios-runtime` uses `std` for `catch_unwind`, which is permitted in this platform backend and does not affect `no_std` portable crates. Reimplementing `objc_msgSend`, ARC, UIKit, or Blocks would increase unsafe surface without reducing native work. Dependencies stay in iOS-specific manifests; replacement is bounded to `ios-runtime` and the capability backend manifests.

No special Info.plist usage description, entitlement, background mode, or restricted capability is needed for this app slice. All used APIs are public UIKit surfaces and the example does not access private selectors or frameworks.
