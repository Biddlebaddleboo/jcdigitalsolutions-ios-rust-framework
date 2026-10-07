# Performance Policy

## Objective

The framework must provide high-level portability without imposing a meaningful general runtime penalty over the best equivalent native implementation.

The project separates:
- intrinsic platform cost;
- language-binding cost;
- avoidable framework cost.

## Rust fast path

For Rust applications, portable wrappers and platform selection should be statically dispatched and inlineable wherever practical.

Target topology:

```text
high-level Rust API
  -> portable wrapper (often optimized away)
  -> statically selected backend
  -> native platform operation
```

The portable abstraction should not require runtime service lookup or virtual dispatch for ordinary fixed-target builds.

## C/C++ path

Expected incremental overhead is normally:
- one ordinary C ABI call;
- any representation conversion actually required by the boundary.

The C ABI should not introduce serialization, IPC, or duplicate object models.

## Python path

Do not claim Python has zero runtime overhead.

The target is:
- minimal native binding overhead beyond CPython;
- coarse meaningful boundary crossings;
- native execution for heavy loops/work;
- no CPython dependency for non-Python consumers.

## Unavoidable platform cost

On iOS examples include:
- `objc_msgSend`;
- required retain/release;
- UIKit layout/rendering;
- Core Animation;
- text shaping;
- main-thread serialization;
- Apple-required Block copying;
- system calls;
- network I/O;
- required representation conversions.

Equivalent costs on other platforms are likewise not framework regressions when native implementations pay them too.

## Avoidable framework cost

Treat these as optimization bugs unless justified:
- additional reference counting;
- wrapper allocation;
- dynamic dispatch added only by the framework;
- duplicate string/data conversion;
- unnecessary copies;
- callback registry locking;
- global locks;
- duplicate object models;
- unnecessary foreign runtime objects;
- serialization;
- runtime backend lookup;
- global service-manager calls;
- task/thread creation per trivial operation;
- C ABI round-trip on the Rust-native path;
- linking unrelated capability modules.

## Performance-gated reimplementation policy

Replacing an Apple framework implementation is not automatically an optimization.

The default decision rule is:

```text
Apple path faster/equal or uniquely system-integrated
    -> keep Apple path

Rust path measurably faster/lighter for the same semantics
    -> prefer Rust path

uncertain
    -> keep Apple path and benchmark before replacement
```

A replacement benchmark should include the dimensions that matter for that capability, including CPU time, latency, allocations, copies, RSS/working set, cache behavior, energy, binary/linkage cost, and startup cost.

Hardware-accelerated Apple paths deserve a strong presumption of retention. Examples include Accelerate/BLAS, Metal Performance Shaders, Core ML compute-unit scheduling, VideoToolbox hardware codecs, and similar APIs that can exploit hardware or OS-private tuning unavailable to ordinary application code.

A Rust implementation may still win for:
- small-data operations where foreign-object setup dominates;
- fused operations that avoid intermediate allocations/copies;
- compact state machines/algorithms;
- parsing/serialization with better memory layout;
- application-specific hot paths that avoid generic framework overhead;
- cross-platform code where one optimized implementation replaces several higher-overhead wrappers.

Do not compare a reduced-function Rust prototype against a feature-complete Apple implementation. Benchmark equivalent semantics.

## Binary and dependency size

Fine-grained modularity is a performance requirement.

Measure:
- final binary size;
- linked platform frameworks/libraries;
- transitive crate dependency count;
- optional Swift ABI linkage;
- optional Python linkage;
- minimal-example size.

A secure-storage-only example must not silently pull camera/media/UI/Swift ABI/Python dependencies.

## Cache locality and data-layout policy

Memory footprint is a runtime performance concern, not merely a binary-size concern.

Framework-owned hot data should be designed to minimize working-set size and unnecessary cache misses.

Prefer, where appropriate:

- narrow integer fields when the domain bound is known;
- compact enum representations;
- bitsets and packed flags for dense boolean/state data;
- contiguous arrays/slabs over pointer-linked structures;
- compact IDs instead of full pointers where indirection is already required;
- hot/cold field splitting;
- field ordering that reduces padding;
- structure-of-arrays when only a subset of fields is traversed repeatedly;
- array-of-structs when whole records are consumed together.

Do not optimize solely for nominal struct byte count. Measure or reason about:

- `size_of::<T>()`;
- alignment;
- padding;
- element stride;
- objects per cache line;
- bytes touched per common operation;
- pointer chasing;
- decode/mask cost introduced by packing;
- branch behavior;
- vectorization opportunities.

### Bit packing

Bit packing is explicitly encouraged for bounded flags/state where it improves density and access locality.

Good candidates include:

- permission/capability flags;
- state-machine flags;
- callback-slot metadata;
- compact generational-handle metadata;
- small bounded enums;
- occupancy/free-slot bitmaps;
- feature masks.

Prefer explicit masks/bitfield helper types over `repr(packed)` when possible. Internal packed representations may differ from stable C ABI representations.

Every packed representation must define:

- bit allocation;
- valid/invalid values;
- overflow behavior;
- generation/counter wraparound;
- endian assumptions if serialized or externally visible;
- atomicity requirements if concurrently modified.

### Cache-regression validation

For performance-sensitive repeated types, track representative:

- struct size/alignment;
- collection footprint at realistic counts;
- traversal throughput;
- mutation throughput;
- allocation count;
- cache-miss-sensitive benchmarks where available.

A change that makes an API syntactically cleaner but materially inflates hot working sets should be treated as a performance regression unless justified.

## Baselines

For iOS UIKit work, compare:
1. Objective-C/UIKit;
2. Swift/UIKit;
3. direct Rust backend;
4. portable Rust API through the framework.

Where useful also compare:
- Rust-native API vs C ABI wrapper;
- C ABI vs C++ convenience wrapper;
- Python native operation cost separately from CPython interpreter cost.

SwiftUI may be measured separately, but it is not the zero-overhead UIKit baseline.

## Required benchmark dimensions

Measure where relevant:
- process launch;
- time to application delegate/root UI;
- RSS;
- allocations;
- live allocation count;
- retain/release traffic;
- callback latency;
- property mutation;
- text/data conversion;
- request construction;
- network completion/body copies;
- portable-wrapper overhead;
- C ABI overhead;
- async adapter overhead;
- binary size;
- linked library/framework set.

## Generated-code inspection

For important Rust hot paths, inspect optimized assembly or LLVM IR.

Verify that portable wrappers/backend selection inline or collapse as expected.

A direct ABI implementation may replace a higher-level backend helper only when it demonstrably removes cost or solves a correctness/ABI limitation.

## Claims

Do not claim "zero overhead" without qualifying the measured surface.

Preferred language:
- "no measured incremental overhead in scenario X";
- "same call topology as native implementation";
- "portable wrapper fully inlined";
- "one C ABI crossing";
- "allocation-free after setup";
- "copy-free for borrowed path."

## CI

Wall-clock microbenchmarks in CI are advisory.

Prefer deterministic gates:
- allocation count;
- copy count;
- absence of hot-path locks;
- callback-storage allocation;
- retain/release surplus;
- dependency/linkage set;
- code-size deltas;
- `--no-default-features` core builds;
- no accidental Swift ABI/Python linkage in native-only examples.

Physical-device Release measurements remain authoritative for platform performance claims.
