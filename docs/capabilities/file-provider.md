# File Provider registered-domain presence

`ios-file-provider` provides one iOS-only, caller-requested snapshot: whether the calling app's own File Provider extension has one or more registered domains. It uses Apple's `NSFileProviderManager.getDomainsWithCompletionHandler` and exposes only a Boolean value

The native request starts when `request_registered_domain_presence()` is called. Its returned `Future` receives one result through per-request Rust state and the caller's `Waker`; no global executor, registry, or main-thread assumption is used. Dropping the future abandons Rust interest, but Apple's request has no cancellation path in this facade

Native failures remain `FileProviderQueryError::Native`, with the `NSError` domain and `NSInteger` code copied to owned Rust values. A native error never becomes `false`. The callback may run on an unspecified queue, so the adapter copies the array presence and error metadata inside the callback and retains no Objective-C object handle

The API floor is iOS 11.0. The query covers only registered domains for this app's own extension. An empty result does not mean FileProvider is unavailable; a non-empty result does not prove user enablement, network access, provider health, sync, or file access. The call also activates FileProvider's materialized/pending-set change notification posting for the process as documented by the SDK header

This capability does not enumerate other providers, expose domain IDs or labels, show picker UI, open files, obtain security-scoped URLs, coordinate file I/O, change domains, or implement provider lifecycle/sync. For user-mediated access to a document from another provider, use a document picker

See the [iOS adapter guide](../ios/fileprovider.md), [B75 plan](../../PLAN_IOS_FILEPROVIDER.md), and [G78 validation plan](../../PLAN_VALIDATION_IOS_FILEPROVIDER.md)
