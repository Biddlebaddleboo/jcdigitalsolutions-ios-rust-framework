# iOS VoIP PushKit client

`ios-voip-pushkit` is an opt-in Rust owner for a public `PKPushRegistry` VoIP registration and its weak delegate. It emits opaque token updates and invalidation events, copies incoming PushKit dictionaries to JSON bytes, and exposes a one-shot completion that the app can retain through its CallKit report.

Create `IosVoipPushService` on the main thread at app launch and keep it alive for the period in which the app accepts VoIP pushes. Parse the payload according to the app's own protocol, build an `IncomingCall`, and call `IncomingVoipPush::report_incoming`; that method reports through `ios-call-provider::CallProvider` and invokes PushKit completion only after the CallKit report callback. For callers with a separate supported CallKit implementation, `finish_after_callkit_report` remains an explicit completion escape hatch. Dropping the event invokes PushKit completion once as a leak-safe fallback, but does not report a call to CallKit.

This client does not provide an APNs provider/server, entitlement, `UIBackgroundModes`, payload schema, identity, signaling, or CallKit provider. Token delivery does not prove APNs server registration or runtime delivery. The package floor is iOS 11.0 for the completion-bearing incoming-push callback.

See [the focused guide](../../../docs/ios/voip-pushkit.md). Build the link probe with `sh platform/ios/ios-voip-pushkit/check-link-imports.sh`; do not run the link probe as an app because it requests VoIP registration and intentionally has no application payload handler.
