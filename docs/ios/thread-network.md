# iOS preferred Thread network availability

`ios-thread-network::request_preferred_network_availability()` starts
`THClient.isPreferredNetworkAvailableWithCompletion:` and returns
`PreferredThreadNetworkAvailabilityFuture`. The future's `Output` is one Boolean from
`THClient`; it has no fabricated error channel because the public completion supplies only a
Boolean. The request starts when the function runs. The callback queue is not documented, so the
implementation shares only synchronized Rust state with the callback and retains the non-`Send`
`THClient` on the caller's thread.

The result reports only whether the ThreadNetwork framework says a preferred network is available.
Apple describes this as a check before retrieving preferred-network credentials, which requires
person consent. The query does not retrieve, inspect, store, or delete credentials; it does not
prove Thread radio support, active connectivity, a joined network, border-router status, Matter
support, or general Thread readiness. It does not present consent UI.

The host app must carry
`com.apple.developer.networking.manage-thread-network-credentials`. Apple requires distribution
access for publication after its entitlement approval and required conformance work. A successful
compile does not establish entitlement or distribution approval. The API floor is iOS 16.4.

The binding is `objc2-thread-network` 0.3.2 with default features disabled and only `THClient`,
`block2`, and `std` enabled. `sh platform/ios/ios-thread-network/check.sh` compiles and documents
the package but does not run tests, link probes, the query, or a device/Simulator operation.

See the [Thread feasibility and partial-scope plan](../../PLAN_CAPABILITIES_THREAD.md) and
[`ios-thread-network`](../../platform/ios/ios-thread-network/README.md).
