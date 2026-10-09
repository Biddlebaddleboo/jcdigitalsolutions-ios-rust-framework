# PLAN_BINDINGS_CLIPBOARD.md — Workstream F6: iOS Clipboard C ABI

## Status

F6 is implemented locally as an opt-in plain-text clipboard C ABI. `sh bindings/c/check-ios-clipboard.sh` passes its locked host, device, and simulator checks; strict Clippy; C11/C++17 static-library links; feature-isolation checks; and symbol checks. The linked device and simulator C consumers import exactly `Foundation`, `UIKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imports `libc++.1.dylib`. The script does not execute consumers. A manual audit found the manifest ownership, thread, status, and acceptance semantics consistent with the C source, header, guide, and D5/B6 contracts. No tests were added or run, and no live pasteboard or privacy UI behavior is claimed

Repo evidence lists Xcode 26.6 and iPhoneOS/iPhoneSimulator SDK 26.5, below the Xcode 27.x baseline; no F6 pass on 27.x is in the log

Strict all-target Clippy also exposed a redundant closure in the existing B7 share-conversion test adapter; it now uses the enum variant constructor directly. This does not change share behavior, and the test was not run

D5 `framework-sharing::Clipboard` and B6 `IosClipboardBackend` are present in the current tree. F6 scope is clipboard only. The separate D6 share C ABI is integrated as F7 over B7 `IosShareSession`; F7 does not wrap D6's borrowed `ShareFuture`

## Objective

Expose D5 plain-text read, write, and clear through an opt-in iOS C ABI backed by B6. Add no rich pasteboard formats, share UI, process-global state, executor, or new backend

## Dependencies and required feature split

- F1 core C ABI, D5 `framework-sharing`, and B6 `ios-sharing` are integrated
- Add optional edges to current `framework-core`, `framework-sharing`, `ios-sharing`, and `ios-runtime` packages only
- Add no package, crate version, or workspace pin
- `ios-sharing` currently builds both clipboard and share modules as one unit; F6 requires a source and dependency feature split before the C feature can build clipboard code without B7 share code
- Add `clipboard` and `share` features to `ios-sharing`; keep both in its default feature set to preserve current downstream behavior
- Gate clipboard modules and UIKit features behind `ios-sharing/clipboard`; gate share modules, `block2`, and share-only UIKit/Foundation features behind `ios-sharing/share`
- The `framework-c-api` optional `ios-sharing` dependency uses `default-features = false`; `ios-clipboard` enables only `ios-sharing/clipboard`
- Keep all F6 dependencies off default and non-iOS paths

## Write scope for F6

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs`
- `bindings/c/src/ios_clipboard.rs`
- `bindings/c/include/framework_ios_clipboard.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-clipboard.sh`
- `platform/ios/ios-sharing/Cargo.toml`
- `platform/ios/ios-sharing/src/lib.rs`
- `platform/ios/ios-sharing/src/share_conversion.rs` (test-only redundant-closure lint fix; no behavior change)
- `docs/bindings/ios-clipboard.md`

The orchestrator owns `Cargo.lock`, root workspace membership, CI, `PLAN_BINDINGS.md`, shared docs indexes, and shared capability manifests. F6 must not edit D5 symbols or semantics, B6 clipboard behavior, D6 share behavior, `framework.h`, C++, Swift, or B15 paths

## `ios-sharing` feature map

`platform/ios/ios-sharing/Cargo.toml` adds `default = ["clipboard", "share"]` to preserve current downstream behavior

