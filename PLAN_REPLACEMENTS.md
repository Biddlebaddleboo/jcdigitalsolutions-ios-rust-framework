# PLAN_REPLACEMENTS.md — Workstream E: Performance-Gated Rust Replacements

## Status and evidence

E1 reviewed foreground HTTP URL/request construction and accepted no Rust replacement. The
code-level decision record is [`docs/decisions/http-request-construction.md`](docs/decisions/http-request-construction.md): a borrowed Rust request cannot replace the Foundation request required by URLSession, and reimplementing URL parsing or transport has no demonstrated equivalent-workload win. Foundation and URLSession remain the iOS default. No production candidate, dependency, parity suite, or replacement benchmark was added. No Apple differential parity or representative physical-device Release result exists; host conversion/harness fixtures are not such evidence.

The inventory below is a source/evidence audit, not a parity or performance experiment. It closes no candidate by performance: absent or unscoped candidates are deferred, and future candidates still require their own bounded semantic contract, Apple differential suite, and representative-device Release win before they can replace an Apple default.

The shared [`parity-harness`](docs/TESTING_AND_PARITY.md) and [`bench-harness`](docs/PERFORMANCE.md) are infrastructure only. The checkout has no registered Apple reference adapter, candidate parity suite, or replacement workload, and no representative physical-device Release measurement ([validation limits](docs/VALIDATION.md)). Those prerequisites are unavailable for every family below; host harness/conversion fixtures do not establish Apple parity or device performance.

| Candidate family | Current Apple behavior and Rust candidate in this checkout | Evidence-backed disposition |
| --- | --- | --- |
| URL/URI/request construction | `framework-format::Uri` is a bounded borrowed RFC 3986 parser, and `framework-network::HttpUrl` is shallow HTTP input validation. Neither replaces the Foundation request required by URLSession. | E1 defers Foundation request/URLSession replacement; see [`PLAN_REPLACEMENTS_HTTP.md`](PLAN_REPLACEMENTS_HTTP.md) and the [decision record](docs/decisions/http-request-construction.md). URI parser parity with Foundation and device performance remain unmeasured. |
| JSON/serialization | No general JSON/serialization candidate is present. `platform/ios/ios-transfer/src/record.rs` encodes only the framework's private durable-transfer journal; it is not a general serializer or replacement for Foundation serialization. | No general Apple serialization behavior is identified as replaced. No parity suite or device workload exists; feasibility remains open. |
| App-specific caches | `framework-alloc` provides `BitSet` and `GenerationalSlab`; neither is a cache nor claims `NSCache` eviction semantics. | No cache behavior is currently replaced and no cache candidate or cache-specific parity/benchmark workload exists. |
| Persistence | `framework-files`, `framework-preferences`, and `framework-transfer` define narrow app-owned contracts with POSIX/NSUserDefaults/URLSession iOS backends. No SQLite, Core Data, or SwiftData replacement is present. | Existing adapters do not replace Core Data/SwiftData or CloudKit semantics. A workload-specific non-CloudKit candidate, parity suite, and device benchmark remain open. |
| Foundation small-data utilities | Rust-native byte/UTF-8 views, URI values, IDs, bit sets, and slabs exist in `framework-data`, `framework-format`, `framework-core`, and `framework-alloc`; no one-for-one Foundation API replacement is claimed. | No specific Apple behavior has been selected for replacement. The helpers have no Apple differential suite or representative-device Release comparison. |
| Natural-language simple algorithms | No natural-language, tokenization, or classical-feature candidate crate/module is present. | No Apple model/service behavior is replaced; no bounded algorithm, parity suite, or device benchmark has been selected. |
| Image parsing/metadata | `framework-image` defines a portable source-metadata contract and `ios-image-io` reads image-zero dimensions/source count with ImageIO. Neither is a Rust parser/metadata-extraction candidate or an ImageIO/CoreImage replacement. | No explicit image-format support matrix, Apple parity suite, or representative-device benchmark exists. |
| PDF parsing | No PDF parser/extraction candidate is present; no PDFKit behavior is replaced. | No bounded document subset, Apple parity suite, or representative-device benchmark exists. |
| Software crypto | `ios-crypto` wraps Apple's CommonCrypto `CC_SHA256`; it is an Apple-backed platform wrapper, not a Rust cryptographic candidate or portable `framework-crypto` contract. `framework-secure-storage` and `ios-secure-storage` expose opaque bytes through Keychain; they do not implement cryptography or replace Secure Enclave/SecKey behavior. | No Rust cryptographic operation is proposed for replacement; no parity suite or device benchmark exists. Do not add homegrown primitives. |
| Compression | No compression candidate or codec dependency is present; Apple Compression behavior is not replaced. | No codec/format subset, Apple parity suite, or representative-device benchmark exists. |
| Core ML pre/postprocessing | No ML or Core ML preprocessing candidate module is present; Core ML inference is not replaced. | No concrete conversion/fusion operation, parity suite, or representative-device benchmark exists. |
| Classical CV/custom image processing | No CV/image-processing candidate module is present; no Vision/CoreImage operation is replaced. | No exact operation, Apple parity suite, or representative-device benchmark exists. |
| Media/protocol parsing | No media/container parser candidate is present. `framework-network` provides HTTP values and `ios-network` uses URLSession; neither is a media parser or replacement for AVFoundation/VideoToolbox. | No bounded parser or Apple parity/device benchmark exists. Capture and hardware codec behavior stays Apple-backed under the plan's presumption. |
| Application protocol stacks | `framework-network` defines foreground HTTP request/response values and `framework-connection` defines an outbound secure TCP byte-stream contract; `ios-network` and `ios-connection` delegate transport to URLSession and Network.framework, respectively. No Rust transport/protocol stack replaces those Apple services. | No same-contract stack candidate, Apple differential suite, or representative-device benchmark exists. E1 separately defers replacing URLSession request/transport behavior. |
| Observation/reactive state | No `framework-observation` crate is present. `framework-async` has executor-neutral one-shot futures, not a replacement for NotificationCenter, Combine, or Observation. | No observation API behavior is currently replaced; no parity suite or device benchmark exists. |
| Transfer/convenience models | `framework-transfer` defines a portable durable GET-download contract and `ios-transfer` implements it with Foundation URLSession. This is a Rust-owned contract over an Apple backend, not a CoreTransferable replacement. | The Rust model exists, but no CoreTransferable parity claim or replacement performance result exists; Apple transfer/system behavior remains in use. |

The plan's separate strong-presumption group (Accelerate/vDSP/BLAS/LAPACK, MPS/MPSGraph, VideoToolbox codecs, Core ML execution, Core Animation, ARKit, Core Location fusion, AVFoundation capture, CoreAudio/AudioToolbox I/O, and CoreVideo zero-copy) has no operation-level Rust replacement or winning device measurement in this checkout. Keep those Apple paths by default; this inventory does not claim they were benchmarked.

## Objective

Implement pure/portable Rust alternatives only for library-like work where equivalent semantics can be proven and Rust provides a meaningful performance/resource advantage on iOS.

This workstream does **not** pursue “zero Apple frameworks.”

## Dependencies

Requires A foundation and G parity/benchmark infrastructure.
Consumes Apple reference implementations from B only in tests/benchmarks.

## Execution decomposition

`PLAN_REPLACEMENTS_HTTP.md` (E1) is an evaluation-only review of foreground HTTP URL/request construction. It may conclude that no Rust replacement is justified; absent parity and representative-device performance evidence, Foundation and URLSession remain the default.

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
