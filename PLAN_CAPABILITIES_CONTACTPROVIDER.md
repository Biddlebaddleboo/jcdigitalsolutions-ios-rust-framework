# D87 — ContactProvider feasibility (row 102)

## Disposition

Keep row `102-extension-entitlement-capabilities-contactprovider` at X. The installed SDK has a useful scalar query, `ContactProviderManager.isEnabled`, but the public surface is Swift-only. The installed framework header exposes no Objective-C class, selector, C function, or C struct for that query; the available Rust bindings also expose no ContactProvider API. Do not add a Rust backend or claim extension support from a Contacts authorization facade.

A future Rust caller could use a separately reviewed Swift-to-C bridge that constructs `ContactProviderManager` and returns both initialization errors and `isEnabled`. That bridge is not an existing SDK C API or a Rust/Objective-C binding, and it would not remove the host extension and user-enable requirements below. No bridge or package is in scope here.

## Local SDK and binding evidence

The inspected toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5. The installed `ContactProvider.framework` Swift interface marks its public API `@available(iOS 18, *)` and marks macOS, Mac Catalyst, watchOS, tvOS, and visionOS unavailable.

`System/Library/Frameworks/ContactProvider.framework/Headers/ContactProvider.h` declares only `ContactProviderVersionNumber` and `ContactProviderVersionString`. The module interface declares `ContactProviderManager.init(domainIdentifier:)`, `isEnabled`, and async/throwing `enable`, `disable`, `signalEnumerator`, `invalidate`, and `reset` methods, plus the extension and enumeration protocols. The framework `.tbd` exports Swift-mangled symbols; it does not declare a C entry point for `isEnabled`. The interface marks only `deinit` with `@objc`; it does not expose the manager query as an Objective-C selector.

`cargo search objc2-contact-provider --limit 5` returned no crate result. A search of the local Cargo registry and generated Rust sources found no `ContactProviderManager`, `ContactProviderExtension`, or `objc2-contact-provider` binding. The available `objc2-contacts` binding concerns the separate Contacts framework and does not expose ContactProvider.

## Candidate query and semantics

Apple defines `ContactProviderManager.isEnabled -> Bool` as the state of whether the person enabled that extension domain. Apple documents `enable()` as async because it may prompt for approval. The `isEnabled` getter itself has no documented prompt. However, constructing the manager is not a pure query: `init(domainIdentifier:)` may register `DefaultContactProviderDomain` if needed, and can throw `extensionNotFound` or `featureNotAvailable`.

Even with a bridge, this Boolean would mean only “the provider domain is enabled.” It would not prove that the extension can launch, enumeration succeeds, sync is current, contacts are available to another app, or any contact data exists. The facade would need a distinct error result for manager initialization; false must not stand in for a missing extension or unsupported feature.

## Host, consent, and lifecycle limits

ContactProvider is an app-extension service, not generic permission to read the user’s Contacts database. Apple requires the host app to contain an extension whose `Info.plist` has:

```xml
<key>EXAppExtensionAttributes</key>
<dict>
  <key>EXExtensionPointIdentifier</key>
  <string>com.apple.contact.provider.extension</string>
</dict>
```

Installing the host app installs the extension, but the system does not run it until its domain is enabled. The system then calls `configure(for:)`; the extension must supply an enumerator and support async content/change enumeration. Its invalidation method is also async. `ContactProviderManager` is for the containing app, not the extension. The person can enable or disable the provider in Settings; disabling it makes its contacts unavailable to other apps. `enable()` may request user approval.

No ContactProvider-specific entitlement or usage-description key appears in the inspected ContactProvider public header or the Apple ContactProvider docs. Do not infer a generic Contacts usage-description requirement from this API. The separate `com.apple.developer.contacts.notes` entitlement applies to access to note fields on the user’s Contacts entries; it is not evidence of a ContactProvider requirement.

## Feasibility conclusion

There is no direct public Rust, C, or Objective-C call surface for even the narrow `isEnabled` snapshot in the installed SDK and bindings. Calling its Swift-mangled symbol from Rust would not provide a supported C/Objective-C contract. A Swift bridge plus an extension target could make a narrow host query possible, but that is a distinct interop and app-target design, not a Rust/C backend justified by this audit. Keep row 102 at X until a supported bridge path and host extension integration are in scope.

No code, package manifest, global matrix, index, or CI file changed. No tests, builds, or runtime probes were run.

## Evidence references

- Installed SDK header: `iPhoneOS.sdk/System/Library/Frameworks/ContactProvider.framework/Headers/ContactProvider.h`
- Installed API interface: `iPhoneOS.sdk/System/Library/Frameworks/ContactProvider.framework/Modules/ContactProvider.swiftmodule/arm64e-apple-ios.swiftinterface`
- Installed framework export map: `iPhoneOS.sdk/System/Library/Frameworks/ContactProvider.framework/ContactProvider.tbd`
- [ContactProvider framework overview](https://developer.apple.com/documentation/contactprovider)
- [ContactProviderManager](https://developer.apple.com/documentation/contactprovider/contactprovidermanager)
- [ContactProviderManager initializer](https://developer.apple.com/documentation/contactprovider/contactprovidermanager/init%28domainidentifier%3A%29)
- [ContactProviderExtension and extension Info.plist configuration](https://developer.apple.com/documentation/contactprovider/contactproviderextension)
- [ContactProviderError cases](https://developer.apple.com/documentation/contactprovider/contactprovidererror)
- [Apple entitlement catalog](https://developer.apple.com/documentation/bundleresources/entitlements)
- [Contacts notes entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.contacts.notes)
