# iOS Call activity snapshot

The `ios-call-observer` package exposes one synchronous query:
`active_call_snapshot() -> CallActivitySnapshot`. It uses CallKit's public
`CXCallObserver.calls` property and returns only an active-call count (`u64`) and
aggregate state flags (`u32`). The flags record whether any returned call is outgoing,
connected, on hold, or ended. They are independent ORed facts and do not identify or
correlate a single call.

The API floor is iOS 10.0. CallKit may block while it retrieves the initial call list,
so keep the query off UI-critical work. The package registers no delegate and has no
callback lifecycle; it releases all CallKit objects before return. It does not read or
return call UUIDs, phone numbers, caller identities, call history, audio state, or call
control handles.

Apple documents that any app may create a `CXCallObserver`; the snapshot has no
documented permission prompt, usage-description key, or entitlement. This does not
cover a full VoIP provider, PushKit/APNs delivery, Call Directory extensions, or default
calling-app support. On iOS and iPadOS 18.2+, default calling-app support has separate
requirements, including `com.apple.developer.calling-app` and a `UIBackgroundModes`
entry of `voip`.

This is a current-state snapshot, not CallKit availability or service readiness. State
may change immediately after the function returns. Do not log or persist the values by
default.

Package-local compile, Clippy, docs, format, and dependency-feature checks:

```sh
sh platform/ios/callkit/ios-call-observer/check.sh
```
