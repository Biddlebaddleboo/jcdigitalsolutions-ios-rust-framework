# iOS NFC reader-support snapshot

**ios-nfc** implements the narrow `framework-nfc` snapshot with the public Core NFC readonly class property `NFCReaderSession.readingAvailable`. Apple defines it as a Boolean that reports whether the device supports NFC tag reading. The iOS SDK header marks the property available starting at iOS 11.0. That is the API floor; the installed Xcode 26.6 / iOS SDK 26.5 is build-environment evidence, not a new minimum Xcode baseline.

```rust
use framework_nfc::NfcReader;
use ios_nfc::CoreNfcReaderBackend;

let reader = NfcReader::new(CoreNfcReaderBackend);
let snapshot = reader.snapshot();
```

Backend construction is inert. `snapshot` calls the typed `objc2-core-nfc` 0.3.2 `NFCReaderSession::readingAvailable()` binding and maps `true` to `Supported` and `false` to `Unsupported`. The method is unsafe in the generated binding; this crate confines that call and documents its safety basis. It calls no session initializer, selector other than the documented property, prompt API, polling API, delegate, or tag operation. There is no global runtime, native handle, callback, background task, or cancellation behavior in this snapshot.

## App configuration and capability limits

Apple documents `NFCReaderUsageDescription` as required when an app uses APIs that access NFC hardware. This guide conservatively requires a non-empty `NFCReaderUsageDescription` string in the app's `Info.plist`; Apple does not explicitly discuss whether the standalone `readingAvailable` property is exempt from the general requirement. This key is a usage-description declaration and does not itself request permission or start a session. Apple documents `com.apple.developer.nfc.readersession.formats` for formats an app may read through NFC tag-reader sessions. This query does not create/use a tag-reader session and does not claim that entitlement is required for the query. A future session/tag implementation must independently verify and satisfy the applicable session-specific entitlement and Info.plist requirements.

The returned support bit is not a promise that an app is provisioned, configured, permitted, or able to start a session. It does not imply that NFC is available in the current environment or that a tag can be discovered. No live-device or simulator NFC result is part of compile/lint validation. This backend does not support prompting, sessions, scans, polling, NDEF, any tag read/write protocol, card emulation, or background tag reading.

The `objc2-core-nfc` dependency is target-gated to iOS with default features disabled. Version 0.3.2 has no per-type Cargo feature for `NFCReaderSession`; its generated binding is available without enabling its optional `block2` or `dispatch2` features. The binding crate also depends on `objc2-foundation` and links Foundation, so the native framework dependency set is CoreNFC plus this binding-required Foundation dependency, not CoreNFC alone. No unrelated capability framework, Swift source, or raw ABI declaration is used.

See [PLAN_IOS_NFC.md](../../PLAN_IOS_NFC.md) and [PLAN_VALIDATION_IOS_NFC.md](../../PLAN_VALIDATION_IOS_NFC.md) for the scoped gates and boundaries.
