# `ios-file-provider`

This package exposes async snapshots of the calling app's own File Provider extension's registered domains. `request_registered_domain_presence()` returns only whether the domain array is non-empty; `request_registered_domain_count()` returns its `u64` length. Each request begins when its function runs. The returned future uses only per-request Rust state and the caller's executor waker; it adds no global executor, registry, or main-thread assumption

The snapshots contain only a Boolean or fixed-width count. A native error remains a `FileProviderQueryError::Native` with its `NSError` domain and `NSInteger` code copied into owned Rust data. It is not converted to `false`. Dropping a future abandons Rust interest but does not cancel Apple's request

This package does not enumerate providers from other apps, expose domain IDs or labels, present a document picker, access files or security-scoped URLs, coordinate file reads/writes, register or remove domains, or implement a provider extension. A positive presence or nonzero count does not mean that a domain is enabled, reachable, online, healthy, or syncing

The API floor is iOS 11.0. The host app must include its own File Provider extension for a meaningful positive result. See the [capability guide](../../../docs/capabilities/file-provider.md), [iOS guide](../../../docs/ios/fileprovider.md), [B75 presence plan](../../../PLAN_IOS_FILEPROVIDER.md), [B82 count plan](../../../PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md), and [G78 validation plan](../../../PLAN_VALIDATION_IOS_FILEPROVIDER.md)

Run `sh platform/ios/ios-file-provider/check.sh` after root lock integration. The gate performs compile, Clippy, rustdoc, source/feature, zero-Swift, and link/import checks only; it does not run tests or execute the link probes
