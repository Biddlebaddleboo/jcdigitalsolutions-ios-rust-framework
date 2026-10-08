# PLAN.md — V1 iOS Rust Framework Implementation

## Status

Planning set generated against repository:

- Repository: `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework`
- Target branch: `main`
- Verified planning baseline: `3fa8222e6e6fff6370ac461a00fa6e61e9e84a6d`
- Required build baseline: macOS with Xcode 27.x; observed local host is Xcode 26.6 build 17F113, below that baseline
- Primary V1 runtime target: iOS arm64
- Secondary V1 target: iOS simulator arm64
- Future targets preserved architecturally: macOS, Android, Windows, Linux, Web/WASM, and a future compact/32-bit-pointer internal mode

This plan is authoritative for V1 implementation. At the planning baseline, the repository held architecture and research docs only. The current checkout has a Cargo workspace, portable foundation, tooling, selected capability contracts and iOS backends, bindings, and validation. Unimplemented paths remain proposed additions; see the workstream plans and capability status manifest for current scope.

## Objective

Implement the first complete iOS backend of a reusable, high-level, platform-agnostic native application framework primarily in Rust, with:

- Rust as the direct fast path;
- a genuine `#![no_std]` portable core from the first implementation commit;
- `alloc` only where required;
- public iOS system capabilities exposed through Rust without a Swift application layer;
- public Objective-C APIs reached through `objc2`/generated bindings unless a smaller supported C path exists;
- public C/CoreFoundation/Darwin APIs called directly where preferable;
- genuinely Swift-only APIs reached through a small, capability-scoped Swift ABI subsystem with no repository-authored or generated `.swift` source;
- a stable modular C ABI for foreign-language consumers;
- optional later Python/C++ bindings without contaminating the core;
- compact/cache-conscious framework-owned state;
- future 32-bit offset/compressed-pointer compatibility without implementing pointer compression in V1;
- dependency minimization without replacing mature/security-sensitive dependencies with fragile custom code;
- Apple differential parity tests for Rust reimplementations;
- benchmark gating before any Rust implementation replaces an Apple library implementation on iOS;
- ARM64/x86-64 handwritten assembly only for measured hot paths where it is reliably faster and remains small;
- continuously updated developer and maintainer documentation.

“All APIs in Rust” means the developer-facing framework surface and application logic are Rust-native. It does **not** mean recreating Apple-owned protected databases, daemons, hardware drivers, entitlement systems, system UI, store authority, GPU/media engines, or cloud services.

## Verified repository facts

1. At planning baseline `3fa8222e6e6fff6370ac461a00fa6e61e9e84a6d`, the repository contained architecture/research docs only; no Cargo workspace or implementation crates existed. This is a historical baseline fact, not the current checkout state.
2. Most ordinary iOS capability families have public C or Objective-C routes and therefore do not require Swift ABI interoperability.
3. Swift interoperability is a residual subsystem, not the foundation.
4. The conservative V1 Swift call backend is microscopic Clang/LLVM ABI adaptation using `swiftcall` / `swiftasynccall`, with generated/compiler-oracle lowering. Nightly rustc Swift ABI support is experimental and must not be a V1 dependency.
5. Translation is the preferred first real Swift-ABI framework proof. StoreKit 2 is the stronger later forcing function.
6. Full App Intents is primarily a compiler/build metadata problem and must remain isolated from the ordinary Swift call layer.
7. Apple differential testing is required for any Rust implementation that claims Apple-compatible semantics.
8. Rust replacement of Apple/library behavior is performance-gated. Apple remains the default when equal/faster or uniquely system-integrated.
9. Portable core/public contracts must be `no_std`-compatible from the start.
10. V1 targets normal 64-bit Apple environments but must not bake 64-bit pointer width into portable semantics, stable ABI, serialized formats, IDs, or persistent indexes.
11. Framework-owned hot state should use compact layouts, narrow fields, bitsets/packed state, arenas/slabs/contiguous storage where beneficial, while keeping the public API ergonomic.
12. Dependencies must remain behind bounded internal seams and should be kept to the smallest dependable set.

## V1 definition of done

V1 is complete when all of the following are true:

