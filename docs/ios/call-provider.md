# iOS CallKit provider and actions

The iOS-only `ios-call-provider` package owns one `CXProvider`. Its Rust API reports incoming calls and ended calls with a fixed-width `CallId([u8; 16])`, then delivers system answer/end actions through a synchronous Rust callback. Native `NSUUID`, `CXProvider`, and `CXAction` objects do not cross the public Rust API.

Create the provider on the main thread and retain it for the service lifetime. Call `invalidate()` before release; `Drop` is only a safety fallback. CallKit stores the provider delegate and specified queue weakly, so `CallProvider` retains both the Rust delegate and the main dispatch queue. Delegate methods use that queue. Incoming-report completion uses a synchronized one-shot handoff and always submits state access and the caller callback asynchronously to the main queue, regardless of the queue used for the native block. It never blocks or touches `MainThreadOnly` state off-main. `providerDidReset` clears tracked call state and sends `ProviderReset` to the callback. Reset does not permanently invalidate the provider.

Incoming-call reporting completes asynchronously on the main delegate queue. A non-null CallKit error is reported as `CallProviderError::SystemRejected`; the app must not proceed with that call. The provider tracks report-pending and active calls so `end_call` during report submission is deferred until report acceptance. A failed report removes the identity without issuing an ended report. App-specific pre-submit identity/ACK checks remain caller-owned and must run before `report_incoming`.

Answer/end action callbacks require an immediate `ActionDecision::Fulfill` or `ActionDecision::Fail`. They must return promptly and must not wait for network work. Unknown or stale call identities fail. Actions are not retained; the backend checks CallKit completion state after the handler and does not complete a timed-out action again. `ProviderReset` and `ActionTimedOut` are lifecycle events; their handler return value is ignored. Rust panics do not cross the Objective-C delegate boundary.

The provider configuration uses public CallKit APIs. On iOS 10–13 it uses deprecated-but-supported `CXProviderConfiguration.initWithLocalizedName:`; on iOS 14 and later it uses `init()`. An explicit Recents setting is available from iOS 11; `None` preserves the iOS 10 system default, while an explicit setting requested on iOS 10 returns `UnavailableOption`. The package preserves an iOS 10 core API floor and does not inherit the Rideshare app's iOS 18 floor.

CallKit provider reporting has no permission prompt or usage-description key. This package adds no entitlements or background modes. It does not implement PushKit/APNs delivery, VoIP token lifecycle, audio, microphone access, `CXCallController`, call-directory extensions, or the default-calling-app role. Apps that need a bounded background lease around report completion can use the separate [`ios-background-execution`](background-execution.md) capability; this provider does not own `UIApplication`.

Run the focused registered capability profile on a macOS host with the required Apple SDKs and
inspection tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-call-provider
```

The profile supplies host tests, target compilation, and link/import evidence. Native compile/link
results do not prove that CallKit accepts a report, presents system UI, executes an action, or
meets a deadline on Simulator or device.
