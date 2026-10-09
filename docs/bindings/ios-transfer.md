# iOS transfer C ABI

## Opt-in surface

The `framework-c-api` crate keeps transfer calls out of its default build. Enable Cargo feature
`ios-transfer` and include
[`framework_ios_transfer.h`](../../bindings/c/include/framework_ios_transfer.h) beside
[`framework.h`](../../bindings/c/include/framework.h). The feature adds optional edges to the
current `framework-core`, `framework-files`, `framework-network`, `framework-transfer`,
`ios-transfer`, and `objc2` crates. The `objc2` edge exists only for the iOS target and supplies
`MainThreadMarker`

This C surface is iOS-only. On other targets, status-returning calls with required pointer
arguments present return `FRAMEWORK_STATUS_UNSUPPORTED` after initializing required outputs. Null
required pointers return `FRAMEWORK_STATUS_INVALID_ARGUMENT`; client creation and event forwarding
also validate their session-identifier spans. Request contents are not parsed before the unsupported
return. Destroy functions retain their null no-op behavior. No mock backend is created. Rust apps
should use `Transfers<IosTransferBackend>` and the D10 API directly

## Client and request

Create one `FrameworkIosTransferClient` for a stable, app-unique background-session ID. B13 checks
the ID syntax and copies it. The handle owns one `IosTransferBackend`; it is not a process-global
registry. Create and every operation call with that handle must run on the main thread; off-main
client destroy is a no-op and leaves the slot unchanged. The `out_client` slot must not hold a live
client; a non-null slot is set to null before input validation. Serialize calls per client.
Synchronous journal and file work may pause that thread

Each `FrameworkTransferRequestV1` is a GET request with a nonzero app-assigned 128-bit ID, an
HTTP(S) URL, ordered headers, and a path under one semantic application directory. Persist each
ID in the app across relaunch. Request spans must stay valid and unchanged through the call; B13
copies accepted data before return. Set every zero-length span to `{NULL, 0}`. Set `headers` to
NULL iff `header_count` is zero. Request header names follow D1 token rules and values follow D1
field-value rules; B13 may reject values that are not UTF-8

Set `struct_size` to `sizeof(FrameworkTransferRequestV1)`, `abi_version` to the ABI major from
`framework_abi_version()`, and `reserved` to zero. Rust reads only the first two fields before its
size/version check. The request record and non-empty header array must meet their C type's alignment.
A wrong size, major, reserved value, zero ID, unknown directory, invalid span, URL, header, or path
returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`

```c
FrameworkIosTransferClient *client = NULL;
FrameworkTransferRequestV1 request = {0};
FrameworkTransferIdV1 id = {UINT64_C(0x1234), UINT64_C(0x5678)};
FrameworkStr url = {(const uint8_t *)"https://example.com/archive", 27};
FrameworkStr path = {(const uint8_t *)"downloads/archive.zip", 21};
int32_t native_code = 0;

request.struct_size = (uint32_t)sizeof(request);
request.abi_version = (uint32_t)(framework_abi_version() >> 32);
request.id = id;
request.url = url;
request.directory = FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT;
request.relative_path = path;

FrameworkStatus status = framework_ios_transfer_client_create(
    (FrameworkStr){(const uint8_t *)"com.example.transfers", 21},
    &client,
    &native_code);