- `clipboard` enables `platform`, `conversion`, `operation`, and the `UIPasteboard` UIKit feature
- `share` enables `share_platform`, `share_conversion`, `share_operation`, `block2`, the `UIActivityViewController`/popover UIKit features, and share-only Foundation and Core Foundation features
- Make target UIKit, Foundation, and Core Foundation dependencies optional where a dependency serves one feature only
- Move `objc2-ui-kit` feature flags out of its dependency declaration and into Cargo feature edges; `clipboard` enables `UIPasteboard`, while `share` enables `UIActivity`, `UIActivityViewController`, `UIDevice`, popover/presentation, view, and window features
- Make `block2` optional for `share`; make `objc2-core-foundation` optional for `share`
- Gate `objc2-foundation` features by use: clipboard enables `NSArray`, `NSData`, `NSDictionary`, and `NSString`; share enables `NSArray`, `NSError`, `NSString`, and `NSURL`
- `platform/ios/ios-sharing/src/lib.rs` gates each private module and public export with its feature
- The C API sets `default-features = false` on `ios-sharing` and activates `ios-sharing/clipboard` only
- A `cargo tree -e features` check must show no `ios-sharing/share`, `block2`, or `UIActivityViewController` feature on the C clipboard path

## Optional C feature and target behavior

- Cargo feature name `ios-clipboard`
- The feature enables optional `framework-core`, `framework-sharing`, `ios-sharing`, and `ios-runtime` edges
- The `ios-sharing` dependency uses `default-features = false`; the feature enables `ios-sharing/clipboard`
- Proposed C manifest edge: `ios-sharing = { path = "../../platform/ios/ios-sharing", version = "0.1.0", optional = true, default-features = false }`
- Proposed feature value: `ios-clipboard = ["dep:framework-core", "dep:framework-sharing", "dep:ios-sharing", "dep:ios-runtime", "ios-sharing/clipboard"]`
- On iOS device and simulator, the feature uses B6's `IosClipboardBackend`
- On other targets, create/read/write/clear initialize every non-null output and validate required output addresses and input span shape; `write` also returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` for invalid UTF-8 before it returns `FRAMEWORK_STATUS_UNSUPPORTED` for valid input; no fake backend is built
- Non-iOS availability writes `FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED` and returns `FRAMEWORK_STATUS_UNSUPPORTED`; create leaves the handle null and destroy is null-safe/no-op
- Default `framework-c-api` builds include no clipboard symbols or clipboard dependencies

## C header and symbols

`framework_ios_clipboard.h` includes `framework.h`, declares fixed-width availability tags, and uses one opaque unique `FrameworkIosClipboard` handle

```c
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef struct FrameworkIosClipboard FrameworkIosClipboard;

FrameworkStatus framework_ios_clipboard_create(
    FrameworkIosClipboard **out_clipboard);

void framework_ios_clipboard_destroy(
    FrameworkIosClipboard **clipboard);

FrameworkStatus framework_ios_clipboard_availability(
    const FrameworkIosClipboard *clipboard,
    uint32_t *out_availability);

FrameworkStatus framework_ios_clipboard_read(
    FrameworkIosClipboard *clipboard,
    uint8_t *out_has_value,
    FrameworkOwnedBuffer *out_text,
    int32_t *out_native_code);

FrameworkStatus framework_ios_clipboard_write(
    FrameworkIosClipboard *clipboard,
    FrameworkStr text,
    int32_t *out_native_code);

FrameworkStatus framework_ios_clipboard_clear(
    FrameworkIosClipboard *clipboard,
    int32_t *out_native_code);