### Architecture and build
- A Cargo workspace exists with independently usable portable, capability, iOS backend, Swift ABI, binding, tooling, benchmark, and example crates.
- Portable crates build as real `#![no_std]` crates with `--no-default-features`.
- iOS device and simulator builds succeed from macOS/Xcode.
- A minimal Rust-owned iOS application reaches `UIApplication -> Rust-defined delegate -> UIWindow -> UIViewController -> UILabel/UIButton -> Rust callback` without Swift application code.
- The repository contains no `.swift` source and the build generates no shipping Swift source.
- Release archive/link validation succeeds for the supported V1 packaging path.

### API and capability coverage
The framework has Rust-facing support, or an explicitly documented system/build-boundary implementation path, for the V1 capability families enumerated in `PLAN_CAPABILITIES.md`.

### Swift-only residuals
- The common Swift ABI substrate is implemented and tested.
- Translation single-string flow works end-to-end.
- StoreKit 2 product retrieval and purchase work end-to-end where StoreKit test infrastructure permits.
- Concrete StoreKit transaction update iteration is implemented if current SDK lowering can be supported without broad protocol-runtime emulation.
- Tier-1 residuals from research are implemented where they fit the common Layer-1 substrate and are available/entitled on the test host.
- App Intents build-metadata Stage 0/1 is isolated and attempted as described in `PLAN_SWIFT_ABI.md`; failure of a supported zero-Swift-source path is documented and does not contaminate the rest of the architecture.

### Correctness/performance
- Rust replacements have parity suites and benchmark evidence before becoming the iOS default.
- System-owned/hardware-accelerated Apple paths remain Apple-backed unless a measured replacement wins.
- Hot common structures have recorded size/alignment/stride/capacity invariants.
- No avoidable mandatory runtime, global executor, global service registry, serialization bridge, or universal object model exists.
- Dependency/linkage/binary-size tests exist and minimal consumers do not pull unrelated capabilities.
- No handwritten assembly is accepted without the required reference implementation, parity tests, and measured win.

### Documentation
- Public Rust APIs have useful rustdoc.
- Capability guides document availability, permissions/entitlements, lifecycle, errors, cancellation, native escape hatches, parity status, and performance behavior.
- Architecture/research docs are updated when implementation proves a different fact.
- Each significant dependency has a recorded rationale and replacement seam.

## Proposed repository layout

The exact crate names may be adjusted only if Cargo/toolchain constraints require it; dependency direction and ownership are fixed.

```text
Cargo.toml
rust-toolchain.toml
deny.toml                         # only if cargo-deny is adopted after dependency review
.clippy.toml                      # only if needed

crates/
  framework-core/
  framework-alloc/
  framework-async/
  framework-abi/
  framework-platform/
  framework-app/
  framework-ui/
  framework-files/
  framework-preferences/
  framework-secure-storage/
  framework-network/
  framework-notifications/
  framework-location/
  framework-bluetooth/
  framework-motion/
  framework-camera/
  framework-audio/
  framework-media/
  framework-photos/
  framework-contacts/
  framework-calendar/
  framework-auth/
  framework-web/
  framework-cloud/
  framework-ml/
  framework-vision/
  framework-metal/
  framework-maps/
  framework-persistence/
  framework-nfc/
  framework-health/
  framework-calling/
  framework-background/
  framework-device-integrity/
  framework-game/
  framework-vpn/
  framework-nearby/
  framework-home/
  framework-accessory/
  framework-documents/
  framework-sharing/
  framework-payments/
  framework-apple-music/
  framework-weather/
  framework-observation/
  framework-transfer/
  framework-format/
  framework-compression/
  framework-crypto/
  framework-image/
  framework-pdf/

platform/
  ios/
    ios-runtime/
    ios-ui/
    ios-files/
    ios-preferences/
    ios-secure-storage/
    ios-network/
    ios-transfer/                       # proposed B13; gated on D10
    ios-notifications/
    ios-location/
    ios-bluetooth/
    ios-motion/
    ios-camera/
    ios-audio/
    ios-media/
    ios-photos/
    ios-contacts/
    ios-calendar/
    ios-auth/
    ios-web/
    ios-cloud/
    ios-ml/
    ios-vision/
    ios-metal/
    ios-maps/
    ios-persistence/
    ios-nfc/
    ios-health/
    ios-calling/
    ios-background/
    ios-device-integrity/
    ios-game/
    ios-vpn/
    ios-nearby/
    ios-home/
    ios-accessory/
    ios-documents/
    ios-sharing/
    ios-payments/
    ios-system-services/
    ios-extension-support/

interop/
  swift-abi-core/
  swift-abi-values/
  swift-abi-async/
  swift-abi-generated/
  swift-abi-apple/
  swift-build-metadata/

bindings/
  c/
  cpp/                             # optional V1 if cheap once C ABI exists
  python/                          # scaffold/optional; must not block core V1

tools/
  xtask/
  sdk-inventory/
  swift-oracle/
  linkage-audit/
  abi-audit/

tests/
  parity/
  integration/
  abi/
  linkage/

benchmarks/
  core/
  replacements/
  ios/

examples/
  ios-minimal/
  ios-capabilities/
  c-minimal/
```

