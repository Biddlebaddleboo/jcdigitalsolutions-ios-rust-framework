# PLAN_BINDINGS.md — active Rust-to-C/C++/Python boundaries

## Verified state and first files
Primary source `bindings/c/**` (exports, ABI manifest, public headers, check scripts), portable `crates/framework-abi/**`, and `docs/VALIDATION.md`. Historical F1–F33 individual status and design details remain at Git `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_BINDINGS.md`. F17–F22 are narrow closed implementation slices documented in `PLAN_BINDINGS_COMPLETED_C_ABI.md`; all other F plans require their own completion evidence. A linked C/C++ probe is not a device runtime test.

## Invariants
Rust apps call Rust directly; C/C++/Python are opt-in outer bindings over the same core. Stable C ABI exposes only fixed-width scalars, versioned records where supported, explicit lengths, opaque handles, explicit statuses/ownership and callbacks. Never expose Rust layouts, Objective-C/Swift wrappers or Future directly. No panic unwinds across C. No implicit allocation/foreign runtime dependency in Rust-only builds. Compile/link/feature flags isolate Apple-specific bindings.

## Remaining implementation scope
- Audit F25–F33 and earlier unclosed F plans individually; don't infer integrated status from isolated worktrees or test scripts.
- F23 (`PLAN_BINDINGS_F23.md`) is integrated behind the opt-in `ios-key-support` feature: its ABI manifest, Cargo.lock, macOS CI gates, guide, and documentation links are present. The static and device/Simulator link-import gates pass; linked probes were not executed, and no tests or live query ran.
- F24 (`PLAN_BINDINGS_F24.md`) is integrated behind the opt-in `ios-mps-status` feature. Its dependency/feature, module/export, ABI manifest, Cargo.lock, macOS CI gates, guide, and documentation links are present. Both F24 gates passed on the `c55410d14c57c3ffdc1982cf62ae3180dd18c468` baseline. Linked C/C++ probes were inspected but not executed; no tests or live MPS query ran.
- Preserve async operation ownership: exactly-once completion when promised, correct callback/context lifecycle, no callback after destroy unless explicitly contracted; distinguish cancel/detach vs completion. Specific F5 transfer/F7 share semantics remain capability-owned; never substitute a generic process-wide operation registry.
- Finish optional C++ and Python layers only as explicitly scoped, preserving native ownership and failure/cancellation semantics; Python interpreter costs are opt-in and must not bleed into core.
- F34 (`PLAN_BINDINGS_F34.md`) adds and documents an explicit transfer-aware `OwnedBuffer::from_transfer` factory; acceptance is a C++17 syntax-only consumer compile for both transferred and absent results, with no tests or runtime claim.
- F35 (`PLAN_BINDINGS_F35.md`) adds an optional direct-owned opaque C error-detail object with fixed-width stored status, copied UTF-8 diagnostics, a borrowed view, and explicit destruction; it preserves the existing `FrameworkErrorHandle(u64)` layout.
- F36 (`PLAN_BINDINGS_F36.md`) adds an isolated optional PyO3 extension with an immutable Python `OwnedBytes` value over the existing owned-buffer C ABI; it adds no PyO3 or CPython dependency to Rust/C workspace builds.
- F37 (`PLAN_BINDINGS_F37.md`) adds a C++17 move-only `framework::ErrorDetail` owner and borrowed `std::string_view` access for the F35 direct-owned error-detail handle; stored status codes pass through unchanged.
- Validate header manifest parity, symbol exports, ABI version negotiation and stable error mapping. No silent wider same-major struct forward compatibility if inputs never promised it.

## Validation and handoff
Focused package scripts, release archive and C11/C++17 consumers, Rust native/C feature isolation, wrong-alignment/null/length/overflow tests, async race/drop/cancellation tests, no unrelated Apple frameworks, no Swift/Python linkage for Rust-only. Report SHA, exact changed source/header/manifest, consumer and runtime evidence, skipped checks and remaining risks.
