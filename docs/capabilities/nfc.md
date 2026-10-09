# NFC reader availability

**framework-nfc** provides a portable `no_std` contract for a one-time snapshot of NFC tag-reader support. It is not a portable NFC session or tag API. The separate [iOS guide](../ios/nfc.md) documents the Core NFC query.

```rust
use framework_nfc::{NfcReader, NfcReaderAvailabilityBackend};

fn inspect_support<B: NfcReaderAvailabilityBackend>(reader: &NfcReader<B>) {
    match reader.snapshot() {
        framework_nfc::NfcReaderAvailability::Supported => { /* device reports reader support */ }
        framework_nfc::NfcReaderAvailability::Unsupported => { /* device reports no reader support */ }
        framework_nfc::NfcReaderAvailability::Unknown => { /* backend cannot classify support */ }
        _ => unreachable!("NfcReaderAvailability is non-exhaustive"),
    }
}
```

`NfcReaderAvailability` is a copied one-byte semantic value. `NfcReader<B>` owns the explicitly supplied static backend; the crate has no platform lookup, global registry, allocation, executor, dynamic dispatch, prompt, or hidden initialization. The query is synchronous and each call is a fresh backend snapshot, not a subscription or cached session state.

`Supported` means only that the backend reports device support for NFC tag reading. It does not mean an app has `NFCReaderUsageDescription`, an NFC reader-session entitlement, permission, a currently usable radio, a started session, a present tag, successful tag discovery, or background reading. `Unknown` allows a backend with no reliable support query to avoid reporting a false negative or positive.

This slice does not implement sessions, NDEF, tag discovery, tag protocols, tag reads/writes, card emulation, prompting, or background tag reading. Thus it is only a partial facet of the broader NFC session/tag-operations capability.

Portable checks:

```sh
cargo fmt --all -- --check
cargo test -p framework-nfc
cargo check -p framework-nfc --no-default-features
git diff --check
```
