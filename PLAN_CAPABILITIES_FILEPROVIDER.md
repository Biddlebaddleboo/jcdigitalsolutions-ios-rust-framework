# PLAN_CAPABILITIES_FILEPROVIDER.md — D85/B75/B82: FileProvider row 099 scope

## Scope

Audit row `099-extension-entitlement-capabilities-fileprovider` for a bounded iOS consumer-app API, direct Rust binding support, API floor, host setup, and extension-only lifecycle limits; B75 implements domain presence and B82 adds the returned registered-domain count

This scoped plan does not change the canonical matrix, root aggregate plans/indexes, CI, Cargo root manifests, or source owned by another workstream

## Status and recommendation

The root workspace and lock include the separate `ios-file-provider` package and `objc2-file-provider` 0.3.2; the B75 and B82 gates pass. The canonical row is `B`/partial; B82 adds no row or matrix count. Neither slice adds `framework-documents::ios` or represents general FileProvider support

A narrow iOS consumer-app slice is implemented in `platform/ios/ios-file-provider`: the original B75 API queries whether the calling app's own File Provider extension has any registered domains through `+[NSFileProviderManager getDomainsWithCompletionHandler:]`, and B82 adds a fixed-width count of that returned array. Apple says the manager is available to the main app that contains its provider extension, and the method returns that extension's registered domains. This is not a query of arbitrary providers installed on the device, Files app state, file access, or sync health

The query has a callback-based Objective-C API and a generated Rust binding path through `objc2-file-provider` 0.3.2. It needs no Swift ABI. No FileProvider permission prompt or general entitlement for this read query appears in the public declaration or Apple API docs reviewed here. The binding's completion callback has no documented queue or actor guarantee in the inspected header; the adapter accepts callback delivery without assuming main-thread context

The APIs return presence or a count only, not domain IDs, `displayName`, account/location labels, URLs, or provider data. A non-empty result or positive count means the caller's provider has one or more registered domains at query time; it does not mean a domain is enabled, reachable, online, healthy, authorized for every operation, or actively syncing. An empty array means no registered domains for that provider, not that FileProvider is unavailable on the device

## SDK and binding evidence

Inspection used Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5

