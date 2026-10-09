# ios-call-observer

`ios-call-observer` provides one synchronous, iOS-only snapshot of the active calls returned by
`CXCallObserver.calls`. It copies only the call count and an ORed set of fixed-width state flags
into `CallActivitySnapshot`.

The first call-list read may block while CallKit retrieves initial state. Call this API only on
iOS 10.0 or later, and keep it off UI-critical work. The returned flags mean that **any** call in
the list has that state; flags are not correlated to one call. The snapshot may be stale as soon
as it returns.

This crate exposes no CallKit object, call UUID, caller identity, phone number, callback, provider,
audio session, PushKit path, or call action. It has no portable facade. Apple documents no prompt,
usage-description key, or entitlement for `CXCallObserver`; default calling-app and VoIP setup is
separate scope.

The package is a root-workspace member. Run its compile-only gate with:

```sh
sh platform/ios/callkit/ios-call-observer/check.sh
```

The check builds release link probes for iOS device and Simulator, then inspects their framework
imports, Objective-C runtime symbols, and deployment minimums without executing either binary. It
does not run tests, call CallKit at runtime, or validate live call visibility.
