# Foundation C ABI Contract

## Linkable C library

The `framework-c-api` package builds `libframework_c_api.a` over `framework-abi`; Rust consumers should use the Rust-native foundation crates directly. The hand-maintained [C header](../../bindings/c/include/framework.h) declares the foundation value types and core C symbols. The [ABI manifest](../../bindings/c/abi-manifest.json) records the version, status values, known layouts, C symbol list, options prefix, and ownership rules.

`framework_abi_version()` returns a `uint64_t` with the major version in bits 63..32 and minor version in bits 31..0. The current ABI is `1.3`; minor 1 added `framework_options_v1_validate`, minor 2 added `framework_owned_buffer_copy`, and minor 3 adds directly owned error-detail objects without changing existing layouts or status codes. Status aliases remain fixed-width `uint32_t` values; the header names the stable codes with `FRAMEWORK_STATUS_*` constants.

`bindings/c/check.sh` builds the static library, compiles the header as C11 and C++, asserts C field layouts, compares the 64-bit C layout report and each `FRAMEWORK_STATUS_*` value with the manifest, compares the declared C symbols with the static archive, then links and runs `examples/c-minimal`. The example calls `framework_options_v1_validate` on a same-major record larger than the 16-byte V1 prefix, copies bytes through `framework_owned_buffer_copy`, and creates, views, and destroys one error-detail object. Rust layout checks cover fixed-width values and pointer-size-dependent handle representation. The manifest layout entry covers targets with 64-bit pointers and 8-byte `uint64_t` alignment; other target layouts follow the explicit C field declarations and target C alignment.

The opt-in C static library enables `framework-abi/std` because owned buffers and error-detail objects must release framework allocations and the static library needs a global allocator and panic runtime. `framework-abi` itself remains `no_std` by default; this feature is not enabled for Rust-only consumers.

`framework_owned_buffer_copy` copies an input `FrameworkSlice` into a framework allocation. A null data pointer is valid only for zero length; zero-length input succeeds without allocating. A length that cannot fit a Rust slice is invalid. The output pointer is required, writable, aligned, disjoint from the input, and empty of a live allocation on entry; the function initializes it to null/zero before input validation, and it remains empty on failure. Allocation or descriptor-capacity failure returns `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`. On success, the caller owns the original unchanged descriptor and must destroy it exactly once. No panic may unwind across an exported C symbol; the factory uses fallible allocation, the version and options-validator functions are arithmetic-only, and invalid pointers/descriptors violate their FFI preconditions.

## Primitive values

The C ABI uses fixed-width values and explicit pointer-length pairs. It does not expose `Vec`, `String`, references, `Box`, `Arc`, Rust enum layout, or a Rust compiler ABI.

| Rust type | C shape | Size/alignment |
| --- | --- | --- |
| `FrameworkStatus` | `uint32_t` | 4 / 4 |
| `FrameworkOperationHandle` | `uint64_t` | 8 / target `uint64_t` alignment; zero means absent |
| `FrameworkErrorHandle` | `uint64_t` | 8 / target `uint64_t` alignment; zero means no detail |
| `FrameworkErrorDetailHandle` | `FrameworkErrorDetail *` | target pointer size/alignment; null means no detail |
| `FrameworkSlice` | `const uint8_t *data; uint64_t length` | pointer + 8 bytes, rounded to the target C alignment |
| `FrameworkStr` | `const uint8_t *data; uint64_t length` | pointer + 8 bytes, rounded to the target C alignment |
| `FrameworkOwnedBuffer` | `uint8_t *data; uint64_t length; uint64_t capacity` | pointer + 16 bytes, rounded to the target C alignment |
| `FrameworkOptionsV1` | four `uint32_t` fields | 16 / 4 |

The pointer fields are the only pointer-width-dependent parts. `FrameworkSlice` and `FrameworkStr` carry byte lengths, not element counts; `FrameworkStr` bytes must be UTF-8. Their backing memory must remain valid for foreign use. An unsafe Rust conversion back to a borrowed slice/string requires the caller to uphold pointer validity, alignment, length, UTF-8, and lifetime requirements.

