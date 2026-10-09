# PLAN_IOS_EXTENSION_SUPPORT.md — B77: Row 113 runtime metadata slice

## Status and disposition

B77 implements one narrow runtime metadata reader for canonical row 113: read `NSExtension.NSExtensionPointIdentifier` from one explicit caller-supplied `.appex` path and return a Rust-owned UTF-8 `String`. Root integrated this as a platform-exclusive `B` partial for the runtime metadata surface, not a compiler/build-host plist generator; the matrix and global docs record the exact limited scope.

The reader uses public Foundation `NSBundle` and `NSDictionary` APIs through generated `objc2-foundation` 0.3.2 bindings. It validates the path shape, dictionary and string types, missing values, and the empty-string case. It never calls `NSBundle.load` or another extension-code loader.

## Exact scope

- Input is one caller-supplied absolute UTF-8 filesystem path whose last component has a non-empty name followed by exact `.appex` suffix; reject relative paths, NUL, and other suffixes.
- Build a file URL with `NSURL.fileURLWithPath:` and select only that bundle with `NSBundle.bundleWithURL:`. Do not enumerate installed bundles or search for a matching identifier.
- Read the bundle's global `infoDictionary`, then only `NSExtension`, then only `NSExtensionPointIdentifier`.
- Require `NSExtension` to be an `NSDictionary` and the point identifier to be an `NSString`; preserve missing and wrong-type cases as distinct errors, reject an empty identifier, and copy valid UTF-8 into Rust-owned output.
- Return only the identifier and bounded errors. Do not return Objective-C objects, property-list contents, principal classes, executable paths, extension attributes, or bundle identifiers.
- Add no portable contract, Swift code, App Intents support, `.appext` support, build-host plist generation, approval flow, entitlement assumption, or extension lifecycle behavior.

## API and platform limits

`NSBundle.bundleWithURL:` is available from iOS 4.0 in the installed public SDK header. `NSURL.fileURLWithPath:` has no later API annotation in that header, so the selected backend floor is iOS 4.0. `NSBundle.infoDictionary` has no higher availability annotation. The link script's device minos 12.0 and Simulator minos 14.0 are modern probe settings, not the API floor.

Apple documents `NSExtensionPointIdentifier` as a string property-list key that identifies the extension point. It lists possible values, but the required additional keys and attributes depend on the selected extension point. This API validates only the generic dictionary/string shape; it does not validate each point's schema.

`NSBundle.infoDictionary` is constructed from the bundle `Info.plist`, and Apple notes it can include private keys. This reader selects only the documented public `NSExtension.NSExtensionPointIdentifier` key. `NSBundle.bundleWithURL:` creates a bundle object for the supplied URL; loading executable code is a distinct API and is not called. Reading metadata does not establish that the path is a registered, signed, installed, enabled, entitled, approved, launchable, or host-compatible extension. No permission prompt or new extension-specific entitlement is required merely to inspect a caller-supplied bundle dictionary. The host must still have filesystem access to the supplied path; the selected extension's own host/signing/distribution rules remain outside scope.

The package is iOS-specific. It has no portable no_std contract, no Swift source, no build-host Xcode API, and no assertion about `.appext` compiler metadata. In particular, this runtime read does not contradict C7's App Intents processor blocker.

## Binding and ownership

The target dependency uses `objc2-foundation` 0.3.2 with defaults disabled and only `alloc`, `NSBundle`, `NSDictionary`, `NSString`, and `NSURL`. Generated bindings provide `NSBundle::bundleWithURL`, `NSBundle::infoDictionary`, `NSDictionary::objectForKey`, `NSURL::fileURLWithPath`, and `NSString::from_str`. `AnyObject::downcast_ref` verifies the two runtime object classes. `NSString::to_string` creates the returned Rust-owned string. No manual Objective-C ABI, raw selector, custom `extern` declaration, or unsafe block is used in this crate.

## Focused acceptance and validation

- [x] Add the scoped iOS package, API, README, and link-import probe under `platform/ios/ios-extension-support`.
- [x] Keep source free of manual unsafe code and avoid loading extension code.
- [x] Add a package-local non-test check for host, iOS device, and iOS Simulator compile/Clippy/docs, feature closure, framework imports, selectors, and deployment metadata.
- [x] Run `sh platform/ios/ios-extension-support/check.sh`: host, iOS device, iOS Simulator, strict Clippy, feature closure, rustdoc, and device/Simulator link/import checks pass; the probes were not executed. No tests were run.
- [x] Root-owned lock refresh includes the package record; the workspace `platform/ios/*` glob includes this crate.
- [x] Root reclassifies row 113 as a platform-exclusive runtime `B` partial, updates the matrix and aggregate docs, adds the CI gate, and publishes the user-facing guide.

The API floor is iOS 4.0; link-probe deployment values are iOS 12.0 device and iOS 14.0 Simulator. A passing compile/link audit cannot establish that an extension is installed or approved, that the metadata is valid for its extension point, or that it runs.

## Validation record

`sh platform/ios/ios-extension-support/check.sh` passed. It ran `cargo fmt --check`, host `cargo check --all-targets`, iOS device and Simulator library checks, strict Clippy on both Apple targets, the narrow binding feature audit, rustdoc, and both link/import probes. It ran no `cargo test` command and did not execute either linked probe.

The device link probe reports minos `12.0`; the Simulator probe reports minos `14.0`. Both inspected import sets are exactly `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`. The probes contain the expected Objective-C message send and Foundation selectors (`bundleWithURL:`, `infoDictionary`, `objectForKey:`, and `fileURLWithPath:`), with no Swift, ExtensionKit, or PlugInKit symbol imports.

## References

- Apple [`NSBundle`](https://developer.apple.com/documentation/foundation/bundle?language=objc): `bundleWithURL:` returns the bundle for a caller-supplied file URL; `infoDictionary` describes bundle `Info.plist` metadata.
- Apple [`Bundle.infoDictionary`](https://developer.apple.com/documentation/foundation/bundle/infodictionary): dictionary construction and additional private-key caveat.
- Apple [`NSExtensionPointIdentifier`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsextension/nsextensionpointidentifier): documented property-list type and extension-point identifier meaning.
- Installed iPhoneOS SDK 26.5: `Foundation.framework/Headers/NSBundle.h`, `NSURL.h`; `NSBundle.bundleWithURL:` is iOS 4.0, and the information-dictionary declarations have no higher availability annotation.
- Generated binding source: cached `objc2-foundation` 0.3.2 `src/generated/NSBundle.rs`, `NSDictionary.rs`, `NSURL.rs`, and `NSString.rs`.

## Root-owned integration

The workspace's `platform/ios/*` glob includes `ios-extension-support`; the refreshed root lockfile contains its package record with `objc2` and `objc2-foundation` dependencies. Root added the package gate to CI, reconciled row 113 as a runtime `B` partial, and updated aggregate docs and indexes. This workstream does not edit those shared files.
