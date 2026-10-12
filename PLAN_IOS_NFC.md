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

## Workstream B573 — one-shot NDEF read

### Objective and boundary

Add the first actual NFC tag operation to row 040 without changing the B37 reader-support query. The iOS backend owns one user-mediated `NFCNDEFReaderSession`, reads the first successful NDEF message, and allows Core NFC to invalidate after that read. It does not implement `NFCTagReaderSession`, tag handles, NDEF writes, ISO7816, MIFARE, card emulation, or background reading.

### Public API and lifecycle

- `framework-nfc` adds byte-preserving `NdefTypeNameFormat`, `NdefRecord`, and `NdefMessage`, `NdefReadError`, a statically dispatched `NdefReadBackend` future, and the `NdefReader<B>::read` facade.
- `ios-nfc` adds the default `ndef-read` feature and `CoreNfcNdefBackend`. `new(MainThreadMarker, alert_message)` checks only `NFCReaderSession.readingAvailable`, creates an inert typed Core NFC session, and sets its system-sheet alert message. `read().await` begins the session on first poll and returns the first owned message/error; dropping a pending future invalidates an active session.
- The portable NDEF model preserves record order and raw type/identifier/payload bytes; it does not transcode payloads to UTF-8.
- Core NFC weakly retains its delegate; `CoreNfcNdefBackend` strongly owns it. The supplied `DispatchQueue::main()` matches the `MainThreadOnly` delegate and backend. The small callback state slot uses `RefCell` on that main queue and wakes the caller's registered task after releasing the borrow. No global registry or executor is added. Rust panic is caught at the delegate boundary.
- iOS 11.0 is the public API floor. `NFCReaderUsageDescription` and `com.apple.developer.nfc.readersession.formats` with NDEF enabled are app provisioning requirements; neither is preflighted by the constructor. Hardware unsupported, unsupported-or-not-provisioned, security requirements, radio disabled, access denied, ineligibility, cancellation, timeout, interruption, and busy outcomes remain distinguishable. Core NFC error code 1 is intentionally reported as `UnsupportedOrNotProvisioned` because it can describe unsupported hardware or a missing entitlement.

### Validation and evidence boundary

Host tests cover opaque byte/record preservation, TNF round trips including reserved values, facade behavior, single start, first-terminal-result wins, cancellation/late-callback rejection, task wake, and Core NFC access/lifecycle error mapping. With Rust 1.94.1 and Xcode 26.6 / iOS SDK 26.5, these local gates passed:

```sh
cargo +1.94.1 test --locked --offline -p framework-nfc -p ios-nfc
cargo +1.94.1 check --locked --offline -p framework-nfc --no-default-features
cargo +1.94.1 check --locked --offline -p ios-nfc --no-default-features --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-nfc --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-nfc --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --offline -p ios-nfc --all-targets -- -D warnings
cargo +1.94.1 clippy --locked --offline -p ios-nfc --all-targets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --offline -p ios-nfc --all-targets --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.94.1 doc --locked --offline -p framework-nfc -p ios-nfc --no-deps
RUSTDOCFLAGS='-D warnings' cargo +1.94.1 doc --locked --offline -p ios-nfc --features ndef-read --target aarch64-apple-ios --no-deps
cargo +1.94.1 fmt --all -- --check
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-nfc/check-link-imports.sh
sh platform/ios/ios-nfc/check-link-imports.sh
git diff --check
```

The package-local Release link/import script built and inspected both target probes; the framework set remained exactly CoreNFC and Foundation with no Swift/Python runtime or unrelated capability crate. The installed validator identity is `350971094cc3b77191cd6b3d07e01af04d71943d`, SHA-256 `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`; its current `--list` has no `ios-nfc` profile and `--explain ios-nfc` returns `unknown capability`. Root integration must register the default `ndef-read` feature and new guards in the shared profile and update the capability-status row. The scoped commit omits root `Cargo.lock` by ownership instruction; root integration must resolve the added direct dependency edges before rerunning `--locked` gates against the integrated snapshot. Compile, lint, doc, and link checks do not establish entitlement provisioning, prompt behavior, NFC hardware access, physical tag reads, or Simulator runtime support. Do not edit generated reports in this workstream.