## Shared public foundations and proposed symbols

`PLAN_FOUNDATION.md` owns these symbols. Other workstreams consume them and may not silently redefine their semantics.

### `framework-core`
Proposed public/internal foundations:

- `Platform`
- `Availability`
- `Capability`
- `CapabilityId`
- `PlatformErrorCode`
- `ErrorKind`
- `Error`
- `Result<T>`
- `Cancellation`
- `OperationId`
- `Generation`
- `CompactHandle`
- `PermissionState`
- `AuthorizationState`
- `NativeHandle<T>` only in platform-extension modules
- fixed-width time/duration primitives if needed without `std::time`

Requirements:
- `#![no_std]`
- no `alloc` unless a module proves it needs it
- no public `usize` semantic IDs
- no platform-native types
- no dependency-specific error types

### `framework-alloc`
Proposed internal reusable primitives:
- compact slab/arena primitives;
- generational handle table;
- small typed bitset/status word helpers;
- optional small-vector/storage helpers only if justified;
- hot/cold record patterns.

No public API should expose internal bit allocation.

### `framework-async`
Proposed:
- runtime-neutral `OperationState`;
- exactly-once completion primitive;
- `CancellationToken`/registration semantics;
- callback-first primitive;
- Rust `Future` adapter without a mandatory executor;
- FFI-safe completion adapter used by `framework-abi`.

### `framework-abi`
Proposed stable C-facing primitives:
- ABI version;
- fixed-width status codes;
- pointer+length slices/strings;
- owned buffer handles;
- opaque operation handles;
- cancellation;
- callback signatures;
- explicit create/destroy/retain/release functions where required;
- versioned extensible structs.

No Rust layout or third-party type may cross this boundary.

### `framework-platform`
Compile-time backend selection and platform capability markers only. No runtime registry.

## Workstream execution rule

Every named `PLAN_*.md` workstream must be executed by its own bounded subagent/executor. Parallel-safe workstreams must run concurrently in isolated branches/worktrees after prerequisites are integrated. No single agent should serially absorb multiple independent workstreams merely for convenience; if a workstream becomes too large, decompose it into additional named subplans before implementation. The orchestrator retains centralized integration and contradiction resolution.

## Workstreams

### Workstream A — Foundation
Plan: `PLAN_FOUNDATION.md`

Owns:
- root workspace/build policy;
- shared portable primitives;
- compact handles/state;
- runtime-neutral async/cancellation;
- stable internal semantic contracts;
- dependency policy implementation;
- initial docs/rustdoc conventions.

Must land first.

### Workstream B — iOS Native Runtime and Backends
Plan: `PLAN_IOS_NATIVE.md`

Depends on A.

Owns:
- objc2/C/CoreFoundation/Darwin boundary;
- Objective-C class/delegate/block machinery;
- iOS build/package bootstrap;
- capability-scoped iOS native backend crates;
- extension/system-owned service shells.

May proceed in parallel with C and D once A interfaces are integrated.

### Workstream C — Swift ABI and Compiler/Metadata Residuals
Plan: `PLAN_SWIFT_ABI.md`

Depends on A and selected tooling contracts from G.

