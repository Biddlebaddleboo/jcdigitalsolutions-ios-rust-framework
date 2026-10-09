# D88 — BrowserEngineKit feasibility (row 101)

## Disposition

Keep row `101-extension-entitlement-capabilities-browserenginekit-for-entitled-apps` at X for now. BrowserEngineKit has typed Objective-C APIs and the Rust crate `objc2-browser-engine-kit` 0.3.2 has generated bindings, so lack of bindings is not the blocker. The blocker is scope: the useful core APIs launch and manage several privileged extension processes, grant process capabilities, create XPC channels, and require a real browser architecture plus Apple-issued entitlements and region-specific distribution approval. The current repo has no such host/extension implementation.

There is one narrow Rust-callable observation: `BEProcessCapabilityGrant.isValid() -> bool` reports whether a system-granted process capability remains valid. It is only useful with a live grant previously returned by a browser process `grantCapability` call. It does not report entitlement presence, app eligibility, BrowserEngineKit support, extension health, or whether a browser engine works. Treat it as a possible utility inside a future entitled host-process implementation, not a standalone BrowserEngineKit capability or a reason to promote this row.

## Local SDK and Rust binding evidence

The inspected toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5. The installed `BrowserEngineKit.framework` umbrella header imports `BrowserEngineCore` and exposes Objective-C headers for process management, capability grants, scrolling, text input, drag, accessibility, downloads, and related browser UI. The module map re-exports the framework headers; the `.tbd` re-exports `BrowserEngineCore.framework`.

The host-side process types `BEWebContentProcess`, `BENetworkingProcess`, and `BERenderingProcess`, plus `BEProcessCapability` and `BEProcessCapabilityGrant`, have an iOS 17.4 API floor in the installed headers. The bundle-identifier process factory overloads have an iOS 18.2 floor. Grant invalidation-handler overloads have an iOS 17.6 floor. Those are symbol availability floors, not permission to use an alternative engine on every iOS device or storefront.

The crates.io package `objc2-browser-engine-kit` 0.3.2 provides feature-gated generated Rust bindings, including `BEWebContentProcess`, `BENetworkingProcess`, `BERenderingProcess`, `BEExtensionProcess`, `BEProcessCapability`, and `BEProcessCapabilityGrant`. Its `BECapability` feature exposes the grant protocol; the generated `isValid` method is an unsafe Objective-C message call and would need a reviewed Rust wrapper around a valid, retained grant object. The crate source binds the Objective-C host process APIs, but does not provide the Swift `WebContentExtension`, `NetworkingExtension`, `RenderingExtension`, or `ExtensionFoundation.AppExtension` protocol implementations needed to define the three extension targets.

The repo search found no existing BrowserEngineKit code or C facade. This workstream does not add a package or integrate the third-party binding.

## Entitlement, approval, and distribution boundary

For a dedicated alternative-browser app, Apple documents these host requirements:

- `com.apple.developer.web-browser-engine.host` on the browser app
- `com.apple.developer.web-browser` so the app can act as the default browser
- Apple-issued entitlement profile and approval for the relevant region
- Separate web content, networking, and rendering extension targets, with extension-point identifiers `com.apple.web-browser-engine.content`, `com.apple.web-browser-engine.networking`, and `com.apple.web-browser-engine.rendering`
- `com.apple.developer.web-browser-engine.webcontent`, `com.apple.developer.web-browser-engine.networking`, and `com.apple.developer.web-browser-engine.rendering` on their respective extensions
- `web-browser-engine` in `UIRequiredDeviceCapabilities`

For in-app browsing in a non-browser app, Apple instead requires the embedded-engine entitlement `com.apple.developer.embedded-web-browser-engine` and the association entitlement `com.apple.developer.embedded-web-browser-engine.engine-association`; the app must meet Apple’s in-app-browsing criteria. The extension and engine shape differs from a dedicated browser. App Store Connect rejects use of browser-engine entitlements on non-browser apps or components outside their documented role.

Apple’s current region guidance lists EU availability from iOS 17.4 and iPadOS 18, and Japan availability from iOS 26.2. The entitlement request and profile are restricted to qualifying browser apps or qualifying embedded-engine apps. The EU and Japan guidance sets functional, distribution, security, and privacy criteria, plus ongoing obligations such as timely security updates. The default-browser selection is a separate user choice; this API does not present a generic permission prompt or an entitlement-status query.

