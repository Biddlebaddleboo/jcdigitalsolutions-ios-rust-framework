# PLAN_BINDINGS_F35.md — Direct-owned C error-detail object

## Objective

Add optional C ABI-owned diagnostic detail without changing the existing `FrameworkErrorHandle(u64)` representation or adding a process-wide registry.

## Status

Implementation is complete in this workstream. The C ABI is version 1.3; `FrameworkErrorHandle(u64)` remains unchanged. No tests or runtime example execution were performed.

## Contract

- `FrameworkErrorDetailHandle` is a distinct opaque `FrameworkErrorDetail *`; its size and alignment follow the target C pointer ABI.
- `framework_error_detail_create` copies one `FrameworkStr` UTF-8 message byte-for-byte, including embedded NUL, without adding a terminator; it preserves the supplied framework status exactly, including unknown future values.
- `framework_error_detail_view` returns its call result separately from the stored fixed-width status and a borrowed `FrameworkStr` valid only while the object remains alive.
- `framework_error_detail_destroy` releases the original pointer exactly once; null is a no-op.
- The object owns a `FrameworkOwnedBuffer` for message bytes and is allocated directly with `alloc::alloc::alloc`; no global registry, integer pointer encoding, or `FrameworkErrorHandle` conversion exists.
- `framework-abi` remains `no_std + alloc`; the opt-in C static library already supplies its allocator through the `std` feature.

## Failure and safety behavior

- Creation requires a non-null, aligned, writable output-handle pointer with no live object and disjoint from input bytes; it writes null first and leaves null on failure.
- Null message data is valid only at zero length. Nonempty input must be readable and valid UTF-8 for the synchronous call.
- Unrepresentable or greater-than-`isize::MAX` message length and invalid UTF-8 return `FRAMEWORK_STATUS_INVALID_ARGUMENT`; reservation/capacity or object-allocation failure returns `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`.
- View requires two non-null, aligned, writable, mutually disjoint outputs that are disjoint from object/message storage; it initializes both before null-detail validation.
- A null view output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without writes; an impossible invalid internal message representation returns `FRAMEWORK_STATUS_INTERNAL_ERROR` with the outputs left defaulted.
- Invalid non-null pointers, misalignment, overlap, stale handles, mutation, and double destruction violate FFI preconditions and are not recoverable status cases.
- Callers synchronize destruction against all outstanding views. No input pointer is retained.
- The implementation has no panic path for valid pointer preconditions; no panic may unwind across C, and an unexpected panic aborts at the `extern "C"` boundary.

## ABI and implementation

- ABI minor is 3; major remains 1. Existing record layouts and status values are unchanged.
- Public header, Rust exports, manifest symbols/ownership, focused C ABI documentation, and the C minimal example are updated together.
- No Swift, platform backend, or shared-tool engine source is in scope.

## Evidence and limits

- Pinned `ios-rust-build` and `ios-rust-validate` version `0.1.0` installed for `x86_64-apple-darwin`; Rust `1.94.1`.
- PASS: `cargo +1.94.1 check --locked -p framework-abi --no-default-features`.
- PASS: `cargo +1.94.1 check --locked -p framework-c-api`.
- PASS: `cargo +1.94.1 build --locked --release -p framework-c-api`; archive symbol set matches `bindings/c/abi-manifest.json`.
- PASS: Rust formatting check, ABI manifest JSON/value inspection, C11 and C++17 header syntax, and C minimal example syntax.
- Not run: Cargo tests, `bindings/c/check.sh`, linked C example, and any runtime check.
- Host compilation and syntax checks do not prove C runtime allocation behavior, panic injection, 32-bit execution, or iOS device/simulator linkage.

Mainline run
[38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
later passed the C API header, layout, symbol, and consumer check on macOS 15 and Xcode 27, with
the C ABI gates 235–275 also passing on both Apple lanes. The linked C example and C ABI probes
were not executed; no runtime allocation or device behavior is claimed.
