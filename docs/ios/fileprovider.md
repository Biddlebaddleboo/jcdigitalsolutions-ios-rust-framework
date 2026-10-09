# iOS File Provider registered-domain snapshots

`ios-file-provider` wraps the iOS 11.0+ Objective-C class method `NSFileProviderManager.getDomainsWithCompletionHandler`. Apple permits this manager query from the main app that contains the File Provider extension. It returns the domains registered for that app's own provider, not other installed providers

## Request and result

```rust
async fn query() -> Result<bool, ios_file_provider::FileProviderQueryError> {
    let snapshot = ios_file_provider::request_registered_domain_presence().await?;
    Ok(snapshot.has_registered_domains())
}
```

The function starts the native query at call time and returns a one-shot `Future`. The future stores its own completion state and the executor `Waker` supplied during `poll`; there is no process-global registry or crate-owned executor, but the caller must poll the future using an executor. The framework callback queue is unspecified, so callers must not assume main-thread delivery. A caller that needs UIKit work must marshal it to its own main-thread executor

`RegisteredDomainPresence` reports only whether the returned domain array is non-empty. `FileProviderQueryError::Native` preserves `NSError.domain` and `NSError.code` as owned Rust data; it does not flatten an OS error to `false`. No localized description or `userInfo` dictionary is copied

Call `request_registered_domain_count()` when the caller needs the exact number rather than a
Boolean. Its `RegisteredDomainCount::count()` is a `u64` copied from the callback array length. Each
presence or count function call starts its own asynchronous request; neither result contains domain
identifiers, display names, account/location names, URLs, or provider objects. See the focused
[B82 domain-count plan](../../PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md)

If the future is dropped before callback completion, Rust interest and its `Waker` are released. The native query is not cancelled; the callback keeps only per-request Rust completion state until it returns. No FileProvider object or domain identifier escapes the callback

## Host setup and limits

The query itself has no usage-description key, user permission prompt, or general FileProvider entitlement in the reviewed public API. A meaningful result requires a File Provider extension belonging to the app. For a legacy nonreplicated provider, Apple specifies an extension target with `NSExtensionFileProviderDocumentGroup`, `NSExtensionPointIdentifier` `com.apple.fileprovider-nonui`, and `NSExtensionPrincipalClass`; the shared group uses the App Groups capability and `com.apple.security.application-groups` entitlement

An iOS 16+ replicated extension is a different and much larger boundary: it adopts `NSFileProviderReplicatedExtension` and `NSFileProviderEnumerating`, supplies item/version behavior, and participates in system-managed local/remote synchronization. This adapter implements none of those interfaces

`getDomainsWithCompletionHandler` means only that the app's provider has a domain registration at callback time. It does not report whether a person enabled the provider in Files, whether the server is online, whether items are synced/materialized, or whether a future document operation succeeds. Apple's SDK header also documents that materialized-set and pending-set change notifications begin posting only after the query is called

This API never returns domain identifiers, account/location names, URLs, items, file data, or native handles. It does not query or access another provider. Use a document picker for user-mediated document selection, and the separate file-coordination APIs for supported coordinated access after a URL is provided

See [the portable capability scope](../capabilities/file-provider.md), [B75](../../PLAN_IOS_FILEPROVIDER.md), and [G78](../../PLAN_VALIDATION_IOS_FILEPROVIDER.md)
