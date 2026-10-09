# Secure storage contract

## Scope

`framework-secure-storage` defines a portable facade for opaque secret bytes. It does not provide
a storage backend by itself. In particular, this slice does not implement Apple Keychain, another
platform store, encryption, key generation, biometric authentication, or remote synchronization.
The facade alone provides no secrecy.

The crate is `#![no_std]` and uses `alloc` for caller-owned read results. Each
`SecureStorage<B>` receives its backend state from the caller and statically dispatches through
`SecureStorageBackend`; it has no global lookup, hidden initialization, boxed trait object, or
executor. Backend methods are synchronous and may block; a backend must document its native
threading and availability behavior.

Availability is a backend-reported general signal and may be `Unknown`. It does not guarantee that
a particular service/item operation can succeed or that a requested policy can be met; the
operation result remains authoritative.

## Identifiers and operations

`ServiceId` and `ItemId` are borrowed UTF-8 identifiers. Each rejects empty text and NUL bytes but
otherwise passes the exact caller text without normalization, case folding, prefixing, or copying.
Backends must define any native naming constraints without silently changing these portable
identifiers.

`store` accepts opaque bytes and borrows them only for the call. A backend must copy or persist the
bytes without retaining the caller's borrow. Native storage APIs may require a copy or encoding
conversion, and the backend owns that cost. On success, `StoreOutcome` reports the effective
`AccessPolicy`; a backend that cannot satisfy a required policy must return
`SecureStorageError::UnsupportedPolicy` before mutating stored state. A backend may enforce a
stronger policy than requested, which can reduce availability, so callers should inspect the
reported policy.

`read` returns an owned `Vec<u8>`. The backend may need to copy from its native representation to
create this vector; the facade adds no further copy. The returned data is plaintext available to
the caller. This API does not keep caller memory protected after return, promise zeroization, or
prevent copies made by the caller, allocator, debugger, crash reporter, or platform.

`remove` returns `true` only when the item existed and was removed; `false` means no item existed.
The contract makes no crash-durability or multi-operation transaction guarantee. Backend errors
retain `framework_core::ErrorKind` and an optional native status code. Invalid identifiers map to
`InvalidInput`; an unsupported required policy maps to `Unsupported`.

## Access policy

`AccessPolicy` has two independent requirements:

- `device_unlock_required`: reads are allowed only while the device is unlocked. This does not
  request biometric or per-access user authentication.
- `device_bound`: the item must not become available through restore or migration to a different
  device.

`AccessPolicy::unrestricted()` requires neither property. `AccessPolicy::satisfies` checks whether
an effective policy fulfills a required policy. A backend may reject a requirement it cannot
provide, and must do so before any store mutation. These semantic flags do not select or prove a
particular platform protection class.

## Confidentiality and platform limits

The facade deals only in opaque bytes. It neither interprets secrets nor exposes cryptographic
keys, algorithms, key generation, or encryption APIs. It does not promise encryption at rest,
hardware-backed protection, access-control enforcement, secure deletion, backup exclusion, or
resistance to a compromised process. Those properties depend on the selected backend's chosen
public platform API and the caller's configuration.

## Current iOS backend

The separate B2 `ios-secure-storage` backend uses public Keychain Services generic-password
operations; it does not add Keychain types to this portable API. Its mapping of the two policy
flags, native errors, status codes, copy behavior, and Keychain lock-state limits are documented in
the [iOS backend guide](../ios/secure-storage.md). B2 currently reports `Availability::Available`
without checking device lock state or a requested item's policy; Keychain operations can still fail
under the selected accessibility class. The portable policy reports only its two modeled
requirements; it does not express the Keychain `AfterFirstUnlock` requirement for an initial
unlock after device restart.

The portable contract has no access-group selector. In B2, `SecItemAdd` without
`kSecAttrAccessGroup` creates an item in the app's default group, while unfiltered
`SecItemCopyMatching`, `SecItemUpdate`, and `SecItemDelete` queries search all groups available to
the app; updates and deletes affect all matching items. Therefore B2 does not configure or let
callers select a shared group, but it is not isolated to the default group when the host app has
multiple groups. Matching `ServiceId`/`ItemId` pairs may read, update, or delete items across those
groups. Hosts with multiple groups must avoid such collisions. This is native adapter behavior,
not a portable access-group or cross-platform guarantee.

The facade itself does not claim Keychain parity or guarantees stronger than a backend's public,
supported APIs. Any additional adapter must document its policy mapping, availability, errors,
native status codes, and data-copy behavior.

## Example

```rust
extern crate alloc;

use alloc::vec::Vec;
use framework_secure_storage::{
    AccessPolicy, ItemId, SecureStorage, SecureStorageBackend, SecureStorageError, ServiceId,
};

fn read_token<B: SecureStorageBackend>(
    storage: &mut SecureStorage<B>,
) -> Result<Option<Vec<u8>>, SecureStorageError> {
    let service = ServiceId::new("com.example.account")?;
    let item = ItemId::new("access-token")?;
    let _requested_for_new_item = AccessPolicy::new(true, true);
    storage.read(service, item)
}
```

The example's backend is supplied by its caller; this crate alone cannot read or write a secret.

## Contract tests and unsupported behavior

Unit tests use an in-memory backend to check identifier validation, policy rejection before
mutation, owned reads, effective policy reporting, and remove semantics. They do not validate
platform persistence, confidentiality, native protection, Keychain behavior, cryptography,
biometrics, or performance. The D2 workstream contains no platform backend; iOS Keychain behavior
belongs to the separate B2 workstream.
