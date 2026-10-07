# Testing and Apple Parity Policy

## Purpose

Testing must prove both correctness and substitution safety.

For ordinary framework code, tests verify the framework contract.

For a Rust implementation intended to replace an Apple library-layer implementation, tests must also demonstrate behavioral parity for the supported contract before the Rust path becomes the default Apple backend.

Performance alone is insufficient.

## Shared parity harness

The dependency-free `parity-harness` package under `tools/parity-harness` provides fixed `Case`
inputs, typed `Outcome::Success` and `Outcome::Error` results, and a generic `Adapter` interface.
`compare_suite` uses static generic dispatch for the reference/candidate pair; it does not contain
Apple frameworks or register any real suite.

`Normalization::ignore_top_level_fields` is explicit per suite and ignores only the named
top-level fields in successful object results. Error categories and optional native codes are never
normalized. Reports list every normalized field. Each mismatch preserves the case ID and input, raw
reference/candidate results, normalized reference/candidate results, and caller-supplied OS/SDK
labels. Its deterministic JSON serializer rejects non-finite floats rather than emitting invalid
JSON.

The crate's unit fixtures use deterministic fake adapters to cover equal values, unequal values,
unequal error categories/native codes, and explicit normalization. These fixtures validate the
comparator only; they are not Apple parity evidence. `cargo xtask parity` remains unavailable until
a real Apple reference adapter and Rust candidate suite are registered.

## Test layers

### 1. Pure portable unit tests

Run against `no_std`-compatible implementation code where practical.

The test harness itself may use `std`.

Cover:
- value semantics;
- compact/packed representations;
- state machines;
- error mapping;
- overflow/range behavior;
- cancellation state;
- ownership bookkeeping;
- deterministic algorithms.

### 2. Property and generated tests

Use property-based/generated inputs where a broad state space exists.

Good targets:
- parsers;
- serializers;
- URL logic;
- bit-packed IDs/state;
- numeric conversions;
- date/time arithmetic;
- protocol codecs;
- normalization.

Persist any generated failure as a deterministic regression vector.

### 3. Differential Apple parity tests

When replacing Apple library behavior, execute the same input against:

```text
Apple implementation
Rust implementation
```

Then compare normalized observable behavior.

The Apple implementation is the reference oracle for the subset the framework claims to replace.

Parity comparison should include where relevant:
- returned values;
- ordering;
- normalization;
- accepted/rejected inputs;
- error category/code;
- boundary and overflow behavior;
- Unicode/encoding behavior;
- cancellation;
- concurrency/thread restrictions;
- persistence/serialization compatibility;
- side effects;
- version-specific semantics.

Do not normalize away real semantic differences.

### 4. Platform integration tests

Test the real platform boundary for capabilities that cannot be replaced, including:
- Apple object lifetime;
- delegates/callbacks;
- main-thread constraints;
- cancellation;
- availability;
- entitlement/error handling;
- lifecycle and background behavior where testable.

Prefer fakes/local harnesses over external services when system behavior can be injected.

### 5. ABI tests

Validate:
- Rust/C ABI layout where specified;
- symbol availability;
- ownership transfer;
- panic containment;
- callback signatures;
- Swift ABI thunk shapes;
- generated headers;
- ABI version compatibility.

### 6. Minimal-link tests

Build tiny consumers using one capability at a time.

Verify:
- expected crate dependency graph;
- expected Apple framework imports;
- no accidental Swift ABI subsystem;
- no Python runtime;
- no unrelated UI/media frameworks;
- binary-size budget.

### 7. Performance tests

A Rust replacement may become the default only when:

1. parity/correctness tests pass;
2. performance benchmarks show the intended win;
3. the win is meaningful enough to justify maintenance.

Benchmark:
- latency;
- CPU time;
- allocations/bytes allocated;
- copies/transcodes;
- RSS/working set;
- cache-sensitive throughput where meaningful;
- startup/setup cost;
- binary/linkage delta;
- energy for sustained/hardware-accelerated workloads.

Use Release/LTO settings representative of shipping builds.

## no_std testing

Portable production crates must compile without `std`.

CI should include commands equivalent to:

```bash
cargo check -p framework-core --no-default-features
cargo check -p <portable-capability> --no-default-features
```

as crate structure becomes available.

Tests may link `std`; production portable crates may not.

## Apple parity harness structure

Preferred organization:

```text
tests/parity/
  fixtures/
  generators/
  apple_reference/
  rust_candidate/
  comparators/
  regressions/
```

The exact layout may differ, but Apple-reference code must stay out of the portable implementation.

## Version coverage

If Apple behavior differs by OS version:
- document the supported behavior range;
- run parity on representative minimum/current versions where practical;
- encode version-specific expectations explicitly.

Do not accidentally freeze one SDK/runtime behavior as universal if Apple documents version differences.

## Nondeterminism

Only normalize nondeterminism that is intrinsic/documented.

Examples:
- timestamps;
- generated identifiers;
- ordering explicitly documented as unspecified.

Do not normalize:
- incorrect ordering where ordering is specified;
- changed error classes;
- precision loss;
- missing fields;
- different normalization semantics.

## Concurrency and cancellation

For async APIs test:
- exactly-once completion;
- cancellation before start;
- cancellation during work;
- completion racing with cancellation;
- teardown;
- dropped futures/callback owners;
- reentrancy;
- no use-after-free;
- no duplicate action.

Use deterministic scheduling/fake clocks where possible.

## Unsafe/packed-state testing

For packed representations test:
- all valid bit patterns that correspond to public states where tractable;
- invalid/reserved patterns;
- encode/decode round trips;
- generation wrap behavior;
- overflow handling;
- alignment assumptions;
- concurrent atomic updates if applicable.

## Architecture-specific assembly testing

Every handwritten ARM64/AArch64 or x86-64 hot path must be tested against a portable Rust reference implementation.

Required coverage should include:
- fixed edge-case vectors;
- randomized/property-generated inputs;
- boundary lengths and alignments;
- zero/empty inputs where supported;
- maximum/minimum numeric values;
- overlapping/aliasing cases if the contract permits them;
- CPU-feature gated paths;
- fallback selection;
- architecture-specific regression cases.

For pure deterministic kernels, require byte/value-for-value equality unless the documented numeric contract permits tolerance.

For floating-point/SIMD code, define the accepted precision/rounding/NaN contract before comparing implementations.

Benchmarks must compare:
1. portable optimized Rust;
2. compiler intrinsic/SIMD implementation when applicable;
3. handwritten assembly candidate.

Handwritten assembly is accepted only if it produces a meaningful, repeatable win and does not sacrifice correctness, portability of the public API, or maintainability.

CI should compile all supported architecture paths where runners/toolchains permit, while physical/device benchmarks determine performance claims.

## Replacement acceptance checklist

Before changing an Apple backend to Rust-by-default:

- [ ] Supported semantic subset documented.
- [ ] Differential parity suite exists.
- [ ] Edge/error behavior covered.
- [ ] Prior mismatches have regression fixtures.
- [ ] Release benchmark exists.
- [ ] Rust wins the intended metric(s).
- [ ] No unacceptable regression in another critical metric.
- [ ] Platform/version differences documented.
- [ ] Apple fallback remains available when required by semantics/performance.
- [ ] Documentation updated.

## Principle

A Rust implementation is not a valid replacement merely because it compiles or benchmarks faster.

It must be **correct first, behaviorally compatible for the claimed contract, and then measurably better**.
