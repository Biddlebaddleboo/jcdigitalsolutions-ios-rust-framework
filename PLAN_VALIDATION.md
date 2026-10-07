# PLAN_VALIDATION.md — Workstream G: Tooling, Parity, Performance, CI and Documentation

## Objective

Build the shared machinery that makes the architectural rules enforceable: toolchain manifests, no_std gates, SDK inventory, parity infrastructure, linkage/dependency audits, ABI audits, performance benchmarks, codegen inspection, archive smoke tests, and documentation freshness.

## Dependencies

Foundation must establish workspace/crate names first.
Thereafter this workstream runs continuously and does not own capability implementation.

## Write scope

- `tools/xtask/**`
- `tools/sdk-inventory/**`
- `tools/swift-oracle/**`
- `tools/linkage-audit/**`
- `tools/abi-audit/**`
- `tests/parity/**` shared harness
- `tests/abi/**`
- `tests/linkage/**`
- `benchmarks/**` shared harness
- CI workflows
- global support/generated reports
- global docs indexes

## Toolchain manifest

Command should capture:
- macOS version/arch;
- Xcode path/version/build;
- Swift compiler version;
- clang/LLVM version;
- Rust toolchain;
- iphoneos and simulator SDK paths/versions;
- target triples/deployment versions.

Store reproducibility manifest as build/test artifact, not necessarily a committed machine-specific file.

## no_std enforcement

CI must build every portable crate:
- with default features appropriate to portable use;
- with `--no-default-features`;
- without linking `std`.

Add a check that platform/test/dev dependencies do not accidentally become normal portable dependencies.

## Dependency audit

Track:
- direct deps;
- transitive deps;
- duplicate versions;
- enabled features;
- proc-macro/build deps;
- std requirements;
- linked native/system libs.

Fail or require explicit approval when a narrow capability adds disproportionate graph growth.

Do not optimize for dependency count at the expense of security/correctness.

## SDK capability inventory

Generate machine-readable inventory from installed SDK:
- frameworks/modules;
- Objective-C headers;
- public C symbols/headers;
- `.swiftinterface` declarations;
- availability;
- `@objc` exposure;
- async/throws;
- generic signatures;
- actor isolation;
- relevant protocol conformances.

Use it to detect SDK drift and update the human support matrix.

Do not treat generated inventory as a replacement for public API/compliance review.

## Swift oracle harness

For selected Swift-only public APIs:
- generate tiny temporary Swift reference call;
- emit SIL;
- emit LLVM IR;
- compile optimized object;
- inspect undefined/defined symbols;
- inspect assembly;
- compare with framework thunk.

Repository/shipping source remains Swift-free; oracle source is test/tool input only.

## Parity harness

Shared differential API:
```text
fixture/generator
 -> Apple reference adapter
 -> Rust candidate
 -> comparator
 -> regression fixture on mismatch
```

Support:
- fixed fixtures;
- property/generated inputs;
- documented nondeterminism normalization;
- OS-version expected differences;
- error comparison;
- serialization compatibility where claimed.

## ABI audit

Check:
- C ABI layout;
- symbol export list;
- thunk calling convention;
- Swift runtime symbol provenance/availability;
- panic containment;
- ownership creator/destroyer pairs.

## Linkage audit

For minimal examples:
- Mach-O imported dylibs/frameworks;
- Swift runtime linkage when not requested;
- Python absence;
- unrelated capability absence;
- binary size.

Required examples:
- portable core;
- preferences;
- secure storage;
- network;
- location;
- UI;
- StoreKit/Translation Swift residual when enabled.

## Performance harness

Authoritative iOS performance claims require physical-device Release measurements where hardware/system services matter.

Harness supports:
- latency distributions;
- CPU;
- allocations;
- copies;
- RSS;
- hot working set;
- code size;
- startup;
- energy where available.

CI microbenchmarks are advisory; deterministic structural gates may fail CI.

## Codegen audit

For zero-cost wrappers/hot kernels:
- emit LLVM IR/assembly;
- verify static dispatch/inlining;
- verify no unexpected allocation/lock/C ABI round trip;
- compare compiler/intrinsics/asm variants.

## Archive smoke

Using Xcode public tooling:
- build Release device artifact;
- bundle/link;
- sign where credentials/environment permit;
- archive;
- verify no `.swift` shipping source;
- verify framework/runtime linkage;
- inspect entitlements/Info.plist expectations.

App Store upload itself is not required for every CI run.

## CI matrix

At minimum:
- format;
- clippy;
- unit tests;
- no_std checks;
- simulator build;
- C header compile;
- dependency/linkage audit;
- docs;
- zero-Swift-source check.

Periodic/manual:
- device integration;
- physical benchmarks;
- archive/signing;
- entitlement-specific capabilities.

## Documentation infrastructure

Generate/index:
- support matrix;
- capability availability;
- benchmark decisions;
- dependency inventory;
- ABI version manifest.

Do not overwrite hand-written semantic docs with generated noise.

Require each workstream to update:
- rustdoc;
- capability guide;
- architecture note if invariant changed;
- parity/performance decision where relevant.

## Final independent audit

Before V1 signoff:
1. verify current `main` against PLAN.md;
2. compare every workstream output to its plan;
3. inspect all dependency additions;
4. inspect unsafe/assembly inventory;
5. inspect `std` leaks;
6. inspect all linked Apple frameworks;
7. verify unsupported/deferred capability list;
8. rerun cross-workstream tests;
9. review final diff for accidental architectural drift;
10. ensure PLAN files are deleted before final implementation completion.

## Handoff

Report:
- tool/CI commands;
- generated support matrix;
- parity coverage summary;
- benchmark decision summary;
- ABI/linkage/dependency audit results;
- unresolved environment-only validation (for example entitlement or physical-device access).