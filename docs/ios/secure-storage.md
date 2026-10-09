# iOS secure storage

`ios-secure-storage` implements the portable `framework-secure-storage` contract with public
Keychain Services generic-password items. The backend is the zero-sized, caller-owned
`IosSecureStorage` value; it has no global registration or initialization step. `SecItemAdd`
uses the app's default Keychain access group when no group is supplied, while lookup, update, and
delete queries without a group filter search all access groups available to the app. Rust callers
use `SecureStorage<IosSecureStorage>` directly and remain on the Rust-native API path.

```rust
use framework_secure_storage::{
    AccessPolicy, ItemId, SecureStorage, SecureStorageError, ServiceId,
};
use ios_secure_storage::IosSecureStorage;

fn save_and_read() -> Result<(), SecureStorageError> {
    let mut store = SecureStorage::new(IosSecureStorage::new());
    let service = ServiceId::new("com.example.account")?;
    let item = ItemId::new("access-token")?;
    let required = AccessPolicy::new(true, true);
    let outcome = store.store(service, item, b"opaque secret", required)?;
    assert!(outcome.effective_policy().satisfies(required));
    let _secret = store.read(service, item)?;
    Ok(())
}
```

## Identity, operations, and errors

The adapter maps `ServiceId` to `kSecAttrService` and `ItemId` to `kSecAttrAccount`, and limits
items to `kSecClassGenericPassword`. Identifiers are passed as UTF-8 Core Foundation strings
without normalization or prefixing. The adapter does not set `kSecAttrAccessGroup` or
`kSecAttrSynchronizable`. For `SecItemAdd`, omitting `kSecAttrAccessGroup` selects the app's
default group. For `SecItemCopyMatching`, `SecItemUpdate`, and `SecItemDelete`, Apple documents
that omitting the group searches all groups available to the app; updates and deletes affect all
matching items. Therefore this backend does not select or configure a shared group, but it is not
isolated to the default group when the host app belongs to multiple access groups. A matching
service/account identity in another available group may be read, updated, or deleted. Hosts that
use multiple groups must ensure these identifiers do not collide with items managed outside this
backend. See Apple's [`kSecAttrAccessGroup`](https://developer.apple.com/documentation/security/ksecattraccessgroup),
[`SecItemCopyMatching`](https://developer.apple.com/documentation/security/secitemcopymatching%28_%3A_%3A%29),
[`SecItemUpdate`](https://developer.apple.com/documentation/security/secitemupdate%28_%3A_%3A%29),
and [`SecItemDelete`](https://developer.apple.com/documentation/security/secitemdelete%28_%3A%29) documentation.

`read` calls `SecItemCopyMatching` with `kSecReturnData`. `store` first calls `SecItemUpdate` for
the exact class/service/account identity and adds a missing item with `SecItemAdd`; a duplicate
created by a concurrent caller is followed by one update retry. `remove` calls `SecItemDelete`.
The native `OSStatus` is retained as `PlatformErrorCode` for failures. Only `errSecItemNotFound`
maps to `None` for read or `false` for remove; a missing item during the store update is the normal
add path, not a caller-visible absence.

These calls are synchronous and may block on Keychain work and interprocess service activity;
Apple documents `SecItemUpdate` as blocking its calling thread ([`SecItemUpdate`](https://developer.apple.com/documentation/security/secitemupdate%28_%3A_%3A%29)).
The backend performs no callback, executor hop, or main-thread dispatch. A single facade needs
exclusive `&mut` access per operation; separate backend values may issue calls concurrently, and
the adapter provides no multi-operation transaction or caller-level isolation guarantee. A read
copies the returned `CFData` into a caller-owned `Vec<u8>`; a store copies the borrowed Rust bytes
into `CFData` before Security persists them. The returned vector is plaintext. The adapter makes
no promise of zeroization, prevention of caller/debugger/crash copies, crash durability, or secure
deletion.

## Access-policy mapping and limits

| Portable requirement | Keychain accessibility | Reported effective policy |
| --- | --- | --- |
| `device_unlock_required = false`, `device_bound = false` | `kSecAttrAccessibleAfterFirstUnlock` | `(false, false)` |
| `device_unlock_required = true`, `device_bound = false` | `kSecAttrAccessibleWhenUnlocked` | `(true, false)` |
| `device_unlock_required = false`, `device_bound = true` | `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly` | `(false, true)` |
| `device_unlock_required = true`, `device_bound = true` | `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` | `(true, true)` |

The mapper resolves the requested class and verifies the effective-policy relation before any
Keychain mutation. All four current boolean combinations map to public classes, so the current
contract has no unsupported combination. `AfterFirstUnlock` classes still require an initial
unlock after device restart, then allow access while the device is subsequently locked; this
availability detail is not represented by the portable two-flag policy. `WhenUnlocked` classes
are unavailable while the device is locked. Keychain errors such as `errSecInteractionNotAllowed`
remain visible as their original native status. These availability meanings follow Apple's
[accessibility value definitions](https://developer.apple.com/documentation/security/item-attribute-keys-and-values#Accessibility-Values).

This backend does not provide biometric or per-access authentication or shared-access-group
selection. It does not provide iCloud Keychain synchronization, secure deletion, explicit
zeroization, crash durability, or hardware-backed custom-key APIs. It stores only opaque
generic-password bytes and does not add cryptography. No Security object or native handle is
returned or retained; this adapter exposes no native escape-handle API. Applications that need
custom access groups or other Keychain
attributes must use a separately scoped platform integration with the required signing
configuration.

The used `SecItemAdd`, `SecItemCopyMatching`, `SecItemUpdate`, and `SecItemDelete` declarations in
the installed Security SDK are marked available from iOS 2.0. The four selected accessibility
constants are marked available from iOS 4.0 in the installed iOS 26.5 SDK. The crate does not set
or test an application deployment target; iOS 4.0 is the lowest SDK-annotated availability among
these symbols, not a claim that this application has been tested on that OS release. Default-group
adds do not require a custom `Info.plist` key or shared-access-group entitlement. The host app's
signed entitlements determine which groups unfiltered searches can see; this backend does not
select or configure a non-default group. Strict default-group isolation is not provided by this
implementation. If required, it remains an open design item: the backend needs a public, reliable
way to select the signed app's default group for each lookup, update, and delete.

## Binding and dependency boundary

The backend uses `objc2-security` 0.3.2 with only `SecBase` and `SecItem` features and
`objc2-core-foundation` 0.3.2 with `CFData`, `CFDictionary`, `CFNumber`, `CFString`, and `alloc`.
The crate's `CFNumber` feature is needed to expose the generated `kCFBooleanTrue` singleton used
with `kSecReturnData`; the adapter does not construct or consume a `CFNumber` value.
These generated bindings cover the exact Security declarations used here: `SecItemAdd`,
`SecItemCopyMatching`, `SecItemUpdate`, `SecItemDelete`, generic-password class/service/account,
return-data/value-data, and `kSecAttrAccessible` plus the four selected classes. Core Foundation
provides typed string/data values, dictionary call-back constants, retain ownership, and a
caller-owned data copy. The small unsafe boundary only builds a Core Foundation dictionary from
valid CF references and calls the declared Security functions. No handwritten Security ABI or
string-encoded substitute key is needed.

This choice differs from the `objc2-security` feasibility note in
[`NATIVE_CAPABILITY_MATRIX_PASS2.md`](../research/NATIVE_CAPABILITY_MATRIX_PASS2.md), which only
established that `SecItemAdd` is exposed. The installed crate binding also exposes the other three
operations and all required attributes. In the locally inspected `security-framework-sys` 2.17.0
binding, the accessibility *values* are present but the `kSecAttrAccessible` dictionary key is
not, so using that binding alone would need one additional handwritten extern declaration or an
unsupported literal key. The selected `objc2-security` binding has complete key/value coverage for
this operation set. `objc2-security`'s broad default feature set is disabled; `objc2` itself and
its Objective-C runtime feature are not enabled, since these APIs are C/Core Foundation calls.
The binding layer remains private to this crate; neither Security nor Core Foundation types enter
the portable API. These dependencies are target-scoped to iOS and do not affect portable
`no_std` crates. The only enabled transitive helper is `bitflags` for generated Core Foundation
feature definitions; this dependency path has no build scripts or proc macros. The repository's
opt-in C link probe calls read, store, and remove, then checks device and simulator Mach-O imports.
For those probe binaries only, the exact direct import set is `CoreFoundation` from
`CoreFoundation.framework`, `Security` from `Security.framework`, and `libSystem.B.dylib`; the
`nm -u` scan rejects Swift, Objective-C, and network symbols. This is not a claim about the full
imports of an app that links the backend. The generated bindings avoid duplicating ABI types,
ownership annotations, and framework symbol declarations by hand. Replacing them later is
localized to `keychain.rs` and `Cargo.toml`, behind the same `SecureStorageBackend` implementation

The selected native symbols are public `Security.framework` and `CoreFoundation` APIs. The
backend does not call Objective-C methods, use private APIs, or require Swift runtime support.
Signing selects the app's default group for adds; unfiltered searches retain the all-entitled-groups
behavior described above. This adapter does not configure group membership or expose a group
selector; custom group setup and entitlements remain the host app's responsibility.

## Validation scope

Host tests cover the four policy mappings and status translation without accessing an app
Keychain. Device and simulator target checks cover compilation and linkage declarations only.
They do not prove persistence, lock-state behavior, access-group behavior, live native error
semantics, secure deletion, or performance on a signed app or physical device. No live-device
Keychain test is included in this workstream.
