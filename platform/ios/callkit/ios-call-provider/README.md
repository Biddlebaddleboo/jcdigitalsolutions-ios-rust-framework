# ios-call-provider

`ios-call-provider` owns one public CallKit `CXProvider` and exposes Rust call IDs, incoming-call reports, call-end reports, and synchronous answer/end action decisions. Its `objc2-call-kit` dependency is iOS-target-only, pinned at 0.3.2, and uses default features off.

The provider and its weakly stored delegate are main-thread-bound. Keep `CallProvider` alive for its service lifetime and call `invalidate()` before release. The provider retains its explicitly selected main dispatch queue because CallKit stores that queue weakly. Incoming-report completion uses a synchronized one-shot handoff and is always marshaled asynchronously to the main queue before it touches provider state or invokes the caller, regardless of the queue used by CallKit for the native block. A system-rejected report is an error; the caller must not proceed with the call. Answer/end handlers must return promptly and must not wait for network work.

This crate does not implement PushKit/APNs, audio, outgoing-call transactions, call-directory extensions, default-calling-app role, app background execution, or app-specific pre-submit identity/ACK checks. See [the iOS CallKit provider guide](../../../../docs/ios/call-provider.md).

Run the focused registered capability profile on a macOS host with the required Apple SDKs and
inspection tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-call-provider
```

The profile supplies host tests, target checks, and link/import evidence. Static checks do not
establish CallKit runtime behavior on a Simulator or device.
