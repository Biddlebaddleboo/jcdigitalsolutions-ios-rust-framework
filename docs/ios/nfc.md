# iOS Core NFC reader support and NDEF reads and writes

`ios-nfc` exposes separate reader-support, NDEF read, and opt-in NDEF write operations through
public Core NFC APIs.

## Reader support snapshot

The stateless `CoreNfcReaderBackend` maps the readonly `NFCReaderSession.readingAvailable`
property to `NfcReaderAvailability::Supported` or `Unsupported`. It is a device-capability query,
not an authorization, entitlement, configuration, session, scan, or tag-read check. The property
is available from iOS 11.0. Construction is inert, and calling `snapshot` does not prompt, start
polling, or access tag data.

```rust
use framework_nfc::NfcReader;
use ios_nfc::CoreNfcReaderBackend;

let reader = NfcReader::new(CoreNfcReaderBackend);
let availability = reader.snapshot();
```

The support query conservatively requires a non-empty `NFCReaderUsageDescription` in the app's
`Info.plist`. This key does not request permission or start a session. The query alone does not
claim the reader-session entitlement is required or sufficient.

## One-shot NDEF read session

With the default `ndef-read` Cargo feature, `CoreNfcNdefBackend::new` creates an inert
`NFCNDEFReaderSession` on the iOS main thread. The first poll of `NdefReader::read` begins polling
and presents Apple's user-mediated system reader UI; the first successful NDEF message is copied
into portable owned records in record order. The portable record type, identifier, and payload
are raw bytes, not assumed UTF-8 strings.

```rust
use framework_nfc::{NdefMessage, NdefReader, NdefReadError};
use ios_nfc::CoreNfcNdefBackend;
use objc2::MainThreadMarker;

async fn read_ndef(
    main_thread: MainThreadMarker,
) -> Result<NdefMessage, NdefReadError> {
    let backend = CoreNfcNdefBackend::new(main_thread, "Hold your iPhone near an NFC tag")?;
    let mut reader = NdefReader::new(backend);
    reader.read().await
}
```

The read future starts native work on its first poll; dropping it while pending closes the active
session. The backend strongly retains its Rust delegate while the Core NFC session holds its
documented weak delegate reference. Native callbacks run on the supplied main dispatch queue,
update a small `RefCell` result slot, then wake the caller's future task after releasing the borrow.
No app-owned executor or global state is used.

NDEF session use requires a non-empty `NFCReaderUsageDescription` plus the
`com.apple.developer.nfc.readersession.formats` entitlement with the NDEF format enabled. The
constructor checks only `readingAvailable`; it cannot preflight signing entitlements, Info.plist,
or user/system access. It reports unsupported hardware immediately and keeps Core NFC outcomes
distinct: unsupported-or-not-provisioned feature, security requirements, disabled radio, denied
access, ineligibility, cancellation, timeout, interruption, and a busy reader. Core NFC error code
1 can mean unsupported hardware or missing entitlement, so it maps to
`UnsupportedOrNotProvisioned`, not a false hardware-only diagnosis.

`NFCNDEFReaderSession` and the reader-support query both have an iOS 11.0 API floor. The installed
Xcode 26.6 / iOS SDK 26.5 is build-environment evidence, not a new minimum Xcode baseline.
`objc2-core-nfc` 0.3.2 supplies the typed Objective-C bindings. NDEF session construction uses its
`dispatch2`-gated initializer with `DispatchQueue::main()` for main-thread delegate delivery.
The dependency remains target-gated to iOS; no Swift source, raw ABI, or unrelated Apple framework
is used.

The read operation reads one valid NDEF message from NFC Forum tag types 1–5 and allows Core NFC to
invalidate after the first successful read. It does not expose a native tag handle, implement
custom `NFCTagReaderSession` protocols, ISO7816 APDUs, MIFARE commands, card emulation, or
background reading. Host tests and device/Simulator compile/link checks are not proof of
provisioning, prompt behavior, physical tag access, or Simulator NFC runtime support.

## One-shot NDEF write session

Enable the optional `ndef-write` Cargo feature to expose `CoreNfcNdefWriteBackend`. Its
`NFCNDEFReaderSession` implements the public `readerSession:didDetectTags:` callback, making the
session read-write; it sets `invalidateAfterFirstRead` to `false`. The first future poll begins
the system session. The backend connects to the first detected NDEF tag, queries its status and
capacity, and writes only when the tag is writable and the complete encoded message fits. Read-only,
unsupported, too-small, access, and lifecycle failures remain terminal errors. The same NDEF
usage-description and reader-session entitlement requirements apply.

NDEF `TNF`, type, identifier, and payload bytes are passed to Core NFC without UTF-8 conversion.
The portable contract rejects empty or malformed records, chunked continuation records, and
messages over Core NFC's 128 KiB limit. The native message length is checked against the portable
encoded length before polling starts. Dropping a pending future invalidates the active session;
native callbacks are main-queue-bound, panic-contained, and settle one result.

This does not add arbitrary tag protocols, persistent native tag handles, background writes, or
hardware claims. The iOS device and Simulator link profile compiles the opt-in writer and checks
Core NFC/Foundation imports and deployment floors; no physical-tag write or live session runtime
was performed.

The capability boundary and remaining NFC gaps are tracked in [the Core NFC plan](../../PLAN_CAPABILITIES_NFC.md).
