# `ios-thread-network`

This iOS-only package implements `request_preferred_network_availability()` using
`THClient.isPreferredNetworkAvailableWithCompletion:`. It returns only whether the framework
reports a preferred Thread network as available. Apple documents this as a pre-consent check before
retrieving preferred-network credentials; this query does not retrieve credentials or show that
the device has Thread radio support, joined-network state, or border-router capability.

The host app must carry Apple's `com.apple.developer.networking.manage-thread-network-credentials`
entitlement, including the distribution access Apple grants after approval. The package does not
request credentials, change credentials, start a Thread network, or claim a general Thread support
status. API floor: iOS 16.4.

`objc2-thread-network` 0.3.2 is used with default features disabled and only `THClient`, `block2`,
and `std` enabled. No Swift source or handwritten Objective-C ABI is used. The package gate
`sh platform/ios/ios-thread-network/check.sh` performs static Rust checks only; it runs no tests or
link probes.

See the [Thread capability audit](../../../PLAN_CAPABILITIES_THREAD.md) and [iOS guide](../../../docs/ios/thread-network.md).
