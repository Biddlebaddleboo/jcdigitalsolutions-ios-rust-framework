# Application, files, preferences, and foreground HTTP

## Scope and runtime status

D1 adds four independently usable `no_std` crates: `framework-app`, `framework-files`,
`framework-preferences`, and `framework-network`. Each uses a caller-supplied generic backend, so
Rust monomorphization selects the implementation at build time. There is no boxed backend, runtime
lookup, global registry, framework initialization call, or required executor.

These crates define portable contracts only; D1 adds no native backend implementation. The
integrated workspace has separate `ios-files`, `ios-preferences`, and `ios-network` adapters, as
recorded in the canonical manifest. Those adapters are not part of these D1 crates, and a portable
contract alone does not establish iOS runtime support. The `framework-app` lifecycle contract is
still not connected to the partial UIKit app example described in [the iOS runtime
guide](../ios/runtime.md); that example does not provide a reusable lifecycle or UI facade.

The complete row-by-row source of truth for all V1 capability families is
[`capability-status.json`](capability-status.json). It marks unsupported implementations `X` and
uses `null` for unverified platform metadata. The UIKit sample remains partial `B` support; the
manifest separately records iOS backends for sandbox files, preferences, and foreground HTTP.
Read-only packaged resources use the separate D7 [`resources` contract](resources.md), distinct
from D1's writable sandbox-file contract. The integrated workspace has a separate `ios-resources`
main-bundle backend for exact ordinary files; its limits are documented in the
[iOS resources guide](../ios/resources.md).

## Application lifecycle

`framework-app` defines `ApplicationState`, `LifecycleEvent`, and `ApplicationBackend`. Create an
`Application<B>` from an explicitly supplied backend; `availability`, `state`, and `poll_event`
call that backend directly. `poll_event` is nonblocking. The backend owns event order, queue size,
overflow, callback thread, and any platform lifetime rules. The facade does not create an event loop
or deliver callbacks on its own.

The `ios` module defines `IosApplicationBackend` and a borrowed `Extension` view. Its associated
`NativeApplication` type permits an iOS backend to expose a native handle without placing Apple
types in the portable `Application<B>` API. D1 ships no implementation of that trait.

```rust
use framework_app::{Application, ApplicationBackend};

fn inspect<B: ApplicationBackend>(app: &mut Application<B>) {
    let availability = app.availability();
    let state = app.state();
    let event = app.poll_event();
    let _ = (availability, state, event);
}
```

Errors use `framework_core::Error` and preserve the portable category plus an optional signed
platform code. The portable crate has no platform availability probe; the supplied backend reports
`Availability`.

## Sandbox files and directories

`framework-files` exposes `AppDirectory`, validated `AppPath`, `Files<B>`, and `FileBackend`. An
`AppPath` is a UTF-8, slash-separated path under one semantic app directory. It rejects an empty or
absolute path, empty/dot/parent segments, backslashes, and NUL bytes. It does not resolve symlinks,
normalize Unicode, or prove sandbox containment; every backend must enforce containment under its
selected app directory, including its symlink policy. The facade does not return absolute host
paths.

`create_directory` creates one path and does not imply parent creation. `remove_file` only removes a
file; `remove_directory` only removes an empty directory and is not recursive. `read_directory`
returns owned names; entry order is unspecified. The semantic roots are
`Documents`, `Caches`, `Temporary`, and `ApplicationSupport`; a backend defines each root's native
mapping and must document platform backup/eviction policy before making such claims.

`read` and `read_directory` return caller-owned `Vec` storage. A platform adapter may need to copy
native data into those vectors; the facade adds no copy after the backend returns them. `write`
borrows the caller's bytes only for the synchronous call. A backend may copy them but must not keep
the borrow. `read_string` transfers the returned byte vector into a `String` when UTF-8 is valid;
it does not make a second facade-level byte copy. `write_string` passes the UTF-8 bytes as a borrow.
Invalid UTF-8 maps to `FileError::InvalidUtf8`.

Each `write` names its create/replace mode and its atomicity requirement. If `RequireAtomic` is set,
the backend must return `Unsupported` before mutation when it cannot prevent readers from seeing
partial new bytes. A successful write is visible to a later read through the same backend unless
another writer changes the file. `WriteOutcome` reports the backend's atomicity result; atomicity
does not imply crash durability. A failed `AllowNonAtomic` write may have changed the file. No
backend may report stronger guarantees than it has verified.

```rust
extern crate alloc;

use framework_files::{AppDirectory, AppPath, FileBackend, FileError, Files};

fn read<B: FileBackend>(files: &mut Files<B>) -> Result<alloc::vec::Vec<u8>, FileError> {
    files.read(AppPath::new(AppDirectory::Documents, "notes/today.txt")?)
}
```

