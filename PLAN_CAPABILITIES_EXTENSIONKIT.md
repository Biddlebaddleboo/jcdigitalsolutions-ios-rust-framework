# PLAN_CAPABILITIES_EXTENSIONKIT.md — Workstream D86: Row 100 Feasibility Gate

## Status

D86 found a host-scoped extension inventory snapshot in Swift `ExtensionFoundation`, but no ordinary Rust-callable ExtensionFoundation binding. Its `AppExtensionPoint.Monitor` API requires a host-defined extension point and reports enabled, disabled, and unapproved app extensions for that host; it does not report general ExtensionKit availability or prove that an extension process is running. The public iOS ExtensionKit view controllers are UI/lifecycle APIs, and `objc2-extension-kit` 0.3.2 exposes only macOS versions of those classes. Keep row `100-extension-entitlement-capabilities-extensionkit-foundation-where-feasible` at `X`; no implementation or matrix change is part of D86.

## Objective

Assess whether row 100 has a bounded public Rust-callable value/status operation that can stand alone without an app-extension host model, extension bundle metadata, system approval, process launch, or UI/XPC lifecycle.

## Installed SDK and generated binding evidence

- Inspected Xcode 26.6 build `17F113` and the iPhoneOS SDK 26.5. The installed Swift interfaces target iOS 26.5 and identify Swift 6.3.2.
- `ExtensionFoundation.framework/Headers` contains only `EXMacros.h` and `ExtensionFoundation.h`; the umbrella imports the macros header and provides no Objective-C API declarations for app extension models, discovery, process launch, or connection management. The public iOS API is declared in `ExtensionFoundation.swiftmodule/arm64e-apple-ios.swiftinterface`.
- The Swift `AppExtension`, `AppExtensionConfiguration`, and app-extension scene model are available from iOS 16.0. `AppExtensionPoint` and `AppExtensionPoint.Monitor` are available from iOS 26.0; `AppExtensionPoint.Capabilities` is available from iOS 26.2. The monitor's `init(appExtensionPoint:)` is `async throws`; `state` and `identities` return Swift value snapshots. `state` contains enabled identities and counts for disabled and unapproved extensions. `identities` contains extensions currently available to the host's monitored extension point, not every installed extension or an OS-level support value.
- The monitor is the strongest candidate for a status-like slice, but it is Swift-only. `ExtensionFoundation.swiftinterface` declares Swift structs, protocols, a generic extension-point model, actor-isolated requirements, and async APIs. No C/Objective-C declaration or `objc2-extension-foundation` package was found (`cargo search objc2-extension-foundation --limit 3` returned no package result).
- `ExtensionKit.framework/Headers/EXAppExtensionBrowserViewController.h` declares a UIKit controller available from iOS 18.0. It presents out-of-process system UI for a host to let a person enable or disable app extensions; it is not a scalar status query.
- `ExtensionKit.framework/Headers/EXHostViewController.h` declares a UIKit host controller available from iOS 26.0. It hosts the remote UI for one app-extension identity and scene. Its activation/deactivation delegate methods are the extension lifecycle boundary, and `makeXPCConnectionWithError:` is called from the activation callback to connect to the extension scene. The Swift interface defines `EXHostViewController.Configuration(appExtension:sceneID:)`; the configuration type is not an Objective-C struct.
- `objc2-extension-kit` 0.3.2 is available and was inspected from generated source after `cargo info objc2-extension-kit@0.3.2`. Its `EXHostViewController` and `EXAppExtensionBrowserViewController` declarations and delegate callbacks are gated to `target_os = "macos"`; the crate provides no iOS declarations for the iOS 18/26 SDK APIs. Its Rust feature list covers the older Objective-C view-controller APIs and does not bind the Swift `ExtensionFoundation` model or `EXHostViewController.Configuration`.

## Bundle, host, and user-approval boundary

- A host that supports app extensions defines its own `AppExtensionPoint` types. The extension binds to one host point with `AppExtensionPoint.Bind`; the host and extension must build matching extension-point metadata. Apple's `EX_ENABLE_EXTENSION_POINT_GENERATION=YES` build setting directs Xcode to emit the point definition/binding information into the built app or extension bundle (`.appext` metadata for the host; binding information in the extension's `Info.plist`). The system uses that metadata at installation time to match hosts and extensions.
- `AppExtensionPoint.Monitor` is not a global catalog. The host creates it for one of its own extension points, and the system validates the matching installed extensions asynchronously. The monitor reports only extensions that are enabled and approved as usable; the device owner may disable extensions, and newly installed extensions require owner approval before host use. `state` reports separate disabled/unapproved counts.
- Discovering an identity does not launch the extension or establish an XPC connection. The host creates `AppExtensionProcess` with an `AppExtensionIdentity`, handles startup/throws and interruption, keeps a strong process reference for the connection lifetime, uses `makeXPCConnection()` or `makeXPCSession()`, and later calls `invalidate()` / releases the process. A monitor snapshot must not be described as process readiness, connection success, or operation success.
- To host custom extension UI, both the host and extension adopt ExtensionKit. The host configures `EXHostViewController` with an extension identity and scene ID, presents it in UIKit, receives main-actor activation/deactivation callbacks, and opens/closes the scene-specific XPC connection. This is UI integration, not a status facade.
- There is no single ExtensionKit entitlement or privacy permission that applies uniformly to all host-defined extension points in the inspected API. Any concrete extension point, platform extension type, data access, or system service may impose separate signing or permission rules; D86 does not infer them. The inspected generic APIs expose no privacy usage-description key or authorization prompt. System owner approval and the extension browser UI are separate management semantics, not an app permission Boolean.

