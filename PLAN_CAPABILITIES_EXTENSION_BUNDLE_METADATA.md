# PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md — D96: Row 113 Feasibility Gate

## Disposition

### B77 implementation update

B77 now implements the separate runtime-reader route: it reads only `NSExtension.NSExtensionPointIdentifier` from one caller-supplied `.appex` path. Root reclassified row 113 from the build-host boundary to a platform-exclusive `B` partial after the package and focused checks passed. Build-host plist generation, `.appext` metadata, extension-point schema validation, and App Intents processing remain unsupported. See [B77](PLAN_IOS_EXTENSION_SUPPORT.md).

Row `113-compiler-build-host-capabilities-extension-bundle-metadata-helpers-only-through-supported-xcode-public-mechanisms` has narrow, public implementation paths for ordinary extension bundle `Info.plist` metadata. A future Rust build helper can emit or validate a plist input that Xcode consumes through `INFOPLIST_FILE`; a runtime helper can read the built bundle through public `NSBundle` or `CFBundle` APIs. This supports bounded metadata tooling, not extension runtime support.

At the time of D96, row 113 was classified as `compiler_build_boundary`, so the report preferred a `C` partial for a build helper that produces or checks a documented plist input and leaves final processing to Xcode. A runtime `NSBundle`/`CFBundle` reader was identified as a separate `B` surface that alone does not implement a build-host contract. B77 has since implemented that scoped runtime operation and root reclassified the row to a platform-exclusive `B` partial. Do not claim metadata generation for all extension types, `.appext` production from Rust, extension discovery/approval/launch, or App Intents support.

## Audit scope

Inspect the generic row 113 after rows 107–112. Determine whether public Xcode metadata inputs or a Rust-callable Foundation/CoreFoundation API can support a deliberately narrow extension metadata contract.

Do not repeat the C7 App Intents processor audit for row 110. Its `Metadata.appintents` pipeline and unsupported Rust/C processor input are already recorded in `PLAN_SWIFT_APP_INTENTS.md` and `docs/swift-abi/APP_INTENTS_STAGE0.md`. ExtensionFoundation's compiler-generated `.appext` declarations are a separate Swift-only path, covered by [D86](PLAN_CAPABILITIES_EXTENSIONKIT.md).

## Supported plist build input

Apple's Xcode build settings reference documents `INFOPLIST_FILE` as the target's input property-list file. Xcode processes and merges that file with generated target values. The documented `GENERATE_INFOPLIST_FILE` setting can also generate a plist from recognized build settings. Apple's information-property-list documentation states that user-defined settings named `INFOPLIST_KEY_*` do not cause arbitrary new plist entries to be generated; a custom extension dictionary must use an explicit plist input or another documented target-specific setting.

Apple documents `NSExtensionPointIdentifier` as a required string key inside `NSExtension` for an app extension, with extension-point-specific values and attributes. Xcode extension templates include the appropriate point identifier. A Rust build helper may supply or validate a plist fragment for an explicitly selected public extension point, then leave final processing and bundle construction to Xcode. It must not write directly into a finished or signed product, guess private processor inputs, or accept arbitrary unvalidated metadata as proof of a valid extension.

This is a documented file/build-setting route, not a stable Rust-callable Xcode project API. The Xcode documentation does not define a universal command or structured input schema for a Rust tool to synthesize every extension point's complete metadata.

## Rust-callable runtime metadata read

The installed iOS 26.5 SDK has public Objective-C and C entry points:

- `NSBundle.infoDictionary` returns the bundle's dictionary constructed from its `Info.plist`; `objectForInfoDictionaryKey:` retrieves a value by key. Apple notes that `NSBundle` may add its own private keys, so a helper must select only known public plist keys.
- `CFBundleGetInfoDictionary` returns a bundle's global dictionary. `CFBundleGetValueForInfoDictionaryKey` is Apple's recommended lookup when a localized value is wanted; it may return a localized value. A point identifier is global metadata, so a focused extension helper should read it from the global dictionary.
- `NSBundle.bundleWithURL:` is available from iOS 4.0 for a caller-supplied bundle URL. `NSBundle.mainBundle` is also declared in the installed header. The `infoDictionary` and `CFBundle` declarations have no higher availability annotation in the inspected headers; do not invent a numeric floor for those declarations.

The local Cargo registry provides Rust binding options:

- `objc2-foundation` 0.3.2 has generated `NSBundle::infoDictionary` and `NSBundle::objectForInfoDictionaryKey` methods. The former returns `Option<Retained<NSDictionary<NSString, AnyObject>>>`; the latter returns `Option<Retained<AnyObject>>`. Relevant features are `NSBundle`, `NSDictionary`, and `NSString`; add `NSURL` to use `bundleWithURL:`.
- `core-foundation-sys` 0.8.7 declares `CFBundleGetInfoDictionary`, `CFBundleGetValueForInfoDictionaryKey`, and `CFBundleCopyInfoDictionaryInDirectory` as C functions. Its lower-level pointer API requires an ownership-safe wrapper before it belongs in a public crate surface.
- The root workspace already has `objc2-foundation` and `objc2-core-foundation` version 0.3.2 dependencies. A runtime reader using `objc2-foundation` needs no new binding release; a `core-foundation-sys` implementation would need a separate dependency/lock audit. A new package would still need root workspace membership and an owned lock refresh.

The narrow runtime operation should identify a bundle explicitly, read only documented keys, and copy scalar/string values into Rust-owned output. It should not load an extension executable, return Objective-C pointers, promise that the value is accurate for another bundle's runtime state, or infer that the extension is installed, enabled, approved, entitled, launchable, or compatible with a host.