Owns:
- Clang/LLVM Swift ABI thunk machinery;
- metadata/value witnesses;
- Swift String/Array/Optional/concrete enums/generic instances;
- async/throws/cancellation adapters;
- Translation, StoreKit 2 and other Tier-1 residual adapters;
- App Intents/build-metadata experimental subsystem;
- generated SDK ABI fixtures.

Must not modify ordinary native iOS backend ownership rules.

### Workstream D — Portable Capability Facades and Full iOS Capability Assembly
Plan: `PLAN_CAPABILITIES.md`

Depends on A; each capability implementation consumes B or C backend pieces.

Owns:
- developer-facing high-level capability crates;
- semantic portable contracts;
- platform extensions;
- capability-specific rustdoc/guides;
- full capability coverage matrix.

Capability modules may be implemented in parallel after their shared contracts are approved.

### Workstream E — Rust Replacement Candidates
Plan: `PLAN_REPLACEMENTS.md`

Depends on A and G benchmark/parity harness. May run parallel with B/C/D.

Owns:
- pure Rust candidate implementations for library-like work;
- differential Apple reference adapters used only in tests;
- selection gates deciding Rust vs Apple default per operation.

Must not reimplement system-owned/hardware-accelerated services simply to remove Apple dependencies.

### Workstream F — Stable Foreign-Language Bindings
Plan: `PLAN_BINDINGS.md`

Depends on A and stable capability contracts from D.

Owns:
- stable C API;
- generated/maintained C headers;
- C++ convenience layer if feasible without new runtime cost;
- optional Python binding scaffold after C/Rust semantics are stable.

Rust-native API must never route through this layer.

### Workstream G — Validation, Tooling, Performance and Documentation Infrastructure
Plan: `PLAN_VALIDATION.md`

Begins after A establishes crate names; thereafter runs alongside all workstreams.

Owns:
- xtask/tooling;
- SDK inventory;
- Swift oracle generation;
- parity harness framework;
- linkage/ABI audit tooling;
- benchmark harness;
- no_std/dependency/binary-size/codegen gates;
- global documentation indexes and final V1 audit.

Individual workstreams own tests/docs next to their code; G owns shared harnesses and cross-workstream validation.

## Dependency graph and parallel safety

```text
A Foundation
|
+----> B iOS Native ------------------+
|                                     |
+----> C Swift ABI -------------------+----> D capability assembly
|                                     |          |
+----> G validation/tooling ----------+          |
|                                                v
+-----------------------------> E replacements --+--> F foreign bindings
                                                  |
                                                  v
                                             final integration
```

Parallel-safety rules:

- A is integrated first.
- After A, B/C/G can proceed in isolated branches/worktrees.
- D capability crates may begin when the relevant A contract is stable; each capability waits only for the backend primitives it needs.
- E can prototype pure Rust replacements as soon as G provides benchmark/parity harness contracts.
- F starts only after the relevant developer-facing capability contract is stable enough to freeze a C ABI.
- No workstream may modify another workstream's owned shared symbols without central reconciliation.
- Root `Cargo.toml` should use workspace glob patterns established by A so later workstreams can add crates without editing the root member list.
- Shared dependency versions/features are centralized in workspace dependencies by A; later additions require explicit dependency rationale and conflict review.

## Capability implementation classification

For every capability, executor must record one of:

- **R** — pure/portable Rust implementation selected after parity/performance proof;
- **M** — Rust semantics/state machine over a minimal public Apple primitive;
- **B** — Apple/system-owned backend reached through Rust;
- **A** — Apple implementation preferred for performance/hardware reasons;
- **C** — compiler/build/discovery contract requiring packaging metadata;
- **X** — unsupported/deferred because a public zero-Swift-source path is not yet proven.

A capability may have multiple labels (for example H/A behavior is represented as Rust facade + Apple accelerated backend).

## Global invariants

