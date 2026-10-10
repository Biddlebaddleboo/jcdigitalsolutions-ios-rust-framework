# PLAN.md — Active framework implementation handoff

## Authority and baseline

Repository: `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework`; branch: `main`. Planning audit baseline: `f3b14892a560da05a086caf9e6d30a0c9beda25e`. **Before execution, verify the current remote `main` SHA and reconcile any relevant changes.** This file supersedes the former infrastructure-only stop instruction; it does not authorize indiscriminate implementation of every historical `PLAN_*.md` file.

## Objective and current state

Resume bounded work on genuinely unfinished framework capabilities, backends, bindings and cross-cutting requirements with R1 shared native build and R2 shared validation already implemented. R3 verified interop work is distinct: reconcile any truly unfinished R3 requirements against current source before authorizing changes. The completed reusable build and validation engines run as pinned, PATH-installed `ios-rust-build` and `ios-rust-validate` binaries. Their engine source was removed from ordinary `main` at `8b68d2909de9b5e926130265ba241c80ab653e99`; the immutable historical source and release provenance remain available for separately authorized maintenance. The operational contract is `docs/SHARED_TOOLING.md`; `AGENTS.md` remains authoritative for architectural, safety and agent behavior rules.

Historical capability coverage is **partial, not complete** merely because a crate or scalar getter exists. `docs/capabilities/capability-status.json` defines canonical capability identities and support states. Compiler, link, simulator-runtime and physical-device proofs remain distinct. Prior Xcode 26.6 / iOS SDK 26.5 findings do not satisfy an Xcode 27.x qualification requirement. Main CI failures previously seen on non-Apple `objc2` Clippy and the iOS Files import-order audit must be triaged, not hidden or misrepresented as passing.

## Active work selection and limits

1. Inspect the canonical capability status/owner, the relevant family plan, then the named workstream plan and exact files/symbols. Do not scan the entire planning corpus or treat every existing plan as pending work.
2. Classify the selected requirement against current implementation, tests and historical recorded decision: `complete`, `partial`, `blocked`, `superseded`, `not_started`, or `research_closed`. A prior `no-go` is a constraint, not an invitation to invent an implementation.
3. Implement only user-authorized remaining work. Do not select arbitrary new backlog items after finishing an assignment. Deferred work is preserved, not canceled.
4. Existing P/R/A/B/C/D/F/G identifiers and all capability IDs are permanent; never renumber, recycle or use retired gaps. For future workstreams allocate the next unused greater ID in its original series after consulting history.
5. Plans specifying a past source baseline must be reconciled to latest `main` before writing. Outdated commands do not override working architecture.

## Implementation ownership and dependencies

| Contract family | Authoritative entrypoint | Primary change owner |
|---|---|---|
| Portable semantics, `no_std`, error and lifecycle contracts | `PLAN_FOUNDATION.md`, `crates/framework-*/` | A / portable owners |
| Capability coverage, Apple platform feasibility and no-go decisions | `PLAN_CAPABILITIES.md`, canonical capability manifest, targeted `PLAN_CAPABILITIES_*.md` | D and associated backend owners |
| Native iOS implementations, app lifecycle, ObjC/Swift boundaries | `PLAN_IOS_NATIVE.md`, targeted `PLAN_IOS_*.md`, `platform/ios/*` | B |
| C ABI, C++ conveniences and optional Python bindings | `PLAN_BINDINGS.md`, targeted `PLAN_BINDINGS_*.md`, `bindings/*` | F |
| Runtime/compile/link/test requirements and evidence classifications | `PLAN_VALIDATION.md`, targeted `PLAN_VALIDATION_*.md`, `docs/SHARED_TOOLING.md` | G and each capability owner for local specs |
| Verified reusable Swift ABI primitives | `PLAN_REUSE_INTEROP.md`, `interop/swift-abi-*` | R3, only for verified remaining contract gaps |
| Performance replacements | `PLAN_REPLACEMENTS.md`, evidence-backed targeted plan | replacement owner |
| Engine bugs or unsupported tooling protocols | `BUG_REPORT_*.md`, separately authorized maintenance work | distinct tooling-maintenance Codex session, **not** API executor |

Minimize overlapping writes. If shared interfaces or files are required, assign one owner; other workstreams consume them. Explicitly order dependencies and integrate sequentially across such hotspots. Parallelize only independent work in isolated worktrees.

