# File Provider registered-domain snapshots

`ios-file-provider` provides iOS-only, caller-requested snapshots of the calling app's own File Provider extension's registered domains. The B75 presence API reports only whether the returned domain array is non-empty. The additive B82 API returns the array length as a `u64`. Both use Apple's `NSFileProviderManager.getDomainsWithCompletionHandler`; neither exposes identifiers or other domain metadata

Each native request starts when its `request_registered_domain_presence()` or `request_registered_domain_count()` function is called. The returned `Future` receives one result through per-request Rust state and the caller's `Waker`; no global executor, registry, or main-thread assumption is used. Dropping the future abandons Rust interest, but Apple's request has no cancellation path in this facade

Native failures remain `FileProviderQueryError::Native`, with the `NSError` domain and `NSInteger` code copied to owned Rust values. A native error never becomes `false` or a zero count. The callback may run on an unspecified queue, so the adapter copies only the array count and error metadata inside the callback and retains no Objective-C object handle

The API floor is iOS 11.0. The query covers only registered domains for this app's own extension. An empty result does not mean FileProvider is unavailable; a non-empty result does not prove user enablement, network access, provider health, sync, or file access. The call also activates FileProvider's materialized/pending-set change notification posting for the process as documented by the SDK header

This capability does not enumerate other providers, expose domain IDs or labels, show picker UI, open files, obtain security-scoped URLs, coordinate file I/O, change domains, or implement provider lifecycle/sync. For user-mediated access to a document from another provider, use a document picker

See the [iOS adapter guide](../ios/fileprovider.md), [B75 presence plan](../../PLAN_IOS_FILEPROVIDER.md), [B82 count plan](../../PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md), and [G78 validation plan](../../PLAN_VALIDATION_IOS_FILEPROVIDER.md)
