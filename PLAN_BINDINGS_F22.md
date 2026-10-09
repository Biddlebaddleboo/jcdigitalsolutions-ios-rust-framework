# PLAN_BINDINGS_F22.md — F22: iOS Accelerate Vector-Addition C ABI

## Objective

Expose only B65's synchronous equal-length `f32` vector addition through one opt-in C function. Preserve the input/output borrowing and `vDSP_Length` behavior. Do not add a portable math contract, parity claim, or performance claim

## Status

F22 is integrated in the root C ABI feature graph, public exports, ABI manifest, Cargo.lock, macOS CI, aggregate plan, and docs index. `sh bindings/c/check-ios-accelerate.sh` passed formatting, manifest/symbol/feature checks, host/device/Simulator compilation and strict Clippy, Release builds, C11/C++17 links, exact Accelerate/libSystem imports (+libc++ for C++), `_vDSP_vadd`, exports, and minos 10.0/14.0. No tests, consumers, or probes were executed

The pointer-contract audit adds static gate checks for element-to-byte limits, span overlap, full-call lifetime, memory-validity limits, sync rules, and no pointer retention. The ABI manifest now records full-call input immutability and output synchronization preconditions; the gate asserts those ownership strings. Shell syntax, Rust format, C11/C++17 syntax, and these static checks passed. This audit did not run Cargo, links, tests, or probes

Root reran `sh bindings/c/check-ios-accelerate.sh` after the source-order and manifest assertions were added. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact Accelerate/libSystem imports, `_vDSP_vadd`, export parity, and minos 10.0/14.0 passed. Linked probes were not executed; no tests ran

## Feasibility and backend contract

B65 provides `ios_accelerate::vector_add(&[f32], &[f32], &mut [f32]) -> Result<(), VectorAddError>`. It returns `LengthMismatch` before FFI when lengths differ, returns success for equal empty slices without an FFI call, and otherwise calls public `vDSP_vadd` with all strides equal to one. Its only other error is a defensive `vDSP_Length` conversion failure. The backend has no retained state or main-thread requirement and exposes no pointer or native handle

The C wrapper takes lengths in elements, validates equality, representability, byte length, alignment, and address-range arithmetic, then rejects output overlap with either input before it creates Rust slices. Input ranges may overlap because both become immutable Rust slices. On invalid or unsupported status it does not change output. The app must keep both inputs immutable and prevent unsynchronized output access for the full call. The wrapper checks count equality, byte-length bounds, non-nullness, alignment, address-range overflow, and output/input overlap, but cannot prove that memory is valid, readable, writable, or live. No pointer is retained after return. The iOS API floor is 4.0; the existing link probes use target minima 10.0/device and 14.0/Simulator and retain `_vDSP_vadd` from `Accelerate.framework`. See [B65](PLAN_IOS_ACCELERATE.md), [D59](PLAN_CAPABILITIES_ACCELERATE.md), and Apple's [`vDSP_vadd` documentation](https://developer.apple.com/documentation/accelerate/vdsp_vadd)

## Exact C contract

- Optional Cargo feature: `ios-accelerate`, backed only by an optional target-iOS dependency on `ios-accelerate`
- Header: `framework_ios_accelerate.h`
- Export: `FrameworkStatus framework_ios_accelerate_vector_add(const float *a, uint64_t a_length, const float *b, uint64_t b_length, float *output, uint64_t output_length)`
- Counts: element counts, not bytes; all three counts must be equal. Each count must fit `usize`, and its byte length must fit a Rust slice. Nonzero pointers must be non-null, aligned, and valid for the full call on iOS
- Aliasing: inputs may overlap each other; output must not overlap either input. Reject overlap before it creates Rust slices. A mismatch or malformed range returns `INVALID_ARGUMENT` without reading inputs or changing output
- Empty input: null pointers are valid when every count is zero. On iOS, this is `OK` without calling Accelerate; on non-iOS, it is `UNSUPPORTED`
- Output: every element is overwritten on success; invalid arguments and valid non-iOS calls do not change output. Output is unspecified if a caught Rust panic occurs
- Status: success `OK`; mismatch, unrepresentable count, invalid alignment/range, or output/input overlap `INVALID_ARGUMENT`; valid non-iOS call `UNSUPPORTED`; caught panic `PANIC`. There is no native error result or runtime-unavailable path
- Threading: synchronous; no main-thread rule or thread-safety guarantee is added
- Memory and synchronization: the app keeps both inputs immutable and prevents unsynchronized output access for the full call. The wrapper checks count equality, byte-length bounds, non-nullness, alignment, address-range overflow, and output/input overlap, but cannot prove that memory is valid, readable, writable, or live. No pointer is retained after return. The non-iOS stub checks metadata, then returns `UNSUPPORTED` without a data-byte read or write
- Limits: one `f32` unit-stride addition only; no portable facade, numerical parity, certification, or performance claim

## F22-owned files

- `bindings/c/src/ios_accelerate.rs`
- `bindings/c/include/framework_ios_accelerate.h`
- `bindings/c/check-ios-accelerate.sh`
- `docs/bindings/ios-accelerate.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, aggregate binding status, and documentation indexes

## Focused gate

`sh bindings/c/check-ios-accelerate.sh` checks default/iOS/host feature trees, host C11/C++17 links and import isolation, host/device/Simulator compilation and strict Clippy, Release archives, device/Simulator C11/C++17 links, exact Accelerate/libSystem imports (plus libc++ for C++), `_vDSP_vadd`, one C ABI export, and deployment minima 10.0/14.0. Consumer binaries are build-only and were not run; no tests are in scope

Do not alter B65 backend files
