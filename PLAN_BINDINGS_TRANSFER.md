# PLAN_BINDINGS_TRANSFER.md — Workstream F5: iOS Transfer C ABI

## Status

F5's C facade is present in the local tree. The full `sh bindings/c/check-ios-transfer.sh` gate passes after the linked-import and header-symbol checks were added. It checks host feature isolation, iOS device/simulator compile and Clippy, manifest-derived C11/C++17 tags and layouts, exact header/archive symbols, and C/C++ links for both targets

The linked C probes import exactly `CoreFoundation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imports `libc++.1.dylib`. `otool -L` checks linked probes, and `nm -u` scans linked probes for Swift/Python runtime and unrelated capability symbols. Probe binaries were not executed. This does not prove live URLSession event delivery or runtime behavior. No tests were run

## Objective

Expose D10's durable GET download API through an opt-in C ABI backed only by B13's concrete `IosTransferBackend`. Keep the C API iOS-specific; do not claim a portable C backend

F5 now has an implemented B13 backend and per-client host-app event-forward seam. Implement only the C facade in this workstream; do not fake a backend or add C symbols that cannot call the real backend

## Dependencies and gates

- F1 core C ABI is integrated
- D1 `framework-network` and `framework-files`, D10 `framework-transfer`, and B14 `ios-files` file adoption are integrated
- B13 `ios-transfer` has a concrete, statically selected `IosTransferBackend` with a public constructor, stable session ID, durable task state, and defined drop/relaunch semantics
- B13 provides a public constructor, an explicit event-forward call for a C-owned client, and `finish_launch_without_background_events` for ordinary startup. F5 does not edit B13
- The app owns its `UIApplicationDelegate`; framework code does not replace it or use a global session registry

The host app passes its UIKit background-session event to the exact client handle; the client matches the session ID and retains one completion callback until URLSession reports that all session events are done. On ordinary startup, the host calls `framework_ios_transfer_finish_launch_without_background_events` once after UIKit routing rules out a background-session event

## Implemented source scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs`
- `bindings/c/src/ios_transfer.rs`
- `bindings/c/include/framework_ios_transfer.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-transfer.sh`
- `docs/bindings/ios-transfer.md`
- `docs/DOCUMENTATION.md`
- `docs/VALIDATION.md`
- `.github/workflows/ci.yml`

The orchestrator owns `Cargo.lock`, root workspace membership, shared capability manifests, and any B13 source or plan edits. Do not edit `bindings/cpp/**`, `crates/framework-transfer/**`, `platform/ios/ios-transfer/**`, B14, B12, or the core `framework.h` ABI from F5

## Optional feature and target scope

- Cargo feature name `ios-transfer`
- Optional path dependency on the B13 `ios-transfer` crate
- Optional direct edges to existing `framework-transfer`, `framework-network`, `framework-files`, and `framework-core` packages because B13's public trait requires their request, path, status, and availability/error types; do not create a parallel transfer implementation
- Optional iOS-only direct edge to the already workspace-pinned `objc2` crate for `MainThreadMarker`; add no new crate/version or root workspace dependency pin. Keep all transfer dependencies off the default build path
- The default `framework-c-api` build must not include `framework-transfer` or `ios-transfer`
- Add no new third-party package or workspace dependency pin
- The feature is supported only on iOS device and simulator targets. Other targets may expose explicit `FRAMEWORK_STATUS_UNSUPPORTED` stubs; they must not create a mock or weaker backend
- Keep all client state in an explicit opaque handle. No process-global state, trait object, executor, callback table, or second transfer implementation

## C value map

The separate `framework_ios_transfer.h` includes `framework.h` and uses fixed-width values. Do not expose Rust enum layout