1. No `.swift` files in repository or generated shipping sources.
2. No private frameworks, selectors, daemons, symbols, entitlements, or App Review bypasses.
3. Portable crates are `no_std` from their first implementation.
4. No mandatory Tokio/async-std or universal executor.
5. No process-global `Framework::initialize()`.
6. No universal dependency-injection/service-locator/runtime registry.
7. No virtual DOM or duplicate platform UI object model.
8. No C ABI round-trip for Rust callers.
9. No dependency-specific types in portable public APIs.
10. No speculative assembly.
11. No 64-bit pointer-width semantic assumptions.
12. No performance replacement without parity/correctness first.
13. No fragile in-house crypto/ABI/parser replacement solely to lower dependency count.
14. Panics never unwind across FFI.
15. Objective-C/Swift ownership follows native ownership; do not wrap every native object in `Arc`.
16. Main-thread restrictions are typed/platform-specific.
17. Async operations have explicit cancellation and exactly-once completion.
18. Public API remains semantic/ergonomic; packed layouts stay internal.

## Integration order

1. Integrate A.
2. Integrate G bootstrap/no_std/toolchain/linkage/parity harness.
3. Integrate B native iOS runtime and minimal app vertical slice.
4. Integrate C Swift ABI synchronous foundation, then async foundation.
5. Integrate D capabilities in batches, using B/C backends.
6. Integrate E only for candidates that pass parity + performance gates.
7. Integrate F after capability contracts stabilize.
8. Run complete cross-workstream validation.
9. Independently compare final diff against this PLAN.md and every workstream plan.
10. Resolve contradictions centrally; do not let a workstream silently redesign architecture.
11. Delete all `PLAN*.md` files before final implementation commit/release branch merge.

## Cross-workstream validation

Minimum commands/activities once implemented:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check -p framework-core --no-default-features
cargo check -p framework-alloc --no-default-features
cargo check -p framework-async --no-default-features
cargo check -p framework-abi --no-default-features
cargo tree --workspace
cargo tree -d
cargo doc --workspace --no-deps
```

Plus macOS/Xcode tasks provided by `tools/xtask`:

```text
xtask toolchain-manifest
xtask ios-build --simulator --release
xtask ios-build --device --release
xtask ios-minimal-link-audit
xtask swift-abi-oracles
xtask parity --ios
xtask abi-audit
xtask dependency-audit
xtask size-audit
xtask codegen-audit
xtask archive-smoke
```

Exact CLI spelling may change during implementation, but equivalent functionality is required.

## Final diff checklist

Before V1 is accepted:

- [ ] No `.swift` source exists.
- [ ] Portable crates genuinely compile without `std`.
- [ ] No portable API exposes platform/dependency types.
- [ ] No new mandatory runtime/global registry/executor.
- [ ] Capability crates remain independently linkable.
- [ ] Minimal consumers do not pull unrelated frameworks.
- [ ] Public C ABI contains no Rust layout.
- [ ] Panics cannot cross FFI.
- [ ] Objective-C/Swift retain/release/destroy paths are stress-tested.
- [ ] Async cancellation/completion races are deterministic and tested.
- [ ] Rust replacements have Apple parity fixtures.
- [ ] Rust replacements selected by default have benchmark evidence.
- [ ] Hardware/system-owned Apple capabilities remain Apple-backed unless a measured exception exists.
- [ ] Handwritten assembly has portable reference, differential tests, CPU/ABI gating, and meaningful measured win.
- [ ] Hot structures have size/alignment/capacity records.
- [ ] Dependency additions have rationale and minimal features.
- [ ] No public semantic ID/serialized field accidentally depends on `usize`.
- [ ] iOS simulator/device builds pass.
- [ ] Release archive/link smoke passes.
- [ ] Developer docs and capability support matrix are current.
- [ ] Every executor reports changed files, commit SHA, tests, deviations, unresolved assumptions.
- [ ] PLAN files are removed before the final implementation commit.

## Executor handoff

The orchestrator should start implementation with:

> Implement `PLAN.md` exactly. Verify latest `main` first. Read the assigned `PLAN_*.md` before coding. Stay within scope unless current code, compilation, tests, moved symbols, compatibility, or correctness require expansion. Do not silently redesign shared architecture. Run the required tests, inspect the diff, report changed files/commit SHA/tests/deviations/unresolved assumptions, and let the orchestrator integrate in PLAN.md order. After final independent validation, delete all PLAN*.md files before the implementation is finalized.
