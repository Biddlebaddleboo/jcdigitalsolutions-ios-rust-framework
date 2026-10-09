# PLAN_IOS_NFC.md — Workstream B37: iOS Core NFC Reader-Support Query

## Objective

Implement the D32 reader-support snapshot through the public Core NFC class property `NFCReaderSession.readingAvailable`. Do not implement a reader session, prompt, tag polling, tag read/write, or background tag reading.

## API and requirements evidence

- Installed iPhoneOS SDK: Xcode 26.6, iOS SDK 26.5
- `NFCReaderSession.readingAvailable` is a readonly class property marked `API_AVAILABLE(ios(11.0))` in the local CoreNFC SDK header; iOS 11.0 is the API floor, not a claim about the minimum Xcode baseline
- `objc2-core-nfc` 0.3.2 provides typed `NFCReaderSession::readingAvailable() -> bool` as an unsafe class method
- Apple documents this property as a Boolean for whether the device supports NFC tag reading
- Apple documents `NFCReaderUsageDescription` as required when an app uses APIs that access NFC hardware; this plan conservatively requires a non-empty value for the query because Apple does not explicitly discuss a property-only exception
- The entitlement `com.apple.developer.nfc.readersession.formats` is for NFC tag-reading formats and tag-session access. This snapshot does not create or use a tag reader session and makes no claim that the entitlement is needed or sufficient for a future tag operation

## Dependencies

- Foundation A, iOS runtime B, and D32 `framework-nfc` are integrated
- Review `PLAN_CAPABILITIES_NFC.md`, `docs/OBJC_INTEROP.md`, `docs/OWNERSHIP.md`, and `docs/UNSAFE.md`
- Apple references: [NFCReaderSession](https://developer.apple.com/documentation/corenfc/nfcreadersession), [readingAvailable](https://developer.apple.com/documentation/corenfc/nfcreadersession-swift.class/readingavailable), [NFCReaderUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nfcreaderusagedescription), and [reader-session formats entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.nfc.readersession.formats)

## Write scope

- `platform/ios/ios-nfc/**`
- `docs/ios/nfc.md`

Do not edit the portable D32 crate, root workspace manifest, capability status manifest, CI, aggregate plans, documentation indexes, Swift, C bindings, or unrelated backends. The lockfile may change only for the new workspace package and `objc2-core-nfc` dependency entries.

## Required implementation

- Use `objc2-core-nfc` 0.3.2 with default features disabled and the typed Core NFC binding; do not use raw ABI calls
- Map the readonly Boolean property to `Supported` or `Unsupported`
- Keep construction inert and expose no Apple type through the portable contract
- Document and narrowly justify the unsafe property call
- Do not instantiate `NFCReaderSession`, `NFCNDEFReaderSession`, `NFCTagReaderSession`, or any other session class
- Do not prompt, scan, poll, read/write a tag, store a native handle, or claim background tag reading

## Validation and handoff

- Run device and simulator `cargo check` and Clippy gates from `PLAN_VALIDATION_IOS_NFC.md`
- Run format, docs, no-Swift, and diff checks specified by that subplan
- Document the iOS 11.0 API floor separately from the installed SDK/Xcode version, Info.plist key, entitlement boundary, and live-device limitations
- Report exact commands, results, public symbols, limits, and unresolved requirements