## Feasibility result and next evidence

Do not add a portable contract or iOS Rust backend for a generic "ExtensionKit available" status. The only candidate snapshot is a Swift host-model query introduced in iOS 26.0; it requires a concrete extension point emitted with the host bundle, asynchronous system validation, and a Rust/Swift interop decision. The iOS Objective-C view-controller surfaces require the host UI and extension lifecycle, and the available `objc2-extension-kit` 0.3.2 binding does not expose them on iOS.

If a future product explicitly adopts a custom host/app-extension model, a narrower host-scoped inventory contract may be considered, but only after:

1. Choosing one concrete `AppExtensionPoint` and documenting its allowed extension identities, versioning, capability rules, bundle metadata, and point-specific signing/permission requirements.
2. Deciding whether a Swift bridge is permitted or waiting for a generated Rust binding that exposes the iOS 26 `ExtensionFoundation` API. Do not call the Swift API through guessed raw ABI or treat the macOS-only binding as iOS support.
3. Defining a snapshot that preserves `identities`, `disabledCount`, `unapprovedCount`, and asynchronous/error semantics separately; do not collapse these into generic availability or authorization.
4. Keeping discovery separate from app-extension process launch, XPC transport, interruption/retry, invalidation, custom UI, and browser UI.
5. Verifying the selected iOS deployment floor and host/extension `.appext` metadata in a matching signed host and extension. Compile/link checks cannot prove installation matching, owner approval, process launch, or XPC behavior.

## Deferred work

- No `framework-ui` or other portable facade, iOS ExtensionKit backend, ExtensionFoundation Swift ABI bridge, extension point, `.appext` metadata, app extension target, process launch, XPC protocol, custom UI, or extension browser presentation.
- No point-specific entitlement or privacy claim, owner-approval flow, runtime probe, tests, or builds.
- No canonical capability manifest, Cargo/workspace/lockfile, CI, aggregate plan, or shared-index edit.

## Apple and binding references

- [ExtensionFoundation](https://developer.apple.com/documentation/extensionfoundation)
- [Adding support for app extensions to your app](https://developer.apple.com/documentation/extensionfoundation/adding-support-for-app-extensions-to-your-app)
- [AppExtensionPoint](https://developer.apple.com/documentation/extensionfoundation/appextensionpoint)
- [AppExtensionPoint.Monitor](https://developer.apple.com/documentation/extensionfoundation/appextensionpoint/monitor)
- [AppExtensionPoint.Monitor.State](https://developer.apple.com/documentation/extensionfoundation/appextensionpoint/monitor/state-swift.struct)
- [Discovering app extensions from your app](https://developer.apple.com/documentation/extensionfoundation/discovering-app-extensions-from-your-app)
- [AppExtensionPoint.Bind](https://developer.apple.com/documentation/extensionfoundation/appextensionpoint/bind)
- [AppExtensionProcess](https://developer.apple.com/documentation/extensionfoundation/appextensionprocess)
- [AppExtensionProcess.invalidate()](https://developer.apple.com/documentation/extensionfoundation/appextensionprocess/invalidate())
- [Displaying the app extensions available to your app](https://developer.apple.com/documentation/extensionkit/displaying-the-app-extensions-available-to-your-app)
- [EXAppExtensionBrowserViewController](https://developer.apple.com/documentation/extensionkit/exappextensionbrowserviewcontroller)
- [Including extension-based UI in your interface](https://developer.apple.com/documentation/extensionkit/including-extension-based-ui-in-your-interface)
- [EXHostViewController](https://developer.apple.com/documentation/extensionkit/exhostviewcontroller)
- [EXHostViewController.Configuration](https://developer.apple.com/documentation/extensionkit/exhostviewcontroller/configuration-swift.struct)
- [`objc2-extension-kit` 0.3.2](https://docs.rs/objc2-extension-kit/0.3.2/objc2_extension_kit/)
- [Generated `EXHostViewController` binding](https://docs.rs/objc2-extension-kit/0.3.2/src/objc2_extension_kit/generated/EXHostViewController.rs.html)