| C value | C tag | D10 value |
| --- | ---: | --- |
| `FrameworkTransferIdV1` | two `uint64_t` fields | `TransferId(u128)`; `high` is bits 127..64 and `low` is bits 63..0; all-zero is invalid |
| `FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN` | 0 | `Availability::Unknown` |
| `FRAMEWORK_TRANSFER_AVAILABILITY_AVAILABLE` | 1 | `Availability::Available` |
| `FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED` | 2 | `Availability::Unsupported` |
| `FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_PERMISSION` | 3 | `Availability::RequiresPermission` |
| `FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_ENTITLEMENT` | 4 | `Availability::RequiresEntitlement` |
| `FRAMEWORK_TRANSFER_AVAILABILITY_TEMPORARILY_UNAVAILABLE` | 5 | `Availability::TemporarilyUnavailable` |
| `FRAMEWORK_TRANSFER_DIRECTORY_DOCUMENTS` | 0 | `AppDirectory::Documents` |
| `FRAMEWORK_TRANSFER_DIRECTORY_CACHES` | 1 | `AppDirectory::Caches` |
| `FRAMEWORK_TRANSFER_DIRECTORY_TEMPORARY` | 2 | `AppDirectory::Temporary` |
| `FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT` | 3 | `AppDirectory::ApplicationSupport` |
| `FRAMEWORK_TRANSFER_STATE_QUEUED` | 0 | `TransferStatus::Queued` |
| `FRAMEWORK_TRANSFER_STATE_ACTIVE` | 1 | `TransferStatus::Active` |
| `FRAMEWORK_TRANSFER_STATE_SUCCEEDED` | 2 | `TransferStatus::Succeeded` |
| `FRAMEWORK_TRANSFER_STATE_FAILED` | 3 | `TransferStatus::Failed` |
| `FRAMEWORK_TRANSFER_STATE_CANCELLED` | 4 | `TransferStatus::Cancelled` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN` | 0 | `ErrorKind::Unknown` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_INVALID_INPUT` | 1 | `ErrorKind::InvalidInput` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_UNSUPPORTED` | 2 | `ErrorKind::Unsupported` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_UNAVAILABLE` | 3 | `ErrorKind::Unavailable` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_PERMISSION_DENIED` | 4 | `ErrorKind::PermissionDenied` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_CANCELLED` | 5 | `ErrorKind::Cancelled` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_TIMEOUT` | 6 | `ErrorKind::Timeout` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_NOT_FOUND` | 7 | `ErrorKind::NotFound` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_ALREADY_EXISTS` | 8 | `ErrorKind::AlreadyExists` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_RESOURCE_EXHAUSTED` | 9 | `ErrorKind::ResourceExhausted` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_PLATFORM` | 10 | `ErrorKind::Platform` |
| `FRAMEWORK_TRANSFER_ERROR_KIND_INTERNAL` | 11 | `ErrorKind::Internal` |

Use fixed `uint32_t` tags for the portable `ErrorKind` values in a snapshot's failed state. Keep the categories and values explicit in the C declaration and ABI manifest; do not cast a Rust enum to C. Use `FrameworkStatus` for operation results, with an optional `int32_t` native-code output set to zero before work

The Rust enums used by F5 are `#[non_exhaustive]`. Map an unknown `Availability` to `FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN` and an unknown `ErrorKind` in a Failed snapshot to `FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN`. An unknown `TransferStatus` returns `FRAMEWORK_STATUS_INTERNAL_ERROR` with `out_view` still initialized and its semantic fields zero. An unknown `TransferError` maps through its `kind()` and optional native code; a category with no known C mapping returns `FRAMEWORK_STATUS_INTERNAL_ERROR`. An unknown `BackgroundEventError` returns `FRAMEWORK_STATUS_INTERNAL_ERROR`

## Request and response structs

Define these C11-compatible V1 records in `framework_ios_transfer.h`

```c
typedef struct FrameworkTransferIdV1 {
    uint64_t high;
    uint64_t low;
} FrameworkTransferIdV1;

typedef struct FrameworkTransferHeaderV1 {
    FrameworkStr name;
    FrameworkSlice value;
} FrameworkTransferHeaderV1;

typedef struct FrameworkTransferRequestV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkTransferIdV1 id;
    FrameworkStr url;
    const FrameworkTransferHeaderV1 *headers;
    uint64_t header_count;
    uint32_t directory;
    uint32_t reserved;
    FrameworkStr relative_path;
} FrameworkTransferRequestV1;

typedef struct FrameworkTransferSnapshotViewV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkTransferIdV1 id;
    uint32_t state;
    uint32_t failure_kind;
    int32_t native_code;
    uint32_t http_status;
    uint32_t reserved;
    uint32_t reserved2;
    uint64_t response_header_count;
} FrameworkTransferSnapshotViewV1;

typedef struct FrameworkTransferHeaderViewV1 {
    FrameworkStr name;
    FrameworkSlice value;
} FrameworkTransferHeaderViewV1;
```

`FrameworkTransferRequestV1` is C input. First read only `struct_size` and `abi_version`; require `struct_size == sizeof(FrameworkTransferRequestV1)` and `abi_version == (uint32_t)(framework_abi_version() >> 32)`. Then read the full record and require `reserved == 0`. Reject an unknown major, a short record, or nonzero reserved fields

`FrameworkTransferSnapshotViewV1` is output only. Do not read any byte of `out_view`. First write a zeroed full record, then set `struct_size` to its exact size and `abi_version` to the ABI major. On a valid snapshot, set every semantic field; keep `reserved` and `reserved2` zero. On null input, keep all semantic fields zero and return `FRAMEWORK_STATUS_INVALID_ARGUMENT`

