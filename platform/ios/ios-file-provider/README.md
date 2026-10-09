# `ios-file-provider`

This package exposes one async snapshot: whether the calling app's own File Provider extension has one or more registered domains. The request begins when `request_registered_domain_presence()` runs. Its returned future uses only per-request Rust state and the caller's executor waker; it adds no global executor, registry, or main-thread assumption

The result contains only a Boolean. A native error remains a `FileProviderQueryError::Native` with its `NSError` domain and `NSInteger` code copied into owned Rust data. It is not converted to `false`. Dropping the future abandons Rust interest but does not cancel Apple's request

This package does not enumerate providers from other apps, expose domain IDs or labels, present a document picker, access files or security-scoped URLs, coordinate file reads/writes, register or remove domains, or implement a provider extension. A positive result does not mean that a domain is enabled, reachable, online, healthy, or syncing

The API floor is iOS 11.0. The host app must include its own File Provider extension for a meaningful positive result. See the [capability guide](../../../docs/capabilities/file-provider.md), [iOS guide](../../../docs/ios/fileprovider.md), [B75 plan](../../../PLAN_IOS_FILEPROVIDER.md), and [G78 validation plan](../../../PLAN_VALIDATION_IOS_FILEPROVIDER.md)

Run `sh platform/ios/ios-file-provider/check.sh` after root lock integration. The gate performs compile, Clippy, rustdoc, source/feature, zero-Swift, and link/import checks only; it does not run tests or execute the link probes
