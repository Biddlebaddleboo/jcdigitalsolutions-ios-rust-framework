# PLAN_BINDINGS_SHARE.md — Workstream F7: iOS Share C ABI

## Status

F7 C ABI source, header, ABI manifest entry, guide, and compile/link-only check script are integrated
in the local tree. An earlier full `sh bindings/c/check-ios-share.sh` gate passed after the
direct-import manifest update on macOS. It checks feature isolation, manifest-derived tag/layout assertions,
symbol parity across the manifest, public header, and iOS archives, exact C/C++ probe imports, and
forbidden symbols in linked probes and archives. It did not run tests or probe binaries. The manual
ownership audit found and corrected manifest and guide wording that could imply every accepted start
produces a callback. B7's `IosShareSession` serializes callback state and UIKit cleanup on the main
queue; if UIKit reports a terminal result, it sends one callback after `start` returns. UIKit has
no presentation-error callback, so accepted start does not guarantee a terminal callback. The gate
now requires the manifest's inactive-cancel status mapping. D6 remains unchanged; F7 does not wrap
`ShareFuture` because that type holds a borrow of its backend

Repo evidence lists Xcode 26.6 and iPhoneOS/iPhoneSimulator SDK 26.5, below the Xcode 27.x baseline; no F7 pass on 27.x is in the log

Integrated main run [38044959797](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38044959797) passed macOS Swift ABI229, Apple Clippy232, C ABI234, C++17 header235, secure-storage236, notification-response237, transfer238, and clipboard239; Linux Clippy231 was skipped. Share C ABI step240 failed immediately after device and simulator release builds. The script suppresses `nm` stderr, so the exact failing invocation is not printed. Attribution to Xcode 16.4 Apple `nm` failing to inspect Rust LLVM 21 archives is strongly supported by step236 in the same run, where `nm` reported `Unknown attribute kind (102)` (Producer `LLVM21.1.8-rust-1.94.1-stable`, Reader `LLVM APPLE_1_1700.0.13.5_0`), but remains an inference for step240. The script now uses Rust's matching `llvm-nm` for device/simulator archive import and export-symbol scans while retaining Apple `nm` for final Mach-O probes and preserving all symbol/import assertions. Static syntax and diff checks only; step240 has not been rerun. No runtime share UI or device behavior is claimed

## Objective

Expose D6 text and URL share requests through an opt-in iOS C ABI backed by B7 `IosShareSession`. Use one explicit session handle, one active share operation, and a host callback. Add no C executor, global registry, or second UIKit backend

## Dependencies and required B7 seam

- F1 core C ABI and D6 `framework-sharing` are integrated
- B7's public `IosShareSession` is integrated and provides the callback contract below
- F6 provides the `ios-sharing/share` feature gate and leaves it off the F7 C dependency's default path
- Add optional edges only to current `framework-core`, `framework-sharing`, `ios-sharing`, `ios-runtime`, and workspace-pinned `objc2`, `objc2-ui-kit`, and `objc2-core-foundation` crates for UIKit context conversion
- Add no crate, version, or workspace pin
- Do not change D6's future, `ShareBackend`, or portable ownership/cancel contract
- F7 does not edit B7 source, `PLAN_BINDINGS.md`, `Cargo.lock`, shared docs indexes, shared capability manifests, or CI

F7 relies on these B7 `IosShareSession` rules

