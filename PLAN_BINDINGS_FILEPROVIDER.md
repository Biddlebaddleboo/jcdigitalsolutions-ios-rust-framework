# PLAN_BINDINGS_FILEPROVIDER.md — Workstream F16: iOS File Provider C ABI

## Status

F16 source, header, ABI manifest, guide, and focused check are complete. `sh bindings/c/check-ios-file-provider.sh` passed after root refreshed Cargo.lock. The gate verified host, iOS device (minos 11.0), and iOS Simulator (minos 14.0) checks, strict Clippy, Release builds, C11/C++17 links, direct imports, and archive exports. No tests or linked-probe execution took place

## Goal

Expose B75's one-shot registered-domain presence query through an opt-in C ABI. Return only a Boolean snapshot or an owned native NSError domain/code. Preserve the unspecified Apple callback queue, per-request ownership, and noncancellable native request contract. Do not add a main-thread rule, native object handle, domain identifier, global registry, or executor

## Read first

- `PLAN_BINDINGS.md`
- `PLAN_BINDINGS_ASYNC.md`
- `PLAN_IOS_FILEPROVIDER.md`
- `platform/ios/ios-file-provider/src/lib.rs`
- `platform/ios/ios-file-provider/src/completion.rs`
- `platform/ios/ios-file-provider/src/platform.rs`
- `docs/ios/fileprovider.md`

## Write scope

- `bindings/c/Cargo.toml` (`ios-file-provider` optional target dependency and feature only)
- `bindings/c/src/lib.rs` (feature gate, `alloc`/`std` gate, and re-exports only)
- `bindings/c/src/ios_file_provider.rs`
- `bindings/c/include/framework_ios_file_provider.h`
- `bindings/c/abi-manifest.json` (F16 entry, owned-buffer creator, panic/runtime inventory)
- `bindings/c/check-ios-file-provider.sh`
- `docs/bindings/ios-file-provider.md`
- `PLAN_BINDINGS_FILEPROVIDER.md`

Do not edit B75 backend source, root Cargo.toml/Cargo.lock, `PLAN_BINDINGS.md`, root CI, global documentation indexes, or aggregate capability plans. No tests

## Contract decisions

- Export one start, readiness-only callback, poll/result, and destroy. Do not export cancel: `getDomainsWithCompletionHandler:` has no cancellation path. Destroy clears the C slot, detaches Rust interest, closes the C readiness signal, waits for an in-flight C callback, and drops the future; Apple's request and private completion state may continue until the native callback returns
- The Apple request starts during B75's `request_registered_domain_presence()` call. Poll the new future once before publishing the handle so an already-completed native query is captured. Keep readiness inactive until after the output handle is written; a concurrent native completion is recorded as pending and notified after publication
- A per-handle `Arc<ReadinessSignal>` implements `Wake`. Its mutex/condition-variable gate permits arbitrary-queue callback delivery and makes destroy wait for callback return. It is not a scheduler or registry. The C callback may run inline before start returns or later on Apple's unspecified queue; it must not unwind or re-enter F16. The host serializes calls on each handle and schedules a later poll after callback return
- Do not impose a main-thread rule. The B75 callback queue is unspecified. The callback carries no result; poll consumes one terminal result. Poll after consumption returns `FRAMEWORK_STATUS_NOT_FOUND`
- `FrameworkIosFileProviderResultV1` is output-only, 48 bytes and 8-byte aligned on 64-bit iOS. It reports success as `has_registered_domains` 0 or 1; a native error is `FRAMEWORK_STATUS_PLATFORM_ERROR`, with `NSError.domain` copied as owned UTF-8 bytes and `NSError.code` preserved as `int64_t`. This is the NSError domain, not a File Provider domain identifier. Destroy the nested `FrameworkOwnedBuffer` with `framework_owned_buffer_destroy`
- Start rejection initializes the handle slot to null and takes no callback/context ownership. On iOS, an accepted request returns `FRAMEWORK_STATUS_OK`; an iOS runtime below the iOS 11 API floor yields a ready result with `FRAMEWORK_STATUS_UNAVAILABLE`. Valid non-iOS starts return `FRAMEWORK_STATUS_UNSUPPORTED` without a mock handle
- Callback/context remain host-owned and valid until callback return, a ready-result poll, or destroy return. Destroy waits for a callback already in flight and prevents a later callback. It may block until an in-flight host callback returns
- Poll initializes `out_ready` and the output record. The host supplies an empty `native_error_domain` on entry, destroys any returned owned domain before reusing the result, and reads semantic fields only when the FFI poll status is `FRAMEWORK_STATUS_OK` and `out_ready` is 1
- Catch Rust panics at status-returning exports. B75 converts a panic in its native callback to a terminal `FRAMEWORK_STATUS_PANIC` result. The host callback must not unwind across C

## Required validation

`sh bindings/c/check-ios-file-provider.sh` passed with the root lock. It parsed JSON/shell, checked format/diff and C symbol/result-layout parity, inspected default/iOS/host feature trees, verified host stubs and no Apple dependency leakage, ran host/device/Simulator non-test checks, strict Clippy and Release builds, and linked C11/C++17 probes without execution. It verified C imports `FileProvider`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ adds `libc++.1.dylib`. Both archives export the three F16 symbols; device minos is 11.0 and Simulator minos is 14.0. No tests or linked probe binaries ran

Do not run tests or execute linked probe binaries

## Limits and handoff

The Apple API floor is iOS 11.0. The query reports only whether the calling app's own provider has registered domains at callback time. It does not report provider enablement, reachability, sync, file availability, or future access success. `getDomainsWithCompletionHandler:` can start process notifications for materialized/pending-set changes. No provider request, extension, domain registration, entitlement, prompt, picker, or runtime behavior is exercised by compile/link checks

After scoped source and static checks are ready, ask root to integrate the optional dependency into Cargo.lock. Root owns Cargo.lock, CI, aggregate binding status, and documentation indexes
