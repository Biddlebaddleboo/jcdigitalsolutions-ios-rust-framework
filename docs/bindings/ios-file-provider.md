# iOS File Provider C ABI

F16 adds the opt-in `ios-file-provider` feature to `framework-c-api`. It exposes B75's asynchronous snapshot of registered domains for the calling app's own File Provider extension through one operation handle, one readiness-only callback, and a later poll

## Request and readiness

Call `framework_ios_file_provider_registered_domain_presence_start` to begin the Apple request. On success, the output slot contains one unique handle. The request starts during the call, so the callback can run inline before start returns if the result is already ready. Otherwise, it may run later on Apple's unspecified callback queue. The callback only signals that the host should poll; it carries no result

The callback can run concurrently with host work on another thread. Do not unwind or call any F16 function from it. Schedule a later poll after callback return, and serialize all F16 calls on each handle. The C ABI adds no main-thread rule. Callback/context remain host-owned through callback return, a ready-result poll, or destroy return

## Poll result and native errors

`framework_ios_file_provider_operation_poll` returns the FFI call status separately from the operation outcome. `FRAMEWORK_STATUS_OK` with `out_ready` 0 means pending. `FRAMEWORK_STATUS_OK` with `out_ready` 1 means one terminal record was produced; a later poll returns `FRAMEWORK_STATUS_NOT_FOUND`. Poll initializes `out_ready` to zero and the record to an empty value before it reads the handle

`FrameworkIosFileProviderResultV1` is output-only, 48 bytes with 8-byte alignment on 64-bit iOS. It has `struct_size` at 0, `abi_version` at 4, `status` at 8, `has_registered_domains` at 12, `reserved` at 13, `native_error_domain` at 16, and `native_error_code` at 40. A successful query sets only `has_registered_domains` to 0 or 1. A native NSError uses result status `FRAMEWORK_STATUS_PLATFORM_ERROR`; its `NSError.domain` is copied into `native_error_domain` as owned UTF-8 bytes and its `NSInteger` code is preserved in `native_error_code` as `int64_t`. This is an error domain string, not an `NSFileProviderDomain` identifier

Initialize the output record with an empty `native_error_domain` before each poll. Destroy a returned non-empty domain buffer with `framework_owned_buffer_destroy` before reusing the result record. A native error never becomes a false Boolean. API-floor errors use result status `FRAMEWORK_STATUS_UNAVAILABLE`; a callback panic caught by B75 uses `FRAMEWORK_STATUS_PANIC`

## Destroy and native request lifetime

Apple exposes no cancellation path for `getDomainsWithCompletionHandler:`. F16 has no cancel function. Destroy closes the per-handle readiness signal, waits for a callback already in flight, clears the original handle slot, drops the Rust future, and detaches Rust interest. Apple's request and its private completion state may remain alive until the native callback returns; destroy does not stop the request. No callback can start after destroy returns

Serialize calls on one handle and destroy its original slot once. If destroy races with a readiness callback on another thread, destroy may wait until that host callback returns. Do not call destroy or poll from the callback itself. A ready-result poll also closes the readiness signal; no later callback can occur

## Scope and limits

The API floor is iOS 11.0. It reports only whether the calling app's own provider has one or more registered domains at query time. It does not enumerate other providers, expose domain IDs/names/URLs, show a picker, access files, or report enablement, reachability, sync, or future file-access success. The query can cause the process's File Provider materialized/pending-set change notifications to begin posting as documented by the SDK

Non-iOS valid starts and polls initialize outputs and return `FRAMEWORK_STATUS_UNSUPPORTED`; they create no mock handle and import no Apple framework. Destroy of a non-null host handle value returns unsupported without dereferencing or changing the slot. A caught Rust panic returns `FRAMEWORK_STATUS_PANIC`; no panic unwinds across C. The focused gate builds and inspects C11/C++17 probes but never executes them

See [B75](../../PLAN_IOS_FILEPROVIDER.md), [G78](../../PLAN_VALIDATION_IOS_FILEPROVIDER.md), and [F16](../../PLAN_BINDINGS_FILEPROVIDER.md)