- Own one app-selected presenter, source view, anchor, and at most one active presentation
- Retain strong UIKit references after session create; use a typed `MainThread` proof and remain `!Send`/`!Sync`
- Accept an owned D6 `ShareRequest` and a one-shot callback used only if UIKit reports a terminal result; do not expose a borrowed `ShareFuture`
- Proposed Rust shape: `IosShareSession::new(MainThread, Retained<UIViewController>, Retained<UIView>, CGRect)`, `start<F>(ShareRequest, F) -> Result<(), ShareStartError<F>> where F: FnOnce(Result<ShareOutcome, ShareError>) + 'static`, and `cancel() -> bool`; synchronous rejection returns the callback in `ShareStartError<F>`
- Start UIKit work on `start` call on the main thread; return preflight errors synchronously without taking callback/context ownership
- On accepted start, retain callback state until UIKit reports a terminal result or explicit cancel/destroy; if UIKit sends no result, callback/context stay live until cancel or destroy
- If UIKit reports a terminal result, invoke one callback on main after `start` returns; clear session operation state before a later start can succeed
- Serialize cancel, destroy, UIKit completion, and callback on the main thread; suppress late completion after cancel or destroy
- Cancel or destroy detaches the result callback but does not promise to dismiss share UI already on screen
- Add no task executor, process-global map, or delegate replacement

## F7 write scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs`
- `bindings/c/src/ios_share.rs`
- `bindings/c/include/framework_ios_share.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-share.sh`
- `docs/bindings/ios-share.md`
- `PLAN_BINDINGS_SHARE.md` for scoped implementation and CI status updates

The orchestrator owns `Cargo.lock`, root workspace membership, `PLAN_BINDINGS.md`, shared docs indexes, shared capability manifests, and CI. F7 must not edit D6, B7, F6, `framework.h`, C++, Swift, or B15 paths

## Feature and target rules

- Cargo feature name `ios-share`
- `framework-c-api` sets `default-features = false` on `ios-sharing` and enables only `ios-sharing/share`
- Proposed C manifest edge: `ios-sharing = { path = "../../platform/ios/ios-sharing", version = "0.1.0", optional = true, default-features = false }`
- Proposed feature value: `ios-share = ["dep:framework-core", "dep:framework-sharing", "dep:ios-runtime", "dep:ios-sharing", "ios-sharing/share", "dep:objc2", "dep:objc2-ui-kit", "dep:objc2-core-foundation"]`
- Keep `ios-share` off default and non-iOS dependency paths
- Non-iOS builds expose explicit unsupported stubs after required output init and input shape checks; valid operations return unsupported, while null-safe destroy returns `FRAMEWORK_STATUS_OK` for a null slot or null handle; no mock session is created
- `ios-sharing` defaults both `clipboard` and `share` on for current Rust consumers; F7's C graph must not enable `ios-sharing/clipboard` or UIKit `UIPasteboard`
- Check the C feature graph for `ios-sharing/share`, `UIActivityViewController`, and share-only `block2`; check absence of `ios-sharing/clipboard` and `UIPasteboard`
- Do not add an Apple framework link beyond the current B7 requirements

## C header and symbols

`framework_ios_share.h` includes `framework.h` and uses fixed-width tags and records

```c
typedef struct FrameworkIosShareSession FrameworkIosShareSession;

#define FRAMEWORK_IOS_SHARE_ITEM_TEXT UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_ITEM_URL UINT32_C(1)
#define FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED UINT32_C(1)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef void (*FrameworkIosShareCompletion)(
    void *context,
    FrameworkStatus status,
    uint32_t outcome,
    int32_t native_code);

typedef struct FrameworkIosShareItemV1 {
    uint32_t kind;
    uint32_t reserved;
    FrameworkStr text;
} FrameworkIosShareItemV1;

typedef struct FrameworkIosShareRequestV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    const FrameworkIosShareItemV1 *items;
    uint64_t item_count;
} FrameworkIosShareRequestV1;

typedef struct FrameworkIosShareAnchorV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    double x;
    double y;
    double width;
    double height;
} FrameworkIosShareAnchorV1;

FrameworkStatus framework_ios_share_session_create(
    void *presenter,
    void *source_view,
    FrameworkIosShareAnchorV1 anchor,
    FrameworkIosShareSession **out_session);

FrameworkStatus framework_ios_share_session_availability(
    const FrameworkIosShareSession *session,
    uint32_t *out_availability);

FrameworkStatus framework_ios_share_start(
    FrameworkIosShareSession *session,
    const FrameworkIosShareRequestV1 *request,
    FrameworkIosShareCompletion completion,
    void *context);

FrameworkStatus framework_ios_share_cancel(
    FrameworkIosShareSession *session);

FrameworkStatus framework_ios_share_session_destroy(
    FrameworkIosShareSession **session);
```

