# PLAN_REPLACEMENTS.md — Workstream E: Performance-Gated Rust Replacements

## Objective

Implement pure/portable Rust alternatives only for library-like work where equivalent semantics can be proven and Rust provides a meaningful performance/resource advantage on iOS.

This workstream does **not** pursue “zero Apple frameworks.”

## Dependencies

Requires A foundation and G parity/benchmark infrastructure.
Consumes Apple reference implementations from B only in tests/benchmarks.

## Write scope

Candidate pure Rust crates/modules under:
- `crates/framework-format/**`
- `crates/framework-compression/**`
- `crates/framework-crypto/**`
- `crates/framework-image/**`
- `crates/framework-pdf/**`
- `crates/framework-observation/**`
- relevant internal modules of network/persistence/ML/vision capabilities
- `benchmarks/replacements/**`
- `tests/parity/**` candidate-specific suites

## Replacement acceptance rule

A Rust candidate may become the default iOS backend only when:

1. the supported semantic subset is explicitly documented;
2. differential tests against the Apple reference pass;
3. regression fixtures exist for discovered mismatches;
4. Release benchmark on representative Apple hardware shows a meaningful win in the intended metric;
5. no unacceptable regression appears in latency, allocations, copies, memory, code size, energy, or maintenance complexity;
6. the Apple backend remains selectable if it has broader semantics or wins for another workload.

If evidence is ambiguous, keep Apple default.

## Strong initial candidates

### URL/URI/request construction
Candidate:
- framework-owned URL/URI parser/value model where semantics are bounded;
- request construction avoiding Objective-C object churn until the actual native network boundary.

Parity:
- selected Foundation URL behavior for claimed subset;
- explicitly document differences where RFC semantics intentionally differ.

### JSON/serialization
Use a mature minimal `no_std`-capable dependency if it wins the dependency/correctness tradeoff; do not write a fragile parser solely to remove a crate.

Benchmark:
- representative small/medium app payloads;
- allocations/copies;
- parse/serialize throughput.

### App-specific caches
Implement compact Rust cache structures tuned to known access patterns.
Do not claim NSCache-equivalent eviction semantics unless tested.

### Persistence
For non-CloudKit workloads, benchmark Rust/SQLite-style persistence against Core Data/SwiftData-like paths.
Use a mature SQLite binding/core rather than implementing a DB engine.
Do not replace Core Data where CloudKit/system integration is the actual requirement.

### Foundation small-data utilities
Candidates:
- byte/string transforms;
- value formatting without locale-specific semantics;
- small collections/state utilities;
- in-process event/observation.

Avoid replacing locale/calendar/system database behavior without parity.

### Natural language simple algorithms
Rust tokenization/parsing/classical features where no Apple pretrained model/service semantics are claimed.

### Image parsing/metadata
Only bounded formats with mature implementations and parity fixtures.
Keep ImageIO/CoreImage for broader/hardware/system-optimized work where they win.

### PDF parsing
Bounded parsing/extraction use cases only.
PDFKit remains platform rendering/UI backend if it wins or provides system integration.

### Software crypto
Prefer vetted mature RustCrypto-class implementations when appropriate and benchmarked.
Never replace Keychain/Secure Enclave/SecKey hardware/system semantics.
No homegrown cryptographic primitives.

### Compression
Benchmark cross-platform codecs.
Keep Apple Compression for LZFSE/LZ* where Apple wins or format is Apple-specific.

### Core ML pre/postprocessing
Strong Rust candidate to reduce conversions/copies while Core ML remains inference runtime.

### Classical CV/custom image processing
Rust/SIMD/Metal fused paths may replace Vision/CoreImage only for the exact operation when faster.

### Media/protocol parsing
Keep AVFoundation/VideoToolbox for capture/codec hardware; move container/protocol/application parsing to Rust where beneficial.

### Application protocol stacks
Use Network.framework only for transport/system policy; implement application protocols/state in Rust.

### Observation/reactive state
Framework-owned compact event/state observation instead of requiring NotificationCenter/Combine/Observation for internal Rust state.

### Transfer/convenience models
Portable Rust transfer semantics where CoreTransferable-like framework behavior is not required.

## Strong presumption against replacement

Do not replace by default:
- Accelerate/vDSP/BLAS/LAPACK;
- MPS/MPSGraph;
- VideoToolbox codecs;
- Core ML execution/Neural Engine scheduling;
- Core Animation compositor;
- ARKit tracking;
- Core Location fusion;
- AVFoundation capture;
- CoreAudio/AudioToolbox I/O;
- CoreVideo zero-copy integration.

A specialized experiment may exist only if benchmarking proves a concrete operation-level win.

## Assembly/SIMD policy

Optimization ladder:
1. idiomatic Rust;
2. inspect optimized compiler output;
3. intrinsics/portable SIMD;
4. ARM64/x86-64 handwritten assembly only if measurably superior.

For any handwritten assembly:
- portable Rust reference remains authoritative;
- CPU feature/ABI/alignment/clobber documented;
- randomized/edge differential tests;
- benchmark across representative hardware;
- remove assembly when compiler catches up.

## Parity methodology

Use same fixed/generated input corpus:
- Apple reference;
- Rust candidate;
- normalize only documented nondeterminism;
- compare result/error/side effects.

Do not weaken semantics to win benchmarks.

## Performance measurements

Where relevant:
- median/p95/p99 latency;
- CPU time;
- throughput;
- allocation count/bytes;
- copies/transcodes;
- RSS/hot working set;
- branch/cache behavior;
- code size/linkage;
- startup/setup;
- energy on device.

## Dependency policy

A pure Rust candidate is not automatically “better” if it adds a huge dependency graph.
Record:
- direct/transitive deps;
- features;
- no_std support;
- supply-chain/maintenance;
- replacement seam.

## Handoff

For each candidate output a decision record:
- Apple baseline;
- Rust candidate;
- semantic scope;
- parity result;
- benchmark result;
- selected iOS default;
- fallback policy.