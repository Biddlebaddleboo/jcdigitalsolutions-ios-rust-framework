# PLAN_IOS_FILEPROVIDER.md — B75: File Provider domain-presence snapshot

## Objective

Implement a narrow iOS Rust facade for a caller-requested async snapshot of whether the calling app's own File Provider extension has registered domains

## Status

The package and source are implemented in `platform/ios/ios-file-provider`; the root workspace and lock include `ios-file-provider` and `objc2-file-provider` 0.3.2. The G78 gate passes: format, host/device/Simulator checks, strict device/Simulator Clippy, rustdoc, static scope checks, zero-Swift scan, device/Simulator link/import audit, and `cargo xtask docs-check`. The link probe initially failed its `must_use` check; `examples/link_probe.rs` now explicitly binds the probe future, and the full gate then passed. No tests or runtime probes are in scope

## Scope

The only native operation is `+[NSFileProviderManager getDomainsWithCompletionHandler:]`, available from iOS 11.0 through the public Objective-C FileProvider framework. It is callable from the main app that contains the provider extension and reports that extension's registered domains

The original B75 API is `request_registered_domain_presence() -> RegisteredDomainPresenceFuture`. The request starts at function call time and returns one Boolean snapshot. B82 adds `request_registered_domain_count() -> RegisteredDomainCountFuture`, which returns the same callback array length as a fixed-width `u64` and exposes no domain IDs or metadata. `FileProviderQueryError::Native` preserves the native `NSError` domain and `NSInteger` code as owned Rust values; API-floor and non-iOS cases use distinct errors

Both futures use per-request `Arc<Completion>` state protected by a mutex. It holds the caller's `Waker` only until completion or drop. The native callback captures only this per-request state, copies the array count and error domain/code during the callback, and returns no Objective-C object across threads. The B75 API derives presence from the copied count; B82 returns the count. The callback may arrive on an unspecified queue. No main-thread context, global registry, crate-owned executor, or native object handle is required; the caller must poll the returned future through its executor

Dropping the future detaches Rust interest and releases the stored waker/result; the native query is not cancelled. The callback state remains alive until the callback returns. The future reports `Pending` after its first `Ready`, and callback completion is one-shot

## Out of scope

- Listing or returning domain identifiers, display names, account/location names, URLs, or domain handles
- Detecting whether a provider is enabled, reachable, online, healthy, or syncing
- Enumerating File Provider extensions from other apps or all system providers
- Document picker or browser UI, URL translation, file access, security-scoped resources, or file coordination
- Domain creation/removal, provider extension implementation, remote storage, synchronization, item enumeration, or file contents
- Any claim that a presence or count snapshot predicts later file access or service success

## API, binding, and host facts

- Installed SDK: Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5
- Public framework: `FileProvider.framework`; Objective-C class `NSFileProviderManager`
- API floor: iOS 11.0, from `FILEPROVIDER_API_AVAILABILITY_V2_V3` on the manager and domain declarations
- Binding: upstream `objc2-file-provider` 0.3.2, defaults off; selected features `Extension`, `NSFileProviderDomain`, and `block2`. `Extension` enables Foundation `NSArray` and `NSError` requirements. The package directly enables `NSArray`, `NSError`, and `NSString` on `objc2-foundation`
- The callback has no documented main-thread or actor guarantee. The future is thread-safe Rust state and must be polled only through the caller's chosen executor
- The query itself has no usage-description key, prompt, or general entitlement listed in the reviewed API docs. A provider extension is separate host configuration
- Legacy nonreplicated provider setup uses an extension target and Info.plist keys `NSExtensionFileProviderDocumentGroup`, `NSExtensionPointIdentifier` (`com.apple.fileprovider-nonui`), and `NSExtensionPrincipalClass`, plus the App Groups capability and `com.apple.security.application-groups` for its shared group
- The replicated extension path is iOS 16.0+ and requires `NSFileProviderReplicatedExtension`, `NSFileProviderEnumerating`, item/version implementation, and a sync lifecycle; none is implemented here
- Calling `getDomainsWithCompletionHandler:` causes materialized-set and pending-set change notifications to begin posting for the process, per `NSFileProviderManager.h`

## Validation boundary

G78 uses compile, strict Clippy, rustdoc, static-surface, zero-Swift, and link/import checks only. It does not run tests, query a provider, register a domain, present a picker, access a file, exercise an entitlement, or execute a link probe. Exact commands and status belong in [the G78 plan](PLAN_VALIDATION_IOS_FILEPROVIDER.md)

## Sources

- [File Provider overview](https://developer.apple.com/documentation/fileprovider)
- [`NSFileProviderManager`](https://developer.apple.com/documentation/fileprovider/nsfileprovidermanager)
- [`getDomainsWithCompletionHandler:`](https://developer.apple.com/documentation/fileprovider/nsfileprovidermanager/getdomainswithcompletionhandler%28_%3A%29?language=objc)
- [Nonreplicated File Provider extension](https://developer.apple.com/documentation/fileprovider/nonreplicated-file-provider-extension)
- [Replicated File Provider extension](https://developer.apple.com/documentation/fileprovider/replicated-file-provider-extension)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)
- [`objc2-file-provider` 0.3.2](https://docs.rs/objc2-file-provider/0.3.2/objc2_file_provider/)

## Audit record

The B75 Boolean API remains intact; additive B82 scope is documented in [the count plan](PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md). Source scope is limited to the manager query, per-request Rust completion state, and owned count/error data. The package is locked and the G78 compile, Clippy, rustdoc, and link/import gates pass. The link probes were built and inspected, never executed. No tests, provider requests, or runtime probes were run
