# Secure-storage C ABI

## Opt-in surface

`framework-c-api` keeps secure storage out of the default build. Enable its `secure-storage`
Cargo feature and include [`framework_ios_secure_storage.h`](../../bindings/c/include/framework_ios_secure_storage.h)
in addition to [`framework.h`](../../bindings/c/include/framework.h). The separate capability
header leaves Keychain calls out of the foundation header. Rust callers should use
`SecureStorage<IosSecureStorage>` directly rather than route through C.

The feature adds the optional `framework-secure-storage` contract and `ios-secure-storage` backend
dependencies. On iOS, calls use public Security framework generic-password APIs. `SecItemAdd`
without `kSecAttrAccessGroup` creates in the app's default Keychain group; `SecItemCopyMatching`,
`SecItemUpdate`, and `SecItemDelete` omit that filter and search all groups available to the app.
Updates and deletes affect all matching items. The C ABI exposes no group selector, so a matching
service/item pair in another entitled group may be read, updated, or deleted. Consumers must link
`Security.framework` and `CoreFoundation.framework`. On non-iOS targets the feature exports
argument-validating stubs
that return `FRAMEWORK_STATUS_UNSUPPORTED`; they do not link Apple frameworks.

## Operations and threading

The exported functions are `framework_ios_secure_storage_read`,
`framework_ios_secure_storage_store`, and `framework_ios_secure_storage_remove`. They are
synchronous and may block while Keychain Services performs interprocess work. Callers should avoid
using them on latency-sensitive or UI threads. They create no global registry, backend handle,
callback, executor, or hidden initialization state.

`FrameworkStr` identifiers are UTF-8 byte spans. Empty strings, NUL-containing identifiers,
invalid UTF-8, null pointers with nonzero lengths, lengths not representable as `usize`, unknown
policy bits, and missing required output pointers return `FRAMEWORK_STATUS_INVALID_ARGUMENT`
before any Keychain operation. A zero-length `FrameworkSlice` may use a null data pointer and
represents a valid empty secret. As with other C APIs, callers must provide readable/writable
memory for each valid non-empty input/output span; an implementation cannot safely probe whether
an arbitrary non-null address is mapped.

```c
FrameworkStr service = {(const uint8_t *)"com.example.account", 19};
FrameworkStr item = {(const uint8_t *)"access-token", 12};
FrameworkSlice bytes = {(const uint8_t *)secret, secret_length};
uint32_t effective_policy = 0;
int32_t native_status = 0;

FrameworkStatus status = framework_ios_secure_storage_store(
    service,
    item,
    bytes,
    FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_UNLOCK_REQUIRED |
        FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_BOUND,
    &effective_policy,
    &native_status);
```

The policy flags are fixed `uint32_t` bits:

| Flag | Value | Portable requirement |
| --- | ---: | --- |
| `FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_UNLOCK_REQUIRED` | `0x00000001` | Reads require the device to be unlocked; this does not request biometrics or per-access authentication |
| `FRAMEWORK_IOS_SECURE_STORAGE_POLICY_DEVICE_BOUND` | `0x00000002` | The item must not become available through restore or migration to another device |

The effective flags from a successful store report the Keychain accessibility class selected by
`ios-secure-storage`. `AfterFirstUnlock` still requires an initial unlock after restart, and the
two portable flags do not describe that availability detail. See the [iOS backend contract](../ios/secure-storage.md)
for the exact mapping and security limits.

## Output and error rules

Every required output is initialized before parsing inputs or performing Keychain work. Required
output pointers must be writable and distinct; an optional native-status pointer must also be
writable and must not alias another output. For read, each non-null required output is initialized
before a missing required pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`. A non-null
`out_secret` must not contain a live framework-owned allocation on entry; the call sets it empty
first, then returns a buffer only for a present item. A zero `out_found` means absent;
`out_found == 1` with a zero-length buffer means
the item exists and its secret is empty. Destroy every returned owned buffer exactly once with
`framework_owned_buffer_destroy`; do not copy or alter its descriptor.

`out_removed` is `1` only when one or more matching items existed and were removed; the iOS
Keychain query may remove every match across the app's groups. `out_effective_policy_flags` is
zero on failure. Optional `out_native_os_status` is set to zero before fallible input parsing and
remains zero for non-platform errors. For a Keychain failure, the portable result is
`FRAMEWORK_STATUS_PLATFORM_ERROR` and the original nonzero signed `OSStatus` is copied to that
output. Other portable secure-storage errors use the stable `FrameworkStatus` mapping from the
foundation ABI. Unknown policy bits are rejected and do not reach Keychain. Panics are contained
inside each exported function and reported as `FRAMEWORK_STATUS_PANIC`.

## Validation scope

`bindings/c/check-secure-storage.sh` runs deterministic Rust tests, C11 and C++17 header compiles,
a capability symbol audit, and a linked host C consumer against the validating non-iOS stubs. It
builds device and simulator static libraries, then links a C probe that calls read, store, and
remove. For those probe binaries only, `otool -L` enforces the exact direct import set
`CoreFoundation`, `Security`, and `libSystem.B.dylib`; `nm -u` requires Keychain/CoreFoundation
symbols and rejects Swift, Objective-C, and network symbols. This is probe-scoped evidence, not a
claim about arbitrary app links. These checks do not exercise a live Keychain, signed application,
physical device, lock state, access-group configuration, Keychain durability, secure deletion, or
performance. The C binding exposes no access-group configuration, async operation, callbacks,
cancellation, or native object handle.
