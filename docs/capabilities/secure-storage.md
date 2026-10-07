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
resistance to a compromised process. Those properties depend on a future backend's chosen public
platform API and the caller's configuration.

Apple Keychain is a future system-owned backend. No Keychain type or other platform type appears
in this portable API, and this contract does not claim Keychain parity, availability, entitlement
requirements, or an iOS implementation. A future adapter must document its mapping of access
policy, availability, errors, native status codes, and data-copy behavior. It must use public,
supported APIs and must not claim guarantees stronger than those APIs provide.

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
biometrics, or performance. No Apple or other platform backend is included in this workstream.
