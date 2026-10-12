# NFC reader support and NDEF reads

**framework-nfc** provides a portable `no_std` contract for NFC reader support, owned NDEF
messages, and a statically dispatched one-shot read facade. The iOS backend uses Apple's public
Core NFC `NFCNDEFReaderSession`; it does not make the portable API depend on Apple types.

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

The future starts native work on its first poll and does not require a framework-owned executor.
Drop a pending future to cancel and close the active session. NDEF record type, identifier, and
payload remain byte slices; the backend does not interpret records as UTF-8. The support snapshot
is still a separate synchronous query and does not imply permission or session readiness.

The iOS reader session requires iOS 11.0 or later, an NFC-capable device, a non-empty
`NFCReaderUsageDescription`, and the `com.apple.developer.nfc.readersession.formats` entitlement
with the NDEF format enabled. `new` reports unsupported reader hardware immediately; provisioning,
system access, user cancellation, radio-disabled, timeout, busy, and app-interruption outcomes are
reported distinctly when Core NFC invalidates the session. `UnsupportedOrNotProvisioned` preserves
Core NFC's ambiguous unsupported-feature result instead of claiming which prerequisite failed.

This increment reads one NDEF message and lets Core NFC invalidate after its first successful
read. It does not write NDEF data, expose native tag handles, implement ISO7816 or MIFARE commands,
support arbitrary `NFCTagReaderSession` protocols, or perform background reads. Compile and link
checks do not establish entitlement provisioning, the system prompt, physical tag reads, or
Simulator/device runtime behavior.

The prior availability-only increment and its query safety boundary remain in
[the iOS NFC guide](../ios/nfc.md).
