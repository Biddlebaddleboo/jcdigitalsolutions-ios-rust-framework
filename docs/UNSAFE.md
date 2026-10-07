# Unsafe and Low-Level Code Policy

## Position

Unsafe Rust is expected in a framework that participates directly in Objective-C, C, Swift ABI, Blocks, and platform runtime conventions.

The goal is not "zero unsafe." The goal is a small, reviewable unsafe core with explicit invariants.

## Every unsafe boundary must document

- ABI/calling convention;
- pointer provenance and validity;
- initialization state;
- ownership transfer;
- retain/release expectations;
- lifetime relationship;
- aliasing rules;
- thread requirements;
- nullability;
- error behavior;
- panic/unwind behavior.

Prefer `#![deny(unsafe_op_in_unsafe_fn)]` where practical.

## Acceptable reasons for unsafe

- defining/implementing native classes or protocols;
- FFI;
- recovering callback context;
- zero-copy views over native memory with proven lifetime;
- manually bound public APIs;
- narrowly scoped direct Objective-C ABI calls;
- Swift-ABI interoperability;
- architecture-specific assembly.

## Unacceptable justification

"Unsafe might be faster" is not sufficient.

Show either:
- required native semantics;
- an API/binding limitation;
- generated-code evidence;
- benchmark evidence.

## Direct Objective-C ABI

Manual `objc_msgSend` or equivalent runtime calls are exceptions, not defaults.

Before introducing them, compare optimized objc2 output and document why the direct path is better or necessary.

## Assembly

Assembly must be isolated and architecture-gated.

Document:
- register inputs/outputs;
- clobbers;
- stack/alignment assumptions;
- Apple ABI requirements;
- fallback implementation;
- benchmark evidence.

## Auditing

Unsafe code should be concentrated in clearly named modules.

Public safe APIs built over unsafe internals must state the invariant that makes them safe.

Tests should cover teardown, reentrancy, wrong-thread use where representable, stale callback handles, double completion, and lifetime edge cases.