## Shared tooling contract for every assigned workstream

- Install verified, pinned host executables using `tools/install-tools.sh --prefix <PREFIX>`, place `<PREFIX>/bin` on `PATH`, verify both tools' version/protocol, and consult `docs/SHARED_TOOLING.md` for exact commands/schema.
- Cargo native C archive construction is performed by `ios-rust-build` via the package's minimal Rust `build.rs`; the ordinary executor may edit the capability-specific `build-spec.json` and its small bridge only when the capability actually needs native archive support. Do not recreate SDK discovery, Clang/ar loops, or the shared engine.
- Validation defaults to **declared checks** in `tools/validation/specs/validation-v1.json`, where applicable. The installed validator currently has only four configured production pilots (HomeKit identify, ActivityKit status, AlarmKit status, Photogrammetry status); **do not assume a new capability is registered or that all legacy checks are migrated**. Add a fully specified new capability declaration when supported, rather than rewriting the Rust engine.
- For unusual API-specific assertions use an optional, bounded Python 3 adapter through the versioned stdin/stdout JSON protocol. Retain necessary native fixtures and true compiler/link checks for ABI layout, ownership, Swift calling convention, weak symbols and availability. A required missing/failed/invalid adapter is never a pass.
- Retain applicable legacy package-local `check.sh`, compiler-oracle tests, import audits and CI gates until the new configuration demonstrably enforces equivalent or stronger behavior. Conversion of a check is a migration requiring negative-case parity, not just a path replacement.
- Distinguish `PASS`, `FAIL`, `SKIPPED(reason)` and `ERROR-BLOCKED`; no absent SDK/device or skipped gate may be claimed passed. Tooling does not substitute real permission, entitlement, interaction or device tests.
- Ordinary executors must not fetch, read or modify historical shared-engine source. For an apparent engine defect: check the documented CLI/schema and prerequisites, reproduce minimally, sanitize evidence, commit `BUG_REPORT_<DESCRIPTIVE_NAME>.md`, stop the affected work, and hand it to a separate maintenance session. Do not weaken gates or build an ad-hoc replacement validator.

## Workstream handoff requirements

Each assigned plan must name: exact starting files and symbols; verified current implementation; objective and remaining acceptance criteria; owned write surface and read-only dependencies; explicit non-goals; invariants around errors, ownership, async completion, cancellation, idempotency, security and persistence **where relevant**; host/Apple version limits; declarative validation entries and any custom adapters; real device/simulator evidence requirements; deterministic regression tests; exact commands; and an auditable final diff checklist. Preserve historical evidence/decisions as linked read-only references instead of flooding the routine executor context.

Before integration: verify current `main`, use isolated worktrees for parallel-safe tasks, require each executor to report changed paths, SHA, tests, skips, deviations and open assumptions, then integrate in dependency order. Re-run affected validation and independent ABI/consumer checks. Resolve plan contradictions centrally. Audit the final diff against the entire assigned contract. Remove temporary planning handoff files after implementation when that is the established workflow; do not delete durable history without evidence-backed retirement. **Do not initiate additional API work beyond specifically authorized workstreams.**

## Final review checklist

- Preserved Rust-native fast path, public Apple APIs, static backend choice, optional C/Python bindings, platform invariants, zero shipping Swift source and documented escape hatches.
- No inferred full-capability support from partial getter checks; no unsupported Swift async or private symbols.
- No lost negative assertion, deployment floor, framework import, ownership/cancellation guarantee, feature isolation or runtime evidence gate.
- No shared-engine source, dependency or bespoke replacement accidentally imported into normal checkout.
- Every unfinished workstream has a stable owner and accurate dependency; historical no-go/closed evidence remains recoverable.
- Every test command and evidence statement reflects actual current tooling and installed host requirements.

## Retired infrastructure plans (historical only)

`PLAN_REUSE_BUILD.md`, `PLAN_REUSE_VALIDATION.md`, and `PLAN_TOOLING_FINALIZATION.md` were deleted from current `main` in `f3b14892a560da05a086caf9e6d30a0c9beda25e` because their shared tooling implementation is finished. Do not recreate these plans or assign R1/R2 implementation work. Their former Git history is archival. Any unrelated API-specific validation registration is a capability/validation integration task, not an engine rewrite.