- Framework path: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/FileProvider.framework`
- Public module map: `FileProvider.framework/Modules/module.modulemap`; public umbrella header: `FileProvider.framework/Headers/FileProvider.h`
- `NSFileProviderManager.h` declares `NSFileProviderManager` as a public Objective-C class. Its class comment explicitly permits use from the extension, the main app containing the extension, and sibling extensions
- The same header declares `+getDomainsWithCompletionHandler:` as returning `NSArray<NSFileProviderDomain *> *domains` plus nullable `NSError *error`; the method returns the current provider's registered domain list
- `NSFileProviderDefines.h` maps `FILEPROVIDER_API_AVAILABILITY_V2_V3` to iOS 11.0, and `NSFileProviderManager` uses that macro. `NSFileProviderDomain` uses the same macro. Thus the minimum API floor for this candidate is iOS 11.0
- FileProvider headers mark the framework unavailable on Mac Catalyst, watchOS, and tvOS. The target here is iOS; the iOS 11 floor does not imply support for every FileProvider API on that floor
- Apple documents `NSFileProviderReplicatedExtension` for iOS 16.0+. This is a distinct extension implementation model, not a prerequisite for the iOS 11 manager query
- The cached `objc2` 0.6.5 generated-framework catalog lists `objc2-file-provider`. Upstream generated binding docs identify crate version 0.3.2 and list `NSFileProviderManager` under feature `Extension` and `NSFileProviderDomain` under feature `NSFileProviderDomain`
- The generated 0.3.2 crate feature source shows `Extension` enables the Foundation array, error, string, URL, and related types used by the manager, while `NSFileProviderDomain` enables the domain class dependencies. `block2` is the callback block feature. The new package disables default features and selects `Extension`, `NSFileProviderDomain`, and `block2`, plus direct `objc2-foundation` features `NSArray`, `NSError`, and `NSString`
- Root Cargo.lock integration now includes package records for `ios-file-provider` and `objc2-file-provider` 0.3.2
- The installed SDK header notes that `NSFileProviderMaterializedSetDidChange` and `NSFileProviderPendingSetDidChange` notifications begin posting only after `getDomainsWithCompletionHandler:` is called. The call does not mutate domains, but it can activate these notification streams in the process

## Host setup, entitlement, and lifecycle boundary

- For an app that only queries its own extension's domains, the reviewed `getDomainsWithCompletionHandler:` declaration requires no usage-description key or authorization prompt. No general FileProvider entitlement for this query is listed in the inspected public headers or Apple API docs
- A provider implementation is a separate host-project feature. Apple describes FileProvider as an extension through which other apps access documents managed and synced by the containing app; local document sharing instead uses `UIDocumentBrowserViewController` or document-sharing Info.plist keys
- The legacy nonreplicated iOS provider requires a separate extension target and extension metadata. Its `NSExtension` dictionary uses `NSExtensionFileProviderDocumentGroup`, `NSExtensionPointIdentifier` `com.apple.fileprovider-nonui`, and `NSExtensionPrincipalClass`. The shared document group uses the App Groups capability and `com.apple.security.application-groups` entitlement
- The iOS 16+ replicated extension must adopt `NSFileProviderReplicatedExtension` and `NSFileProviderEnumerating`, enumerate remote items, and implement item/version behavior. It owns remote/local synchronization and system callbacks. It is not a small status query or a generic file consumer API
- `com.apple.developer.fileprovider.testing-mode` is a testing-only entitlement for non-empty domain testing modes; it is not a general FileProvider use entitlement and is not needed for the read query
- The related-process rule matters: a consumer app can query its own extension, but this API does not enumerate other developers' provider extensions. Use the document picker for user-mediated access to documents from other providers
- No thread or actor restriction for `getDomainsWithCompletionHandler:` is stated in the inspected Objective-C header or method docs. Do not promise a callback queue; do not use UI or main-thread assumptions in this status-only slice

## Candidate contract and limits

The original B75 slice is a caller-requested asynchronous presence snapshot; additive B82 adds the
count without changing its ownership or lifecycle boundary:

- `request_registered_domain_presence()` starts `getDomainsWithCompletionHandler:` and returns a per-request future
- `RegisteredDomainPresence` carries only whether the returned array is non-empty
- `request_registered_domain_count()` starts the same query; `RegisteredDomainCount` carries the returned array length as `u64`
- `FileProviderQueryError::Native` preserves `NSError` domain and `NSInteger` code; native errors do not map to `false` or zero
- the result describes only the caller's own registered domains at callback time
- no domain identifiers, labels, account names, URLs, file contents, or native domain handles cross the callback boundary
- no domain registration/removal, browsing, document-picker UI, file coordination, security-scoped URLs, file reads/writes, or sync methods are in the contract
- no global registry or crate-owned executor is required; callers must poll the returned future using their executor

The read method is a direct Rust-callable Objective-C class method, so extension-only behavior is not a blocker for this minimal query. It is implemented as an isolated package. B75 compile/Clippy/rustdoc/import gates pass after root lock integration; the link probes were built and inspected, not executed

## Acceptance boundary

This audit establishes only:

- a public iOS 11+ Objective-C manager query callable from the main app that contains its File Provider extension
- an upstream generated Rust binding crate, `objc2-file-provider` 0.3.2, with the manager/domain/block feature path described above
- the Boolean presence snapshot and additive fixed-width count snapshot for that app's own extension's registered domains
- separate extension-host setup for actual provider service, including App Group and Info.plist requirements for the legacy extension path
- no evidence that a domain count or presence value proves provider availability, enablement, network access, sync completion, or general FileProvider support

The D85 audit led to B75 source, package, guides, and gate scripts; B82 adds the count snapshot in the same package. The focused G78/B82 package gate passes on host, iOS device, and arm64 Simulator; link/import checks confirm iOS 11.0 device and iOS 14.0 Simulator minos plus the expected four imports. No test, provider query, domain registration, document-picker, entitlement, or runtime probe was run. Row 099 remains `B`/partial in the root-owned matrix; B82 changes no matrix count

## Apple and binding references

- [File Provider overview](https://developer.apple.com/documentation/fileprovider)
- [NSFileProviderManager](https://developer.apple.com/documentation/fileprovider/nsfileprovidermanager)
- [`getDomainsWithCompletionHandler:`](https://developer.apple.com/documentation/fileprovider/nsfileprovidermanager/getdomainswithcompletionhandler%28_%3A%29?language=objc)
- [NSFileProviderDomain](https://developer.apple.com/documentation/fileprovider/nsfileproviderdomain)
- [Nonreplicated File Provider extension](https://developer.apple.com/documentation/fileprovider/nonreplicated-file-provider-extension)
- [Replicated File Provider extension](https://developer.apple.com/documentation/fileprovider/replicated-file-provider-extension)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)
- [File Provider testing modes entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.fileprovider.testing-mode)
- [App Extension Programming Guide: Document Provider](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/FileProvider.html)
- [`objc2-file-provider` 0.3.2 generated bindings](https://docs.rs/objc2-file-provider/0.3.2/objc2_file_provider/)
- [`objc2-file-provider` 0.3.2 feature source](https://docs.rs/crate/objc2-file-provider/0.3.2/source/Cargo.toml.orig)
- Implementation: `platform/ios/ios-file-provider`
- Guides: `docs/capabilities/file-provider.md`, `docs/ios/fileprovider.md`
- Scoped plans: `PLAN_IOS_FILEPROVIDER.md` (B75), `PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md` (B82), `PLAN_VALIDATION_IOS_FILEPROVIDER.md` (G78)
- Current row record: `docs/capabilities/capability-status.json`, id `099-extension-entitlement-capabilities-fileprovider`

## Audit record

Inspected the installed FileProvider module map and public headers, Xcode/iPhoneOS SDK versions, cached generated-binding catalog, current Cargo manifests and lock, row 099 status record, upstream `objc2-file-provider` 0.3.2 docs and feature source, and Apple FileProvider, extension, App Groups, and testing-entitlement docs; then added the scoped B75 package, Rust callback/future, focused gate scripts, and guides. B82 adds an array-count snapshot by reusing the existing selector and callback

This workstream did not edit the root manifest, lock, global index, CI, or matrix. The assigned G78 gate built and inspected link probes but did not execute them; no tests or runtime probes ran
