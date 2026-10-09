# Foundation C ABI Contract

## Linkable C library

The `framework-c-api` package builds `libframework_c_api.a` over `framework-abi`; Rust consumers should use the Rust-native foundation crates directly. The hand-maintained [C header](../../bindings/c/include/framework.h) declares only the foundation value types and three core C symbols. The [ABI manifest](../../bindings/c/abi-manifest.json) records the version, status values, known layouts, C symbol list, options prefix, and ownership rules.

`framework_abi_version()` returns a `uint64_t` with the major version in bits 63..32 and minor version in bits 31..0. The current ABI is `1.1`; minor 1 adds `framework_options_v1_validate` without changing existing layouts or status codes. Status aliases remain fixed-width `uint32_t` values; the header names the stable codes with `FRAMEWORK_STATUS_*` constants.

`bindings/c/check.sh` builds the static library, compiles the header as C11 and C++, asserts C field layouts, compares the 64-bit C layout report and each `FRAMEWORK_STATUS_*` value with the manifest, compares the declared C symbols with the static archive, then links and runs `examples/c-minimal`. The example calls `framework_options_v1_validate` on a same-major record larger than the 16-byte V1 prefix. Rust layout tests in `framework-abi` and `framework-c-api` check Rust type sizes, alignments, and the public `FrameworkOptionsV1` offsets against `repr(C)` field-order and target-alignment rules. The manifest layout entry covers targets with 64-bit pointers and 8-byte `uint64_t` alignment; other target layouts follow the explicit C field declarations and target C alignment.

The opt-in C static library enables `framework-abi/std` because its owned-buffer destructor must release `Vec<u8>` allocations and the static library needs a global allocator and panic runtime. `framework-abi` itself remains `no_std` by default; this feature is not enabled for Rust-only consumers.

This slice does not export an owned-buffer creator. The minimal C example therefore calls `framework_owned_buffer_destroy(NULL)` only; it does not invent a descriptor or claim ownership of a Rust allocation. A non-null destroy argument must still be the original unchanged descriptor created by a framework API. No panic may unwind across an exported C symbol; the version and options-validator functions are arithmetic-only, while the inherited destructor has no recoverable panic path and invalid descriptors violate its FFI preconditions.

## Primitive values

The C ABI uses fixed-width values and explicit pointer-length pairs. It does not expose `Vec`, `String`, references, `Box`, `Arc`, Rust enum layout, or a Rust compiler ABI.

| Rust type | C shape | Size/alignment |
| --- | --- | --- |
| `FrameworkStatus` | `uint32_t` | 4 / 4 |
| `FrameworkOperationHandle` | `uint64_t` | 8 / target `uint64_t` alignment; zero means absent |
| `FrameworkErrorHandle` | `uint64_t` | 8 / target `uint64_t` alignment; zero means no detail |
| `FrameworkSlice` | `const uint8_t *data; uint64_t length` | pointer + 8 bytes, rounded to the target C alignment |
| `FrameworkStr` | `const uint8_t *data; uint64_t length` | pointer + 8 bytes, rounded to the target C alignment |
| `FrameworkOwnedBuffer` | `uint8_t *data; uint64_t length; uint64_t capacity` | pointer + 16 bytes, rounded to the target C alignment |
| `FrameworkOptionsV1` | four `uint32_t` fields | 16 / 4 |

The pointer fields are the only pointer-width-dependent parts. `FrameworkSlice` and `FrameworkStr` carry byte lengths, not element counts; `FrameworkStr` bytes must be UTF-8. Their backing memory must remain valid for foreign use. An unsafe Rust conversion back to a borrowed slice/string requires the caller to uphold pointer validity, alignment, length, UTF-8, and lifetime requirements.

`FrameworkStatus` V1 codes are stable: `0` success, `1` invalid argument, `2` unsupported, `3` unavailable, `4` permission denied, `5` cancelled, `6` timeout, `7` not found, `8` already exists, `9` resource exhausted, `10` platform error, `11` internal error, and `12` panic caught by the optional `std` helper. Extended native details belong behind an error handle; native codes are not encoded into `FrameworkStatus`.

`FrameworkErrorHandle` is only the fixed-width opaque value type in this foundation; no error-detail object producer, storage policy, or destructor is defined yet. A later API that creates owned detail must specify its owner and release function before exposing it.

## Owned buffer lifecycle

`FrameworkOwnedBuffer::try_from_vec` transfers an allocation from a Rust `Vec<u8>`. A Rust owner may inspect it, move it back with `into_vec`, or let its Rust value drop. A C consumer must call `framework_owned_buffer_destroy` once on the original descriptor. The destructor resets that descriptor to null/zero, so a second call on the same descriptor is a no-op.

Do not copy the descriptor by value, mutate its pointer/length/capacity, use its pointer after destruction, or destroy two copies. The C destroy function accepts null. A non-null pointer must reference a live, unchanged framework-created descriptor. These are unsafe FFI preconditions because the allocation provenance and exact capacity must remain intact for `Vec` destruction.

## Callbacks and options

`FrameworkCompletionCallback` is an optional C function pointer with an opaque `void *context`, fixed-width operation handle/status, and borrowed result slice. `FrameworkCompletion` is a one-shot Rust adapter. Its unsafe constructor requires the context to remain valid until invocation or adapter drop and requires the callback not to unwind. The result pointer is valid only for the duration of the callback.

`FrameworkOptionsV1` has `struct_size`, `abi_version`, `flags`, and `reserved`. Its V1 constructor sets `struct_size` to 16, `abi_version` to 1, and `reserved` to zero. `framework_options_v1_validate` checks the 16-byte prefix size first, requires ABI major 1, then requires `reserved` to be zero. It accepts a declared size of at least 16 for ABI major 1 and ignores flags and trailing bytes; it reads no bytes after the V1 prefix and retains no pointer. A non-null pointer must be aligned and point to fully initialized, readable storage for a complete `FrameworkOptionsV1` through the synchronous call, and the caller must prevent unsynchronized mutation. A null or undersized record or nonzero `reserved` returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`; another ABI major returns `FRAMEWORK_STATUS_UNSUPPORTED`. This guarantee belongs to this validator only: any later API that reads additional fields must independently check its supported prefix before those field reads.

## Panic boundary

No panic may unwind across a C ABI frame. The default crate remains `no_std`; an `extern "C"` entry point must be written to avoid panicking, and an uncaught Rust panic at that ABI is an abort, not an unwind. The optional `std` feature exposes `catch_unwind_status` for host adapters that need to convert an unwind into `FrameworkStatus::PANIC`. It adds no default dependency or runtime requirement.