## Request and result map

- `FRAMEWORK_IOS_SHARE_ITEM_TEXT` maps to `ShareItem::Text`; `FRAMEWORK_IOS_SHARE_ITEM_URL` maps to `ShareItem::Url`
- `FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED` maps to `ShareOutcome::Completed`; `FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED` maps to `ShareOutcome::Dismissed`; neither means recipient delivery
- `FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN` is the only normal availability result; B7 does not claim that a presenter stays ready
- On 64-bit iOS, `FrameworkIosShareItemV1` is 24 bytes with 8-byte alignment, `FrameworkIosShareRequestV1` is 24 bytes with 8-byte alignment, and `FrameworkIosShareAnchorV1` is 40 bytes with 8-byte alignment
- Request and anchor inputs require exact `struct_size` and ABI major checks before full read; all reserved fields are zero
- Request has at least one item; `item_count` must fit `usize`; `items` is null iff count is zero, then the empty request is rejected
- Each item uses valid UTF-8; zero-length text is `{NULL, 0}`; text and URL order are retained; URL text is passed through as D6 defines, while B7 may reject or normalize it for UIKit
- F7 copies all item text into an owned D6 `ShareRequest` before `framework_ios_share_start` returns
- A session accepts one active operation; a second start returns `FRAMEWORK_STATUS_ALREADY_EXISTS` without callback ownership
- `framework_ios_share_cancel` returns `FRAMEWORK_STATUS_NOT_FOUND` when no accepted operation remains active, including after a terminal callback or successful prior cancel
- A preflight or input error returns its `FrameworkStatus` from `start` and does not call the C completion
- If UIKit calls its completion with `completed == true` and no `NSError`, the C completion runs once with `FRAMEWORK_STATUS_OK`, `FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED`, and native code zero; if UIKit calls it with `completed == false` and no `NSError`, the callback runs once with `FRAMEWORK_STATUS_OK`, `FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED`, and native code zero
- Map known `ShareError::Backend` values through `FrameworkStatus::from_error`; unknown non-exhaustive errors map to `FRAMEWORK_STATUS_INTERNAL_ERROR`; set `outcome` to zero and preserve a nonzero `NSError` code only when it fits `int32_t`
- Unknown non-exhaustive D6 result or error variants map to `FRAMEWORK_STATUS_INTERNAL_ERROR`; `native_code` is zero when no valid code exists

## Session ownership, callback, cancel, and destroy

- `framework_ios_share_session_create` sets `*out_session` to null before validation and requires `MainThread::current()`; its output slot must be naturally aligned, writable, and distinct from inputs and live handle storage
- `framework_ios_share_session_availability` writes the Unknown tag before validation and returns that tag on a valid session; its output slot must be naturally aligned and writable, and must not alias the live handle
- The host passes live Objective-C `UIViewController *` and `UIView *` addresses as `void *`; each address must be a valid object of the exact class
- F7 retains both UIKit objects after successful create; the host keeps each input live through that call
- B7 session owns the retained presenter/source, anchor, main-thread proof, and at most one accepted operation; no process-global lookup exists
- Every status function checks `MainThread::current()` before it reads a C session handle; off-main calls return `FRAMEWORK_STATUS_UNAVAILABLE`
- The source rectangle uses the source-view coordinate system; B7 requires the presenter and source view to belong to the same window and validates presentation context before UI start
- Completion function must be non-null and must not unwind into Rust or call F7 again before the callback returns
- A rejected start calls no callback and takes no callback or context ownership; an accepted start keeps callback and context live until one terminal callback, successful cancel, or session destroy
- The C host keeps `context` live until the terminal callback, successful cancel, or session destroy; B7 does not retain or free host memory
- If UIKit reports a terminal result, callback runs once on the main thread; completion state is detached before callback return. UIKit may send no result; callback/context then stay live until successful cancel or session destroy. No callback follows cancel or destroy
- `framework_ios_share_cancel` succeeds only while one operation is active; after completion or prior cancel it returns `FRAMEWORK_STATUS_NOT_FOUND`
- Cancel detaches the B7 result path and suppresses callback; UIKit share UI may remain visible
- `framework_ios_share_session_destroy` returns `FRAMEWORK_STATUS_UNAVAILABLE` off-main without reading or changing the session slot; the handle remains live so the host can retry on main
- On main, destroy cancels an active operation, clears the original handle slot before drop, accepts a null slot or null handle as `FRAMEWORK_STATUS_OK`, and returns `FRAMEWORK_STATUS_OK` after destruction
- The session-slot pointer must be naturally aligned and writable for the call; the handle must be the unique live handle returned by create. Do not copy a live session handle, destroy an alias, race calls with destroy, or start a new operation from inside its callback

