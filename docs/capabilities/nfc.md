# NFC reader support and NDEF reads and writes

**framework-nfc** provides a portable `no_std` contract for NFC reader support, owned NDEF
messages, and statically dispatched one-shot read and write facades. The iOS backend uses Apple's
public Core NFC `NFCNDEFReaderSession`; it does not make the portable API depend on Apple types.

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

The read operation reads one NDEF message and lets Core NFC invalidate after its first successful
read. It does not expose native tag handles, implement ISO7816 or MIFARE commands, support
arbitrary `NFCTagReaderSession` protocols, or perform background reads. Compile and link checks do
not establish entitlement provisioning, the system prompt, physical tag reads, or Simulator/device
runtime behavior.

The prior availability-only increment and its query safety boundary remain in
[the iOS NFC guide](../ios/nfc.md).

## One-shot NDEF write

Enable the optional `ndef-write` feature to use the public iOS writer. It starts Core NFC on the
first poll, waits for an NDEF tag, connects, checks writable status and exact encoded capacity,
then writes the owned message. Record type, identifier and payload stay opaque bytes, including
embedded zero bytes. Dropping a pending future invalidates the session.

```rust
use framework_nfc::{NdefMessage, NdefRecord, NdefTypeNameFormat, NdefWriter, NdefWriteError};
use ios_nfc::CoreNfcNdefWriteBackend;
use objc2::MainThreadMarker;

async fn write_ndef(main_thread: MainThreadMarker) -> Result<(), NdefWriteError> {
    let backend = CoreNfcNdefWriteBackend::new(main_thread, "Hold your iPhone near an NFC tag")?;
    let mut writer = NdefWriter::new(backend);
    let message = NdefMessage::new(vec![NdefRecord::new(
        NdefTypeNameFormat::Media,
        b"application/octet-stream".to_vec(),
        Vec::new(),
        vec![0, 0xff, b'x'],
    )]);
    writer.write(message).await
}
```

The feature uses the NDEF reader-session entitlement and usage description; it does not make a
tag writable or establish device provisioning. It does not expose tag handles, write non-NDEF
protocols, or provide background operation. Host and compile/link checks do not prove physical-tag
write behavior.
