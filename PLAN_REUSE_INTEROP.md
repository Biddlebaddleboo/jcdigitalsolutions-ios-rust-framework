# PLAN_REUSE_INTEROP.md — R3: reusable verified Swift/native interop primitives

## Implementation scope

Read `interop/swift-abi-core/src/{lib,retained}.rs`, `interop/swift-abi-generated/src/lib.rs`, `interop/swift-abi-core/tests/*` compiler oracle fixtures, ActivityKit/AlarmKit/Photogrammetry Rust and C bridge files, `PLAN_SWIFT_ABI.md` and `docs/SWIFT_ABI.md`. Proposed reusable modules under `interop/swift-abi-*/**` only where justified.

## Verified facts

`SwiftRetained` encapsulates Apple runtime retain/release behind optional `apple-runtime`, preserving no_std. String, Optional, scalar, ownership and async lowering proofs exist. No documented supported general Swift task entry/resumption API was established by the baseline. Recent production capability slices expose synchronous metadata/status, not complete async workflows.

## Required behavior

- First inventory each ABI primitive by actual compiler/SDK fixture, symbol/signature, target, deployment version, proof type and ownership/lifetime constraints.
- Extract only repeated, compiler-proven retained object lifetime, metadata/value witness, aligned temporary allocation/init/destroy, out-pointer validation and narrow error conversion patterns. Use metadata/witness flags; never assume a Swift value is bit-copyable.
- Leave native signatures and SDK quirks in each capability. Do not create a speculative generic `swiftcall` dispatcher or global type registry.
- Pilot shared object pattern between ActivityKit and AlarmKit only if signatures/ownership allow; separately factor a verified Photogrammetry witness pattern. Preserve native bridge crates.
- Maintain Rust-native Result, no_std, feature isolation, no mandatory executor, no shipping Swift source, and correct availability/error mapping.
- **Never mark Swift async, StoreKit 2 transactions, Translation or App Intents implemented based on compiler lowering alone.** Retain these as explicit blockers.

## Safety and deterministic tests

Cover failure after initialization, null pointers, unknown enum, alignment, ownership balancing, double-drop prevention, absent metadata, actor/thread restrictions and weak imports. Run existing compiler-oracle scripts; compare LLVM/assembly signatures on supported targets. Run `cargo +1.94.1 fmt --all -- --check`, locked checks for swift-abi-core/generated, all relevant ABI probes, pilot scripts, docs-check and zero-swift-source. Distinguish host tests, simulator compile/link and real runtime verification; report unavailable SDK/device checks as skipped.

## Ownership and handoff

R3 owns `interop/swift-abi-core/**` and `interop/swift-abi-generated/**` and scoped bridge source; R1 owns pilot build scripts, R2 owns check infrastructure. Report newly shared symbols, migrated call sites, before/after ABI evidence, errors/cancellation limitations, tests and SHA.