## UIKit and `NSError` rules

- No UIKit class type appears in the C header; `void *` context inputs carry the documented exact Objective-C class precondition
- F7 contains raw ObjC address conversion and retain calls in one iOS-only module; required addresses must be live, aligned Objective-C objects for the full call
- B7 uses `UIActivityViewController`; availability is `Unknown` and does not claim a presenter is ready
- B7 preflight `ErrorKind::Unavailable` maps to `FRAMEWORK_STATUS_UNAVAILABLE`; wrong-window or invalid URL errors map to `FRAMEWORK_STATUS_INVALID_ARGUMENT`
- UIKit completion with `completed == true` and no `NSError` maps to `Completed`; `completed == false` and no `NSError` maps to `Dismissed`
- Non-null `NSError` maps to `ErrorKind::Platform`; preserve a nonzero `NSError.code` only when it fits signed 32-bit range
- UIKit presentation has no error callback for an unforeseen refusal or race; F7 makes no presentation-success guarantee beyond the B7 result contract
- All create, start, cancel, destroy, and callback work stays on the main thread; serialize one session and do not re-enter the C API from its callback
- No UIKit delegate replacement, permission API, global registry, task executor, detached future, or force-quit behavior

## Non-test validation and handoff

- Check default isolation and the F7 feature graph with `cargo tree --locked -e features`; confirm `ios-sharing/share` is opt-in and `ios-sharing/clipboard` plus `UIPasteboard` are absent
- Check non-iOS C stubs with host `cargo tree` and `cargo check --locked -p framework-c-api --features ios-share`
- Run iOS device and simulator `cargo check`, Clippy with `-D warnings`, and Release builds with `--locked`
- Compile and link C11 and C++17 header fixtures for host stubs, iOS device, and simulator where the toolchain permits; do not run fixtures
- Use manifest-derived C11/C++17 checks for item, outcome, and availability tags plus every 64-bit record size, alignment, and field offset; use `jq -e` for required F7 symbols, callback/status map, and session ownership records; compare the manifest symbol list with the public header and each iOS archive
- Compare ownership semantics with source and public header by manual review; structural `jq` checks require the ownership records but do not compare their prose
- Compare linked C/C++ device and simulator probes with the manifest's exact `otool -L` import sets; run `nm -u` forbidden-symbol checks on each linked probe and archive for clipboard UIKit features, Swift, Python, and unrelated Apple frameworks
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, JSON and shell syntax checks, and scoped `git diff --check`
- Add or run no tests in F7; report compile/link evidence only, not live share UI or recipient behavior

## Orchestrator handoff

- Link this plan from `PLAN_BINDINGS.md` after root review
- Start F7 only after B7 integrates and documents `IosShareSession` with every required rule above
- Reconcile `Cargo.lock`, docs indexes, capability manifest, and CI outside the F7 executor scope