File methods are synchronous and may block. The contract defines no cancellation operation. An
adapter must state its threading and blocking behavior. `FileError` maps invalid paths and invalid
UTF-8 to `InvalidInput`; backend errors retain their `ErrorKind` and optional platform code.

## Preferences

`framework-preferences` exposes `PreferenceKey`, `Preferences<B>`, and `PreferencesBackend` for
non-secure values. Keys are exact, case-sensitive UTF-8 strings; the facade rejects only empty
keys and NUL bytes and performs no normalization or prefixing. Secure credentials belong in a
separate secure-storage capability, not this crate.

The backend stores opaque byte values. `get_bytes` transfers an owned byte vector to the caller;
`get_string` transfers that vector into a UTF-8 `String` without a second facade-level byte copy.
`set_bytes` and `set_string` borrow data only for the synchronous call. A backend may copy the value
but must not retain its borrow. A successful update is visible to a later get through the same
backend unless another writer changes that key. Each update affects one key only; the API offers no
multi-key transaction and no crash-durability promise. A backend reports one-key atomic visibility
through `WriteOutcome`; `RequireAtomic` must fail with `Unsupported` before mutation if the backend
cannot provide it. `Atomic` is not a durability claim.

```rust
use framework_preferences::{
    AtomicityRequirement, PreferenceKey, Preferences, PreferencesBackend, WriteOptions,
};

fn save<B: PreferencesBackend>(prefs: &mut Preferences<B>) {
    let key = PreferenceKey::new("theme").expect("static key is valid");
    let options = WriteOptions::new(AtomicityRequirement::AllowNonAtomic);
    let _ = prefs.set_string(key, "dark", options);
}
```

The API defines no permission prompt, native store, observer, or automatic synchronization call.
`PreferenceError` preserves stable categories and optional backend codes. Invalid UTF-8 is an
`InvalidInput` error.

## Foreground HTTP values

`framework-network` exposes `HttpUrl`, `HttpMethod`, borrowed `Header` and `HttpRequest`, owned
`ResponseHeader` and `HttpResponse`, `HttpBackend`, and `HttpClient<B>`. The URL check requires a
case-insensitive `http://` or `https://` scheme, a non-empty authority, and no ASCII whitespace/control bytes. It is not a
full URL parser, canonicalizer, percent decoder, or name resolver. Request header names use an
ASCII token form; field values remain opaque bytes except for forbidden control bytes. Duplicate
headers and caller order are preserved. Status values are fixed-width `u16` values from 100 through
599; every valid status, including non-success status, is returned as an ordinary response.

Requests borrow their method, URL, header slice, header bytes, and optional body. The facade makes
no request copy; a backend may copy or transcode at its native transport boundary and must document
that cost. Responses own `Vec` storage for headers and body. Ownership moves from the backend to
the caller; the backend may incur the native-to-Rust copy needed to create those vectors. UTF-8
views borrow without a copy; `into_body_string` transfers the body vector into a `String`.

The backend returns a concrete associated future type, so no boxed `dyn` transport or executor is
required. The operation starts when the facade's future is first polled. The facade adds no retry,
cookie jar, redirect policy, or background-transfer mode. Dropping its future drops the
caller-side interest and backend future; whether the OS request is cancelled or detached is a
backend-specific contract and is not guaranteed here. Backend docs must state completion, callback
thread, reentrancy, and cancellation behavior. D1 adds no transport; the integrated workspace's
separate `ios-network` backend implements `HttpBackend` with Foundation `URLSession`. A request
needs an explicitly supplied backend and cannot be sent by `framework-network` alone.

```rust
use framework_network::{Header, HttpBackend, HttpClient, HttpMethod, HttpRequest, HttpUrl};

async fn get<B: HttpBackend>(client: &mut HttpClient<B>) {
    let method = HttpMethod::new("GET").expect("static method is valid");
    let url = HttpUrl::new("https://example.test/").expect("static URL is valid");
    let headers: [Header<'_>; 0] = [];
    let request = HttpRequest::new(method, url, &headers, None);
    let _ = client.send(request).await;
}
```

`NetworkError` uses stable portable categories and optional backend codes. HTTP status is not
converted to an error. D1 makes no platform metadata claims for its generic contract. The canonical
manifest records verified metadata for the separate `ios-network` backend; no runtime request,
parity result, or performance result is claimed there.

## Validation boundary

The four crate contracts are independently buildable with `--no-default-features`. Unit tests use
small in-memory backends only to verify portable path/key/value/request semantics. They do not
validate an Apple backend, persistence behavior, native permissions, network behavior, iOS parity,
or performance. No `.swift` source, native platform type, global runtime, or third-party dependency
is added by D1.
