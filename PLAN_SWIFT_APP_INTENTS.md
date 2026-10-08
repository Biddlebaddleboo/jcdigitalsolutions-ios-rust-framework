# PLAN_SWIFT_APP_INTENTS.md — Workstream C7: App Intents Metadata Feasibility

## Objective

Audit the normal Xcode App Intents metadata pipeline and determine whether the Stage 1 Rust-defined, zero-Swift-source attempt has a documented, stable input path. Record a precise go/no-go boundary; do not infer support from tool presence or exported symbols.

## Dependencies

- Foundation A and the repository zero-Swift-source invariant are integrated
- Workstream G toolchain/SDK inventory is available
- C6 async task-entry audit is informative for Stage 1 but is not required for the Stage 0 pipeline observation

## Read first

- `PLAN_SWIFT_ABI.md`, Phase 8
- `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`
- `docs/VALIDATION.md`
- existing Swift ABI toolchain and compiler-oracle reports

## Write scope

- `docs/swift-abi/APP_INTENTS_STAGE0.md`

Do not add or retain Swift application source, generated shipping Swift source, build artifacts, private APIs, private metadata formats, runtime registration, or App Intents capability claims. Any temporary Swift compiler fixture must remain outside the checkout and must not be packaged.

## Required audit

1. Record the exact Xcode, Swift, SDK, and host versions. State whether the host meets the repository's Xcode 27.x baseline.
2. Observe a minimal normal Xcode App Intents build using only public documented entry points and ephemeral input. Record the compiler and metadata-processor steps, their declared inputs/outputs, and the bundle relationship for `Metadata.appintents` and `AppIntentsPackage` where the toolchain exposes them.
3. Distinguish public, documented Xcode interfaces from implementation details. Do not parse or generate an undocumented metadata format.
4. Decide whether a parameterless Rust-defined AppIntent with `perform()` reaching Rust can be attempted using supported compiler/build mechanisms without shipping Swift source. If not, stop with the exact missing documented contract and mark Stage 1 unsupported for this toolchain; do not try private workarounds.
5. Cross-reference C6's async task-entry result when assessing whether `perform()` can safely cross into Rust.

## Validation and handoff

- Keep all temporary fixtures and products outside the repository.
- Confirm no `.swift` source or build artifact was added to the checkout.
- Run `cargo xtask docs-check` and `git diff --check`.
- Report exact toolchain versions, observed public commands/artifacts, unsupported/private boundaries, changed files, commit SHA, validation, deviations, and unresolved assumptions. Do not push.
