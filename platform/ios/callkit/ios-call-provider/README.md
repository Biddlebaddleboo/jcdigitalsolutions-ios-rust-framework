# ios-call-provider

`ios-call-provider` owns one public CallKit `CXProvider` and exposes Rust call IDs, incoming-call reports, call-end reports, and answer/end/mute/hold action decisions. Its `objc2-call-kit` dependency is iOS-target-only, pinned at 0.3.2, and uses default features off.

The provider and its weakly stored delegate are main-thread-bound. Keep `CallProvider` alive for its service lifetime and call `invalidate()` before release. The provider retains its explicitly selected main dispatch queue because CallKit stores that queue weakly. Incoming-report completion uses a synchronized one-shot handoff and is always marshaled asynchronously to the main queue before it touches provider state or invokes the caller, regardless of the queue used by CallKit for the native block. A system-rejected report is an error; the caller must not proceed with the call.

`CallProvider::new` preserves its synchronous call-action handler contract; that handler must return promptly and must not wait for network work. Events include answer/end requests and requested mute/hold states. `CallProvider::new_with_deferred_actions` adds separate lifecycle and action handlers. Its callback receives a `CallActionRequest` on the main CallKit delegate queue and must return promptly. The app may move the request or its `CallActionCompletion` to app-owned work; `complete()` only submits a decision and the framework applies it asynchronously on the main queue. Dropping an unsubmitted completion submits failure. CallKit timeout expires the completion without completing the already-timed-out native action again; provider reset, provider invalidation/drop, and call end invalidate pending completions. Late results return `CallActionCompletionError::Expired` or `ProviderUnavailable`. The framework adds no executor or networking policy.

Mute/hold events carry the state CallKit requests. They do not configure `AVAudioSession` or a media transport; the application owns audio and media behavior.

This crate does not implement PushKit/APNs, audio, outgoing-call transactions, call-directory extensions, default-calling-app role, app background execution, or app-specific pre-submit identity/ACK checks. See [the iOS CallKit provider guide](../../../../docs/ios/call-provider.md).

Run the focused registered capability profile on a macOS host with the required Apple SDKs and
inspection tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-call-provider
```

The profile supplies host tests, target checks, and link/import evidence. Static checks do not
establish CallKit runtime behavior on a Simulator or device.