The extension targets use distinct processes and lifecycles. The host launches each process asynchronously, handles interruption callbacks, creates XPC connections, grants and later invalidates process capabilities, and stops processes when no longer needed. One extension cannot load another. Apple requires `arm64e` for browser-app and extension executables that use the alternative-engine process model; Simulator does not support `arm64e`. Embedded-engine apps use a different, restricted architecture and do not use these browser extension targets or JIT.

## Candidate API and limits

The candidate scalar operation is the read-only `BEProcessCapabilityGrant.isValid` property. Apple defines true only while the system honors the grant and the app has not invalidated it. Obtaining the grant first requires a live `BEWebContentProcess`, `BENetworkingProcess`, or `BERenderingProcess` plus a successful `grantCapability` request. That request may fail; extension launch may fail or be interrupted; and process invalidation makes later process calls invalid. A bool snapshot cannot represent those prior errors or lifecycle transitions.

An `isValid` wrapper could be precise if a future API accepts only an already-owned grant reference and returns the momentary validity bit. Such a wrapper would be an opaque-handle utility for a qualified host app. It would not implement process launch, XPC, any extension, WebKit replacement, JIT, browser UI, or entitlement checks. The current D88 scope does not add this utility because it would not independently satisfy the row’s browser-engine capability claim.

## Feasibility conclusion

The generated Rust binding makes direct host-side Objective-C calls technically possible. A useful row implementation still needs a defined extension architecture, the Swift-only extension protocol/lifecycle seam or a supported replacement, correct XPC and interruption ownership, per-target entitlements, Apple approval, and regional distribution rules. No safe standalone Rust value facade or entitlement-status query was found. Keep row 101 at X until a bounded host-process operation is explicitly accepted and its live-grant ownership contract is specified.

## Evidence and checks

- Installed framework: `iPhoneOS.sdk/System/Library/Frameworks/BrowserEngineKit.framework`
- Key headers: `Headers/BrowserEngineKit.h`, `Headers/BEWebContentProcess.h`, `Headers/BENetworkingProcess.h`, `Headers/BERenderingProcess.h`, `Headers/BECapability.h`
- Installed module interface: `Modules/BrowserEngineKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- Rust package: `objc2-browser-engine-kit` 0.3.2, generated source for `BEWebContentProcess`, `BEExtensionProcess`, and `BECapability`
- Read-only environment checks: Xcode 26.6 build `17F113`; iPhoneOS SDK 26.5
- No tests, builds, link probes, or runtime probes ran

`cargo info objc2-browser-engine-kit@0.3.2` was used once to confirm the published package and its feature metadata. It reported a lock update for `objc2-file-provider v0.3.2`, which the parent confirmed is required by active D85/B75 work. This report makes no Cargo manifest or lockfile edit; the shared lock remains under root ownership.

## Primary references

- [BrowserEngineKit overview](https://developer.apple.com/documentation/browserenginekit)
- [Creating browser extensions in Xcode](https://developer.apple.com/documentation/browserenginekit/creating-browser-extensions-in-xcode)
- [Managing the browser extension life cycle](https://developer.apple.com/documentation/browserenginekit/managing-the-browser-extension-lifecycle)
- [ProcessCapability.Grant.isValid](https://developer.apple.com/documentation/browserenginekit/processcapability/grant/isvalid)
- [Web Browser Engine Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser-engine.host)
- [Rendering extension entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser-engine.rendering)
- [Networking extension entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser-engine.networking)
- [Web content extension entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser-engine.webcontent)
- [Embedded Browser Engine Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.embedded-web-browser-engine)
- [Embedded Browser Engine Association Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.embedded-web-browser-engine.engine-association)
- [Alternative browser engines in the European Union](https://developer.apple.com/support/alternative-browser-engines/)
- [Alternative browser engines in Japan](https://developer.apple.com/support/alternative-browser-engines-jp/)
- [`objc2-browser-engine-kit` 0.3.2 crate](https://docs.rs/objc2-browser-engine-kit/0.3.2/objc2_browser_engine_kit/)
- [Generated `BECapability` Rust binding](https://docs.rs/objc2-browser-engine-kit/0.3.2/src/objc2_browser_engine_kit/generated/BECapability.rs.html)