F5 uses in-band `struct_size` and `abi_version` on `FrameworkTransferRequestV1` because C supplies data that Rust must size-check and version-check before parse. `FrameworkTransferSnapshotViewV1` has the same fields as output metadata, not input checks. F4's fixed output-only `FrameworkNotificationResponseViewV1` uses a versioned type name; no in-band size/version fields are needed

## 64-bit iOS record layouts

These sizes, alignments, and offsets apply to 64-bit iOS device and simulator ABIs with 8-byte `uint64_t` alignment

| Record | Size | Alignment | Field offsets |
| --- | ---: | ---: | --- |
| `FrameworkTransferIdV1` | 16 | 8 | `high`: 0, `low`: 8 |
| `FrameworkTransferHeaderV1` | 32 | 8 | `name`: 0, `value`: 16 |
| `FrameworkTransferRequestV1` | 80 | 8 | `struct_size`: 0, `abi_version`: 4, `id`: 8, `url`: 24, `headers`: 40, `header_count`: 48, `directory`: 56, `reserved`: 60, `relative_path`: 64 |
| `FrameworkTransferSnapshotViewV1` | 56 | 8 | `struct_size`: 0, `abi_version`: 4, `id`: 8, `state`: 24, `failure_kind`: 28, `native_code`: 32, `http_status`: 36, `reserved`: 40, `reserved2`: 44, `response_header_count`: 48 |
| `FrameworkTransferHeaderViewV1` | 32 | 8 | `name`: 0, `value`: 16 |

`FrameworkTransferRequestV1` is GET-only. `url` uses D1 `HttpUrl` checks. Request field names use D1 token rules; values are opaque bytes; order and duplicate names remain as supplied. The directory tag and relative path form D1 `AppPath`. No method override, body, redirect policy, or progress field exists

All input spans are read-only and valid through the call; all zero-length spans use `{NULL, 0}`. The `headers` array is null iff `header_count == 0`; reject an unrepresentable count or invalid pointer/count pair

`FrameworkTransferSnapshotViewV1` has one state tag. `http_status` is nonzero only for Succeeded; `failure_kind` and `native_code` are meaningful only for Failed and are zero for other states. `reserved` and `reserved2` are always zero. `response_header_count` reports the observed B13 response fields. Indexed field access returns borrowed name/value spans in observed order; B13 documents Foundation normalization and duplicate/order limits

## C API shape

Declare opaque `FrameworkIosTransferClient` and `FrameworkIosTransferSnapshot` types and these symbols

```c
typedef void (*FrameworkIosTransferEventsCompletion)(void *context);

FrameworkStatus framework_ios_transfer_client_create(
    FrameworkStr session_identifier,
    FrameworkIosTransferClient **out_client,
    int32_t *out_native_code);

void framework_ios_transfer_client_destroy(
    FrameworkIosTransferClient **client);

FrameworkStatus framework_ios_transfer_availability(
    const FrameworkIosTransferClient *client,
    uint32_t *out_availability);

FrameworkStatus framework_ios_transfer_start_download(
    FrameworkIosTransferClient *client,
    const FrameworkTransferRequestV1 *request,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_status(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    FrameworkIosTransferSnapshot **out_snapshot,
    uint8_t *out_found,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_cancel(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_forget(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_snapshot_get_view(
    const FrameworkIosTransferSnapshot *snapshot,
    FrameworkTransferSnapshotViewV1 *out_view);

FrameworkStatus framework_ios_transfer_snapshot_get_header(
    const FrameworkIosTransferSnapshot *snapshot,
    uint64_t index,
    FrameworkTransferHeaderViewV1 *out_header);

void framework_ios_transfer_snapshot_destroy(
    FrameworkIosTransferSnapshot **snapshot);

FrameworkStatus framework_ios_transfer_forward_background_events(
    FrameworkIosTransferClient *client,
    FrameworkStr session_identifier,
    FrameworkIosTransferEventsCompletion completion,
    void *context,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_finish_launch_without_background_events(
    FrameworkIosTransferClient *client,
    int32_t *out_native_code);
```

Map `out_availability` to the fixed tags above. Set all required outputs to their empty value before validation; every optional `out_native_code` is zero before work and remains zero unless a platform error has a native code. `status` sets `out_found` to zero and `*out_snapshot` to null first; an absent task returns `FRAMEWORK_STATUS_OK` with `out_found == 0`. A present task returns one snapshot handle and `out_found == 1`

## Host event forward contract

The app-owned native delegate receives UIKit's `application:handleEventsForBackgroundURLSession:completionHandler:` call. It matches the exact session ID to its live `FrameworkIosTransferClient`, then calls `framework_ios_transfer_forward_background_events` with that ID and one C completion function/context. Host glue keeps UIKit's completion block in that context and invokes it from the C completion function. Framework code does not accept an Objective-C block type, own `UIApplicationDelegate`, or route by a process-global session map