`FrameworkStatus` V1 codes are stable: `0` success, `1` invalid argument, `2` unsupported, `3` unavailable, `4` permission denied, `5` cancelled, `6` timeout, `7` not found, `8` already exists, `9` resource exhausted, `10` platform error, `11` internal error, and `12` panic caught by the optional `std` helper. A `FrameworkErrorDetailHandle` stores one fixed-width framework status code and a UTF-8 diagnostic message; the supplied code is preserved exactly, including unknown future framework status values. It does not store a platform-native code or define another mapping; Rust callers can use `FrameworkStatus::from_error` to apply the stable `framework_core::ErrorKind` mapping first. It is a direct opaque pointer with its own create/view/destroy functions, not a `FrameworkErrorHandle` integer ID. `FrameworkErrorHandle` remains unchanged for ABI compatibility and is not converted to or from the direct pointer.

`FrameworkErrorDetailHandle` is opt-in at the C boundary and uses no global registry. Create copies the input UTF-8 bytes exactly, including embedded NUL bytes, without adding a terminator, and initializes the output handle to null before validation. Null output is invalid; a null message pointer is valid only for zero length; unrepresentable or over-`isize::MAX` lengths and invalid UTF-8 return `FRAMEWORK_STATUS_INVALID_ARGUMENT`; reservation/capacity or object-allocation failure returns `FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`. On success, the original handle must be destroyed exactly once. A null destroy is a no-op. View initializes both non-null outputs to an empty default before null-detail validation, writes the exact stored status code to `out_status` and a borrowed message view to `out_message`; its function return reports the view-call result. The view expires at destruction; callers synchronize destruction against all views and do not mutate or copy the handle. Invalid non-null pointers, misalignment, overlapping outputs, stale handles, and double destruction violate FFI preconditions and are not recoverable errors. The legacy `FrameworkErrorHandle(uint64_t)` type keeps its original layout and is not used to encode a pointer.

## Owned buffer lifecycle

`FrameworkOwnedBuffer::try_from_vec` transfers an allocation from a Rust `Vec<u8>`, and `framework_owned_buffer_copy` copies a borrowed C span into a new allocation. A Rust owner may inspect it, move it back with `into_vec`, or let its Rust value drop. A C consumer must call `framework_owned_buffer_destroy` once on the original descriptor. The destructor resets that descriptor to null/zero, so a second call on the same descriptor is a no-op.

Do not copy the descriptor by value, mutate its pointer/length/capacity, use its pointer after destruction, or destroy two copies. The C destroy function accepts null. A non-null pointer must reference a live, unchanged framework-created descriptor. These are unsafe FFI preconditions because the allocation provenance and exact capacity must remain intact for `Vec` destruction.

## Callbacks and options

`FrameworkCompletionCallback` is an optional C function pointer with an opaque `void *context`, fixed-width operation handle/status, and borrowed result slice. `FrameworkCompletion` is a one-shot Rust adapter. Its unsafe constructor requires the context to remain valid until invocation or adapter drop and requires the callback not to unwind. The result pointer is valid only for the duration of the callback.

`FrameworkOptionsV1` has `struct_size`, `abi_version`, `flags`, and `reserved`. Its V1 constructor sets `struct_size` to 16, `abi_version` to 1, and `reserved` to zero. `framework_options_v1_validate` checks the 16-byte prefix size first, requires ABI major 1, then requires `reserved` to be zero. It accepts a declared size of at least 16 for ABI major 1 and ignores flags and trailing bytes; it reads no bytes after the V1 prefix and retains no pointer. A non-null pointer must be aligned and point to fully initialized, readable storage for a complete `FrameworkOptionsV1` through the synchronous call, and the caller must prevent unsynchronized mutation. A null or undersized record or nonzero `reserved` returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`; another ABI major returns `FRAMEWORK_STATUS_UNSUPPORTED`. This guarantee belongs to this validator only: any later API that reads additional fields must independently check its supported prefix before those field reads.

## Panic boundary

No panic may unwind across a C ABI frame. The default crate remains `no_std`; the error-detail constructor uses fallible reservation and direct fallible allocation and has no panic path for valid pointer preconditions. An unexpected panic at an `extern "C"` boundary aborts rather than unwinds. The optional `std` feature exposes `catch_unwind_status` for host adapters that need to convert an unwind into `FrameworkStatus::PANIC`. It adds no default dependency or runtime requirement.
