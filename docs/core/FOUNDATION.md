# Portable Foundation

## Scope

Workstream A owns shared semantic primitives and root Cargo policy. The foundation layer has five independently usable crates; later workstreams add platform backends and iOS runtime adapters as separate workspace packages. No mandatory framework runtime is part of this foundation layer

```text
framework-alloc ──────► framework-core
framework-async ──────► framework-core
framework-abi ────────► framework-core
framework-platform ───► framework-core
```

The arrows point from each dependent crate to `framework-core`. There are no third-party dependencies. `framework-core` uses only `core`; `framework-alloc` and `framework-abi` use `alloc` for owned storage; `framework-async` uses `core::future`, atomics, and a small internal waker slot; `framework-platform` contains zero-sized target markers only.

The root workspace uses the `crates/*` member glob so later workstreams can add independent crates without editing the root member list. Rust `1.94.1` is pinned in `rust-toolchain.toml`; the workspace declares Rust `1.85` as its minimum because edition 2024 and the unsafe-attribute syntax used by the ABI crate are available there. Release LTO is thin and the bench profile inherits release; consumers retain control of their own application profile.

## Semantic types and representation facts

The following measurements use Rust `1.94.1` on the validation host target, macOS x86_64. Sizes are fixed by the listed integer representation; alignment for scalar wrappers follows that scalar's target ABI and is not universally equal to its byte size. `Error` is an internal Rust value and its observed layout is not a C ABI promise.

| Type | Size | Alignment | Array stride | Representation and domain |
| --- | ---: | ---: | ---: | --- |
| `Platform` | 1 byte | 1 byte | 1 byte | `repr(u8)`; unknown is `0`, dedicated platforms are `1..=6`, other is `255` |
| `Capability` | 2 bytes | 2 bytes | 2 bytes | `repr(u16)`; V1 known IDs are `1..=16` |
| `CapabilityId` | 2 bytes | 2 bytes | 2 bytes | transparent nonzero `u16`; `0` is invalid, unknown future IDs remain representable |
| `Availability` | 1 byte | 1 byte | 1 byte | `repr(u8)` semantic availability state |
| `ErrorKind` | 1 byte | 1 byte | 1 byte | `repr(u8)` portable error category |
| `PlatformErrorCode` | 4 bytes | 4 bytes | 4 bytes | transparent nonzero signed `i32`; `0` means no platform code |
| `Error` | 8 bytes | 4 bytes | 8 bytes | observed Rust layout; not exposed by the C ABI |
| `PermissionState` | 1 byte | 1 byte | 1 byte | `repr(u8)` normalized permission state |
| `AuthorizationState` | 1 byte | 1 byte | 1 byte | `repr(u8)` normalized authorization state |
| `Cancellation` | 1 byte | 1 byte | 1 byte | `repr(u8)` cancellation cause |
| `OperationId` | 8 bytes | target `u64` alignment (8 bytes on this host) | 8 bytes | transparent nonzero `u64`; `0` means no operation |
| `Generation` | 4 bytes | 4 bytes | 4 bytes | transparent nonzero `u32`; `0` is reserved |
| `CompactHandle` | 8 bytes | target `u64` alignment (8 bytes on this host) | 8 bytes | transparent `u64`; low 32 bits are index, high 32 bits are generation |

`CompactHandle` has no all-zero valid value because generation zero is invalid. Its full index range is representable, though `GenerationalSlab` reserves `u32::MAX` as its free-list sentinel. A slab therefore supports at most `u32::MAX` slots with valid indices `0..u32::MAX`. Generation advancement wraps from `u32::MAX` to `1`; after `4,294,967,295` advances of one slot, a historical handle can numerically alias again. Applications must not treat generations as permanent globally unique IDs.

Array stride equals `size_of::<T>()` for every row; the `Error` layout remains an observed Rust layout, not a cross-version or C ABI promise

These values use semantic fixed-width fields rather than native pointers or `usize` identifiers. Raw pointers appear only in the C ABI pointer-length and owned-buffer forms. No serialized pointer representation is defined.

## Crate responsibilities

### `framework-core`

Allocator-free, `#![no_std]` semantic types: `Platform`, `Capability`, `CapabilityId`, `Availability`, `ErrorKind`, `PlatformErrorCode`, `Error`, `Result<T>`, `PermissionState`, `AuthorizationState`, `Cancellation`, `OperationId`, `Generation`, and `CompactHandle`. Errors preserve a stable portable category and optional signed platform code without allocating text or owning a backend error type.

### `framework-alloc`

`#![no_std]` plus `alloc`. `GenerationalSlab<T>` uses contiguous `Vec` slot storage and an intrusive `u32` free list. Empty construction allocates nothing. New slots use generation one; removal advances generation before linking the slot for reuse. `insert` reports capacity/allocation failure while returning the input value. `BitSet` uses fixed-domain contiguous `u64` words; its first bit is the low bit of word zero. It neither resizes nor exposes bit allocation as public framework semantics.

These are reusable tools for compact identities and bounded state; no non-test crate uses them, and no benchmark proves a runtime need. They do not replace `Vec`, `String`, hashing, or general-purpose collections.

### `framework-async`

`#![no_std]` runtime-neutral operation state, cancellation source/token, one-shot completion, and a borrowing `Future`. It starts no task and selects no executor. Its memory ordering and ownership contract are in [ASYNC_AND_OWNERSHIP.md](ASYNC_AND_OWNERSHIP.md).

### `framework-abi`

`#![no_std]` fixed-width status and handle values, pointer-plus-length byte/UTF-8 views, versioned option headers, one-shot C callbacks, and explicitly owned buffers. It uses `alloc` only to transfer a `Vec<u8>` allocation at the ABI edge. Layout and lifecycle are in [C_ABI.md](C_ABI.md).

### `framework-platform`

Zero-sized, sealed platform markers plus the `CurrentPlatform` cfg alias and a static backend association trait. It has no service registry, dynamic selection, native object, or backend dependency.

## Dependency and feature policy

No third-party dependency is justified by the V1 foundation. The operations fit `core`, `alloc`, the standard `Future` trait, and narrow framework-owned types. No executor, atomics helper, collection replacement, serialization crate, C header generator, or panic crate is included. Each future dependency must have a bounded adapter and an explicit replacement rationale before it enters this layer.

`framework-abi` has no default features. Its optional `std` feature supplies `catch_unwind_status` for host-side adapters that can use unwind catching; it does not change the default `no_std` contract. An `extern "C"` entry point must never permit unwinding across the ABI. In a no-std/abort profile, an uncaught panic aborts instead of unwinding.

## Required checks

```sh
cargo check -p framework-core --no-default-features
cargo check -p framework-alloc --no-default-features
cargo check -p framework-async --no-default-features
cargo check -p framework-abi --no-default-features
cargo check -p framework-platform --no-default-features
cargo test -p framework-core
cargo test -p framework-alloc
cargo test -p framework-async
cargo test -p framework-abi --all-features
cargo test -p framework-platform
cargo clippy -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --all-targets -- -D warnings
cargo clippy -p framework-abi --all-features --all-targets -- -D warnings
```

An optimized codegen smoke check compiled `is_current::<CurrentPlatform>()` through a tiny release probe. On the validation host it returned an immediate constant with no backend lookup or call; this confirms static marker selection only and is not a performance benchmark.
