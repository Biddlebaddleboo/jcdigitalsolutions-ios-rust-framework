# ios-call-provider

`ios-call-provider` owns one public CallKit `CXProvider` and exposes Rust call IDs, incoming-call reports, outgoing `CXStartCallAction` transactions, call progress/end reports, and start/answer/end/mute/hold action decisions. Its default `audio-session-events` feature adds provider audio-session activation/deactivation notifications using the iOS-target-only `objc2-call-kit` and `objc2-avf-audio` dependencies pinned at 0.3.2 with default features off. Disable the feature to omit that callback surface and AVFAudio dependency.

The provider and its weakly stored delegate are main-thread-bound. Keep `CallProvider` alive for its service lifetime and call `invalidate()` before release. The provider retains its explicitly selected main dispatch queue because CallKit stores that queue weakly. Incoming-report completion uses a synchronized one-shot handoff and is always marshaled asynchronously to the main queue before it touches provider state or invokes the caller, regardless of the queue used by CallKit for the native block. A system-rejected report is an error; the caller must not proceed with the call.

`CallProvider::new` preserves its synchronous call-action handler contract; that handler must return promptly and must not wait for network work. Events include answer/end requests, requested mute/hold states, provider reset, and action timeout. With the default `audio-session-events` feature, the same handler receives audio-session activation/deactivation events; lifecycle return values are ignored. `CallProvider::new_with_deferred_actions` adds separate lifecycle and action handlers; its lifecycle handler receives provider reset, action timeout, and enabled audio-session events. All delegate handlers run on the main CallKit delegate queue. The action handler receives a `CallActionRequest` and must return promptly. The app may move the request or its `CallActionCompletion` to app-owned work; `complete()` only submits a decision and the framework applies it asynchronously on the main queue. Dropping an unsubmitted completion submits failure. CallKit timeout expires the completion without completing the already-timed-out native action again; provider reset, provider invalidation/drop, and call end invalidate pending completions. Late results return `CallActionCompletionError::Expired` or `ProviderUnavailable`. The framework adds no executor or networking policy.

Mute/hold events carry the state CallKit requests. When enabled, audio-session activation/deactivation events only report CallKit's lifecycle callbacks; they do not expose or retain `AVAudioSession`, configure it, or configure a media transport. The application owns audio and media behavior.

`CallProvider::request_outgoing` requests one CallKit start transaction and reports only whether CallKit accepted that transaction; the separately delivered start action is where the app performs signaling and chooses fulfillment. The progress reports mark app-reported connecting and connected states but do not verify transport or media. This crate does not implement PushKit/APNs, signaling, audio, media transport, call-directory extensions, default-calling-app role, app background execution, or app-specific pre-submit identity/ACK checks. See [the iOS CallKit provider guide](../../../../docs/ios/call-provider.md).

Run the focused registered capability profile on a macOS host with the required Apple SDKs and
inspection tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-call-provider
```

The profile supplies host tests, target checks, and link/import evidence. Static checks do not
establish CallKit runtime behavior on a Simulator or device.
