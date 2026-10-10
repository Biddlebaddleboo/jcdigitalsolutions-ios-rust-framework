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

## R3 residual — resilient-value layout

- Rust `SwiftRetained` already covers ActivityKit/AlarmKit class ownership
- The shared C header already has metadata layout, aligned alloc, and destroy/free helpers
- Source compare found one duplicate in AlarmKit `AuthorizationState` and Photogrammetry `Limits`: read VWT at `metadata[-1]`, require nonzero size and destroy witness, derive storage alignment
- AlarmKit also requires enum witnesses and `get_enum_tag`; Photogrammetry `Limits` is a resilient struct
- `swift_abi_value_storage_layout_from_metadata` now shares only the common metadata-to-layout path as C `static inline`
- The helper requires a compiler-proven metadata pointer with a readable preceding `SwiftValueWitnessTable` pointer, rejects zero size or invalid alignment, returns a borrowed table, and leaves valid outputs null/zero on failure
- The caller owns metadata memory validity and allocated value storage, then calls `swift_abi_destroy_and_free_value` once after successful Swift value init
- Each thunk retains its own init, enum tag, result mapping, and error codes; ActivityKit is unchanged because its scalar `Bool` path has no Swift value
- No Cargo feature, Rust API, Apple symbol, thunk signature, framework link, error code, or target floor changed
- C6 async no-go unchanged; no task entry/resume, executor, async, or cancellation support added

### R3 check evidence

- Before: AlarmKit and Photogrammetry had two direct `metadata[-1]` VWT reads and duplicate size/destroy/alignment checks
- After: both use `swift_abi_value_storage_layout_from_metadata`; device/Simulator IR still has the same Apple declarations, Swift calls, `sret`, enum tag, destroy, and link symbols
- PASS `cargo +1.94.1 fmt --all -- --check`
- PASS `cargo +1.94.1 check --locked --offline -p swift-abi-core -p swift-abi-generated`
- PASS `sh platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh` for device and arm64 Simulator
- PASS `sh platform/ios/ios-photogrammetry-status/check-swiftcall.sh` for device and arm64 Simulator
- PASS AlarmKit Swift oracle and C thunk IR compile for arm64 iOS 26.0 device and arm64 iOS 26.0 Simulator, using the oracle source and symbol record in `PLAN_CAPABILITIES_ALARMKIT.md`
- PASS `sh platform/ios/ios-alarmkit-status/check-link-imports.sh` for device and arm64 Simulator; link probes not run
- PASS `sh platform/ios/ios-photogrammetry-status/check-link-imports.sh` for device and arm64 Simulator; link probes not run
- PASS `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, and `git diff --check`
- No tests, host layout fixture, Simulator runtime, or physical-device query in this pass
- Xcode 26.6 / iOS SDK 26.5 remains below the Xcode 27.x qualification baseline; AlarmKit has no x86_64 Swift IR oracle