## Boundaries and unsupported claims

- Legacy app-extension `Info.plist` data is a public property-list input. Its required point identifier and attributes depend on the specific extension point; a generic helper cannot validate every extension schema.
- ExtensionFoundation host-defined extension-point metadata uses Swift `AppExtensionPoint` declarations and compiler-generated `.appext` metadata. The audited D86 report found no Rust/C declaration or supported generic Rust input schema for that route. Do not present plist reads or writes as `.appext` support.
- App Intents `Metadata.appintents` is not an ordinary extension `Info.plist` dictionary. C7's no-go applies to that processor path only and is not repeated or broadened here.
- Metadata alone does not implement extension behavior, principal-class code, host/extension matching, signing, entitlements, distribution approval, process launch, XPC, or UI lifecycle.
- A read-only plist helper needs no new permission prompt or extension-specific entitlement merely to inspect a bundle dictionary. The extension target's own signing, entitlement, privacy, and App Store rules remain determined by its selected extension point and host.

## Feasibility result and next evidence

A metadata-only partial is feasible. For the row's current build-host classification, the most direct contract is a Rust-owned, typed input for a selected documented extension-point plist fragment, emitted as a normal property-list file for the host's Xcode target to process through `INFOPLIST_FILE`. It must not pretend that a fragment is a complete or valid extension without the extension-point-specific keys, target, principal implementation, signing, and host configuration.

The public `NSBundle`/`CFBundle` reader is an alternate small implementation seam if runtime inspection is preferred. It can snapshot `NSExtensionPointIdentifier` from a known bundle into Rust-owned text without loading extension code. If root selects this route, classify the row as a runtime Apple backend (`B`/platform-exclusive) instead of treating that reader as compiler metadata support. If neither precisely bounded operation is implemented, row 113 remains `X`.

Before changing row 113 from `X`, an implementation workstream should:

1. Define the exact bundle selection and output keys; use only documented `Info.plist` keys and copy results into Rust-owned values.
2. Add a focused `ios-extension-support` package using the existing workspace binding version and minimal feature set, without a Swift source or private metadata processor.
3. Establish the bundle-url behavior, missing/malformed key behavior, type validation, and the actual linked deployment floor from headers and target checks.
4. Verify the helper does not load extension code and document that it proves metadata only, not registration, approval, entitlement, runtime availability, or behavior.

## Evidence and checks

- Read-only environment checks: `xcode-select -p`, `xcodebuild -version`, and `xcrun --sdk iphoneos --show-sdk-path` identify Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5.
- Inspected `CoreFoundation.framework/Headers/CFBundle.h` declarations for `CFBundleGetInfoDictionary`, `CFBundleGetValueForInfoDictionaryKey`, and `CFBundleCopyInfoDictionaryInDirectory`.
- Inspected `Foundation.framework/Headers/NSBundle.h` declarations for `mainBundle`, `bundleWithURL:`, `infoDictionary`, and `objectForInfoDictionaryKey:`. The header marks `bundleWithURL:` available from iOS 4.0 and has no higher availability annotation on `infoDictionary`.
- Inspected local `objc2-foundation` 0.3.2 generated `NSBundle.rs`, `core-foundation-sys` 0.8.7 generated `bundle.rs`, root `Cargo.toml`, and the existing D86 ExtensionFoundation report.
- No source, manifest, lockfile, matrix, aggregate plan, index, or CI file was changed. No tests, builds, link probes, processor invocations, or runtime probes ran.
- Xcode 26.6 is below the repository's planned Xcode 27.x baseline; this report makes no Xcode 27 compatibility claim.

## Root integration status

- D96 originally added only this report and left row 113 `X`; B77 is now integrated and row 113 is `B`/partial for the runtime metadata reader only. `platform/ios/ios-extension-support/check.sh` passed host/device/Simulator compile, strict Clippy, rustdoc, feature-closure, and link/import gates; probes were not executed.
- If root later reopens the unimplemented build-host route, it needs a separate package/schema and a documented Xcode input contract; the runtime `ios-extension-support` package must not be repurposed for plist generation. No plist serializer dependency was selected in this feasibility pass.
- The runtime reader route uses public `NSBundle` through `objc2-foundation` 0.3.2, with API floor iOS 4.0 and link-probe minos 12.0/14.0. The canonical matrix, root lock, CI gate, and user-facing guide now record this bounded route.
- Do not modify row 110 or reuse this report as contrary evidence to C7's App Intents result.

## Apple references

- [Managing an app's information property list](https://developer.apple.com/documentation/bundleresources/managing-your-app-s-information-property-list)
- [Xcode build settings reference](https://developer.apple.com/documentation/xcode/build-settings-reference)
- [App Extension Keys](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/AppExtensionKeys.html)
- [`NSExtensionPointIdentifier`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsextension/nsextensionpointidentifier)
- [`NSBundle`](https://developer.apple.com/documentation/foundation/bundle?language=objc)
- [`Bundle.infoDictionary`](https://developer.apple.com/documentation/foundation/bundle/infodictionary)
- [`CFBundleGetInfoDictionary`](https://developer.apple.com/documentation/corefoundation/cfbundlegetinfodictionary%28_%3A%29?language=objc)
- [`CFBundleGetValueForInfoDictionaryKey`](https://developer.apple.com/documentation/corefoundation/cfbundlegetvalueforinfodictionarykey%28_%3A_%3A%29?language=objc)
- [Adding support for app extensions to your app](https://developer.apple.com/documentation/extensionfoundation/adding-support-for-app-extensions-to-your-app)