```

## Value, status, and ownership rules

- `framework_ios_clipboard_create` initializes `*out_clipboard` to null before validation, then requires `MainThread::current()` before it creates B6 state
- Every iOS status-returning function checks `MainThread::current()` before it dereferences a C handle; otherwise-valid off-main calls return `FRAMEWORK_STATUS_UNAVAILABLE`, while malformed inputs can return `FRAMEWORK_STATUS_INVALID_ARGUMENT` first
- C code must serialize calls on one handle and call destroy on the main thread; destroy requires a naturally aligned writable original handle slot, clears it before drop, accepts null inputs, and has no recoverable error path
- The C handle owns one `Clipboard<IosClipboardBackend>`; no global lookup, copied handle, alias destroy, or concurrent call is valid
- `framework_ios_clipboard_availability` maps `Availability::Unknown` to tag 0 because UIKit provides no query for current programmatic-read usability or approval; `Unknown` does not mean `RequiresPermission`; unknown future variants also map to tag 0
- `framework_ios_clipboard_read` copies the first string returned by `UIPasteboard.strings`; it maps `None` to `out_has_value == 0` and the default empty descriptor, while `Some("")` maps to `out_has_value == 1` and a zero-length owned buffer
- When `out_has_value` is 1, C releases `out_text` exactly once with `framework_owned_buffer_destroy`, even when its length is zero
- `FrameworkStr` input is exact UTF-8; NUL bytes are valid; an empty text span is `{NULL, 0}`; the input stays readable through the synchronous call
- B6 copies write input to an `NSString` and assigns `UIPasteboard.string`, replacing all current pasteboard items; UIKit copies the assigned string, and no C input span is retained after the call; other apps may change shared pasteboard state at any time
- Poll the concrete B6 read, write, or clear future once on the main thread; B6 completes on its first poll; `Pending` maps to `FRAMEWORK_STATUS_INTERNAL_ERROR` and no executor or queue is added
- Map known `ClipboardError::Backend` values through `FrameworkStatus::from_error`; unknown non-exhaustive errors map to `FRAMEWORK_STATUS_INTERNAL_ERROR`; initialize each optional native-code output to zero because B6 supplies no native code
- Set every required output to its empty value before address or value checks; `out_text` must not hold a live owned buffer on entry
- Every non-null output pointer and destroy handle slot must be naturally aligned and writable; output storage must be disjoint from inputs, handles, and other outputs; optional native-code pointers may be null
- Exports with a status result catch Rust panics and return `FRAMEWORK_STATUS_PANIC`; no panic may unwind across C
- F6 clear calls B6's current operation; its UIKit implementation sets `UIPasteboard.items` to an empty array, also replacing all current pasteboard items; write and clear replace all current representations, including non-text ones
- General-pasteboard reads may trigger iOS privacy UI; F6 makes no permission, privacy-prompt, or live pasteboard claim

## Share C ABI scope boundary

F6 adds no `ios-share` Cargo feature, C header, symbol, callback, or operation handle. This is an F6 scope rule, not an open D6 C ABI task

D6 `ShareBackend::share` returns a future that borrows its backend. F7 uses B7 `IosShareSession` for its C-owned session and callback rules; it does not expose that borrowed future. See `PLAN_BINDINGS_SHARE.md` for the complete F7 contract

## Non-test validation and handoff

- Check default-feature isolation with `cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios --no-default-features`
- Check the C feature graph with `cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios --features ios-clipboard`; assert `ios-sharing/clipboard` is present and `ios-sharing/share`, `block2`, and `UIActivityViewController` are absent
- Check host feature stubs with `cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin --features ios-clipboard` and `cargo check --locked -p framework-c-api --features ios-clipboard`
- Run iOS device and simulator `cargo check`, Clippy with `-D warnings`, and Release builds with `--locked`
- Compile and link C11 and C++17 fixtures for host stubs, iOS device, and simulator where the toolchain permits; do not run fixtures
- Derive all six availability tags from `bindings/c/abi-manifest.json` and compile C11/C++17 checks against the public C declarations; use `jq -e` to require the clipboard `client`, `read`, and `inputs` ownership records plus the core `FrameworkOwnedBuffer` creator/destroyer fields
- Compare ownership prose with source and public C declarations by manual review; the gate checks required records, not exact prose semantics
- Audit iOS imports for `UIActivityViewController`, share-only symbols, Swift, Python, and unrelated Apple frameworks
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, JSON and shell syntax checks, and scoped `git diff --check`
- Do not add or run tests in F6; report compile/link evidence only, not live paste behavior or privacy UI

## Orchestrator handoff

- Link this plan from `PLAN_BINDINGS.md` after root review
- Reconcile `Cargo.lock`, docs indexes, capability manifest, and CI outside the F6 executor scope
- Keep F6 clipboard-only; future share C ABI changes belong in `PLAN_BINDINGS_SHARE.md`