If UIKit does not provide a background-session event during launch, the app calls `framework_ios_transfer_finish_launch_without_background_events` once after launch routing rules out that event. This classifies ordinary startup so B13 can finalize missing native tasks after initial reconciliation. Either launch-path call may return `FRAMEWORK_STATUS_INVALID_ARGUMENT` when the other path already classified the launch; do not call both

On success, B13 retains the callback/context until URLSession reports that all events for the session are done and initial task reconciliation finishes; it then calls the callback once asynchronously on the main dispatch queue, after the forward call returns. The host keeps the context valid and retains the client until that call, and the C callback must not unwind into Rust. A rejected forward call does not take callback ownership; the host must complete or otherwise resolve UIKit's handler. B13 rejects a second forward while one handler remains stored; once an earlier callback is queued on main, a later event batch may be forwarded before that callback runs, so the host retains each accepted context/client until its callback. B13 documents exactly-once and callback-thread rules

## Ownership and error rules

- The app assigns each nonzero `TransferId` and saves it across relaunch; `start_download` copies all needed request data before return
- `client_create` copies the session ID; one client owns one explicit B13 backend and no global state
- `client_create`, every call with a client, and `client_destroy` run on the main thread because B13 is `!Send`/`!Sync`. Serialize calls on one client. Synchronous journal and file operations may block the main thread; all task calls are synchronous and F5 adds no operation handle or async queue
- The client handle is unique and non-copyable by contract. Use its original pointer slot for destroy; destroy clears that slot before drop and does not cancel or forget durable tasks
- Do not destroy a client until each host event callback has run. A fresh client with the same session ID must reattach to retained B13 state after relaunch
- A snapshot owns one D10 `TransferSnapshot`, may outlive its client, and retains response headers. Its views borrow from the snapshot until snapshot destroy
- Snapshot destroy clears the original pointer slot before drop. Do not copy a live handle, destroy an alias, race calls with destroy, or use a view after destroy
- C input spans last through the call only; all zero-length spans use `{NULL, 0}`. Output structs are fully overwritten before any fallible work
- Map `DuplicateId` to `FRAMEWORK_STATUS_ALREADY_EXISTS`, `NotFound` to `FRAMEWORK_STATUS_NOT_FOUND`, and `NotTerminal` to `FRAMEWORK_STATUS_INVALID_ARGUMENT`; map `BackgroundEventError::SessionIdentifierMismatch` and `BackgroundEventError::LaunchAlreadyClassified` to `FRAMEWORK_STATUS_INVALID_ARGUMENT`, and `BackgroundEventError::CompletionAlreadyPending` to `FRAMEWORK_STATUS_ALREADY_EXISTS`. Map backend `Error` with `FrameworkStatus::from_error`. Preserve a native code through the optional output. In Failed snapshots, retain the D10 error kind and optional native code
- Success means durable task acceptance, not network start. Cancel is a durable stop request, not proof that work stopped. Forget removes only a terminal record, not its destination file
- A successful non-2xx HTTP response remains Succeeded. The body stays at the requested path; C exposes no body bytes
- No panic may unwind across C. Every fallible exported call catches Rust panics and returns `FRAMEWORK_STATUS_PANIC`; destroy has no recoverable panic path

## Limits

- GET only; no upload, request body, progress, custom retry, redirect override, executor, or permission API
- No general portable backend claim. Non-iOS stubs may return `FRAMEWORK_STATUS_UNSUPPORTED` only; they must not invent weaker task or file semantics
- No claim of continuation after user force-quit or power-loss durability; follow D10 and B13 limits
- No notification, B12, foreground HTTP, C++ layer, Python runtime, native response object, or app-delegate replacement
- Atomic destination commit and file containment remain B14/B13 duties; F5 must not read a full response body into a C or Rust byte buffer

## Validation and handoff

- Check default-feature isolation and the opt-in B13 dependency edge
- Run strict C API build/lint and `cargo fmt --all -- --check`; do not add or run behavioral tests in this slice
- Compile C11 and C++17 fixture sources against the public C declarations, then link against the iOS feature library where the toolchain permits
- Compare optional C symbols in the public header and host archive, tags, layouts, and ownership records with `bindings/c/abi-manifest.json`
- Check iOS device and simulator builds. Link C and C++ probes for each target; require exact direct imports `CoreFoundation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` for C, plus `libc++.1.dylib` for C++. Use `nm -u` on linked probes only to reject Swift/Python runtime and unrelated capability imports; this proves probe link shape, not arbitrary app linkage
- Run `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Report changed paths, commit SHA, exact checks, C symbols/layouts, ownership/lifetime rules, host event-forward proof, target limits, deviations, and open B13 assumptions