if (status == FRAMEWORK_STATUS_OK) {
    status = framework_ios_transfer_start_download(client, &request, &native_code);
}
```

`TransferId` maps to two `uint64_t` fields: `high` carries bits 127..64 and `low` carries bits
63..0. All-zero is invalid. The C tags for availability, directories, states, and error kinds are
fixed in the header; Rust does not cast enum layouts across C

## Query, cancel, and forget

`framework_ios_transfer_status` returns `FRAMEWORK_STATUS_OK` with `out_found == 0` when no record
has the ID. A present record returns an owned snapshot with `out_found == 1`. A snapshot may
outlive its client. `out_snapshot` must not hold a live snapshot on entry; status sets each non-null
required output to its empty value before work. `framework_ios_transfer_snapshot_get_view` overwrites the full output record
without reading its prior bytes; the view has the exact size and ABI major, zero reserved fields,
and zero semantic fields on error. Its `http_status` and `response_header_count` apply only to
Succeeded. Its `failure_kind` and `native_code` apply only to Failed. Indexed header spans borrow
from the snapshot until `framework_ios_transfer_snapshot_destroy`; the header output is set to empty
spans before work

Destroy each original client or snapshot pointer slot once. Each destroy clears the slot before
drop. Do not copy a live handle, destroy an alias, race a call with destroy, or use a header view
after snapshot destroy. All non-null output pointers must meet their C type's alignment and permit
writes; keep outputs distinct from each other, input spans, request records, header arrays, and live
handle storage. An off-main client-destroy call is a no-op and leaves the slot unchanged. Keep the
client alive until every accepted background-event callback runs
Client destroy does not cancel or forget durable tasks; snapshot destroy does not affect a task

`cancel` records a durable stop request; it does not prove that task work has stopped. `forget`
removes a terminal record but leaves the destination file. Success means durable task acceptance,
not immediate network activity. A non-2xx HTTP response remains Succeeded; the response body stays
at the requested path and C exposes no body bytes

## UIKit event lifecycle

The app owns its `UIApplicationDelegate`. In
`application:handleEventsForBackgroundURLSession:completionHandler:`, match the exact session ID
to its live client, then call `framework_ios_transfer_forward_background_events`. Host glue keeps
UIKit's completion block in `context`; the C callback invokes it when B13 calls back. A successful
call retains that callback and context until URLSession completes the event batch and first task
reconciliation ends. B13 calls the callback once on the main dispatch queue, after the C call
returns. Keep the context and client alive until then. The completion function must be non-null and
must not unwind into Rust

If UIKit starts the app with no background-session event, call
`framework_ios_transfer_finish_launch_without_background_events` once after UIKit confirms that no
background-session event applies. Do not call both launch paths for one launch. A rejected event-forward call takes no
callback or context ownership; host glue must resolve UIKit's completion handler itself. B13 can
reject a second callback while one remains stored. A later event batch may arrive after B13 queues
an earlier callback but before it runs, so keep each accepted context and the client alive through
its callback

## Errors and limits

`DuplicateId` maps to `FRAMEWORK_STATUS_ALREADY_EXISTS`, `NotFound` to
`FRAMEWORK_STATUS_NOT_FOUND`, and `NotTerminal` to `FRAMEWORK_STATUS_INVALID_ARGUMENT`. A
session-ID mismatch or a second launch classification maps to `FRAMEWORK_STATUS_INVALID_ARGUMENT`;
a stored completion callback maps to `FRAMEWORK_STATUS_ALREADY_EXISTS`. Backend errors use the
portable `FrameworkStatus` map and retain an optional native code. Unknown Rust variants use the
plan's explicit fallback tags or `FRAMEWORK_STATUS_INTERNAL_ERROR`. Optional native-code outputs
are zero before work and retain a nonzero code only for a platform error

Every export with a status return catches Rust panics and returns `FRAMEWORK_STATUS_PANIC`. No panic
may unwind across C. The ABI cannot prove that an arbitrary C pointer is mapped or that C code
keeps a handle alive. The API adds no upload, request body, progress, retry policy, redirect
override, executor, permission call, app-delegate replacement, or force-quit continuation claim
Atomic destination commit and path containment remain B13/B14 duties. F5 does not read a full
response body into a C or Rust byte buffer

`sh bindings/c/check-ios-transfer.sh` checks feature isolation, C11/C++17 declarations, ABI
layouts, exact header and host-archive symbol parity with the manifest, host C/C++ links to stub
exports, and iOS device/simulator compile and link data. It links a C probe for each iOS target;
`otool -L` requires exact direct imports `CoreFoundation`,
`Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`. C++ fixture links also allow
`libc++.1.dylib`; `nm -u` scans C/C++ probe outputs, not the static archives, for Swift/Python runtime
and unrelated capability symbols. The script does not execute probes; this evidence applies to those probe links only
The script does not execute host or iOS probes, route UIKit events, prove URLSession delivery, or
establish device runtime
behavior
