# iOS CallKit provider and actions

The iOS-only `ios-call-provider` package owns one `CXProvider`. Its Rust API reports incoming calls and ended calls with a fixed-width `CallId([u8; 16])`, submits outgoing start transactions, and delivers start, answer, end, mute-state, and hold-state requests through either a synchronous callback or an opt-in deferred completion. Native `NSUUID`, `CXProvider`, `CXCallController`, and `CXAction` objects do not cross the public Rust API.

Create the provider on the main thread and retain it for the service lifetime. Call `invalidate()` before release; `Drop` is only a safety fallback. CallKit stores the provider delegate and specified queue weakly, so `CallProvider` retains both the Rust delegate and the main dispatch queue. Delegate methods use that queue. Incoming-report completion uses a synchronized one-shot handoff and always submits state access and the caller callback asynchronously to the main queue, regardless of the queue used for the native block. It never blocks or touches `MainThreadOnly` state off-main. `providerDidReset` clears tracked call state and sends `ProviderReset` to the callback. Reset does not permanently invalidate the provider.

Incoming-call reporting completes asynchronously on the main delegate queue. A non-null CallKit error is reported as `CallProviderError::SystemRejected`; the app must not proceed with that call. The provider tracks report-pending and active calls so `end_call` during report submission is deferred until report acceptance. A failed report removes the identity without issuing an ended report. App-specific pre-submit identity/ACK checks remain caller-owned and must run before `report_incoming`.

`CallProvider::request_outgoing` submits one public `CXStartCallAction` through `CXCallController`.
`OutgoingCall` carries the framework `CallId`, handle type/value, and optional video request; values
are checked against the provider's supported handles and video configuration before submission.
The asynchronous completion reports only whether CallKit accepted the transaction. A successful
result does not mean that signaling was sent, a remote party answered, or media connected.

CallKit separately sends `CallProviderEvent::StartRequested` or `CallActionKind::Start` through the
configured action handler. The app owns signaling and chooses whether to fulfill that action. After
successful fulfillment, `report_outgoing_started_connecting` and then
`report_outgoing_connected` submit the app's public CallKit progress reports; out-of-order calls
return `CallProviderError::InvalidState`. Unique per-request tokens prevent a late rejected
transaction from removing a newer outgoing call that reused the same `CallId`. The API does not
implement PushKit/APNs, destination identity/ACK policy, audio, or media transport.

`CallProvider::new` preserves the immediate callback contract: call-action callbacks return `ActionDecision::Fulfill` or `ActionDecision::Fail`, must return promptly, and must not wait for network work. `AnswerRequested` and `EndRequested` describe answer and end actions. `MuteRequested { call_id, is_muted }` and `HoldRequested { call_id, is_held }` expose the state requested by `CXSetMutedCallAction` and `CXSetHeldCallAction`. Unknown or stale call identities fail. This path does not retain actions and does not complete a timed-out action again. `ProviderReset` and `ActionTimedOut` remain lifecycle events whose return value is ignored.

`CallProvider::new_with_deferred_actions` takes separate lifecycle and action handlers. The action callback receives a `CallActionRequest` on the main CallKit delegate queue and must return promptly. Its `kind()` identifies answer/end or carries the requested mute/hold state. It may move the request or its `CallActionCompletion` to app-owned worker or future work. `complete(ActionDecision)` submits a one-shot result for asynchronous main-queue resolution; `Ok(())` means queued, not that CallKit has accepted or executed the action. Dropping an unsubmitted completion queues `ActionDecision::Fail`. The native `CXCallAction` remains retained only in a main-thread-bound provider registry. CallKit timeout expires the token without re-completing the system-completed action. Provider reset, `invalidate()`, provider drop, and an ended call invalidate pending tokens; reset/invalidation return `ProviderUnavailable`, while timeout/call end return `Expired`. An end action removes its active call from the local registry once. Panics do not cross the Objective-C delegate boundary.

The mute and hold events report the state CallKit requests; they do not configure `AVAudioSession` or change an application's media transport. Fulfilling the action declares that the app handled the requested state. Audio-session ownership and media mute/hold behavior remain app-owned.

With the default `audio-session-events` feature, CallKit's public provider delegate activation and deactivation callbacks deliver `AudioSessionActivated` and `AudioSessionDeactivated` lifecycle events. They report CallKit's session handoff only: the event carries no `AVAudioSession` object and performs no session configuration, activation, or media I/O. Disable the feature with `--no-default-features` to omit the optional `objc2-avf-audio` dependency.

The provider configuration uses public CallKit APIs. On iOS 10–13 it uses deprecated-but-supported `CXProviderConfiguration.initWithLocalizedName:`; on iOS 14 and later it uses `init()`. An explicit Recents setting is available from iOS 11; `None` preserves the iOS 10 system default, while an explicit setting requested on iOS 10 returns `UnavailableOption`. The package preserves an iOS 10 core API floor and does not inherit the Rideshare app's iOS 18 floor.

CallKit provider and outgoing transaction requests have no permission prompt or usage-description key. This package adds no entitlements or background modes. It does not implement PushKit/APNs delivery, VoIP token lifecycle, audio, microphone access, call-directory extensions, or the default-calling-app role. Apps that need a bounded background lease around report completion can use the separate [`ios-background-execution`](background-execution.md) capability; this provider does not own `UIApplication`.

Run the focused registered capability profile on a macOS host with the required Apple SDKs and
inspection tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-call-provider
```

The profile supplies host tests, target compilation, and link/import evidence. Native compile/link
results do not prove that CallKit accepts a report, presents system UI, executes an action, or
meets a deadline on Simulator or device.
