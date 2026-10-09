# PLAN.md — V1 execution reset: reusable machinery and active-only plans

## Authority and baseline

Repository: `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework`; branch: `main`; inspected commit `ac944e80f195c810d145d51064bb1f9c3c875b49`. Reverify remote main before implementation. This is the authoritative active handoff. Existing `AGENTS.md` invariants remain in force: Rust-native fast path, zero shipping Swift sources, public Apple APIs only, static backend selection, replaceable dependencies, no required runtime registry or custom renderer, C ABI for foreign callers, optional Python, and performance evidence before replacement. Planned Xcode 27.x qualification is not established by Xcode 26.6 / SDK 26.5 static proofs.

## Objective

Reduce context usage and duplicated implementation/tooling without weakening modularity, feature coverage, safety or validation. Compact active planning instructions now, retire only verified completed legacy plans, introduce reusable build and validation infrastructure, then consolidate compiler-proven interop patterns. Do not implement application behavior while performing planning cleanup.

## Verified repository facts

- Capability manifest: `docs/capabilities/capability-status.json`; the guide reports 98 partially supported B rows of 114, with 16 X. No whole capability is therefore proved complete.
- `interop/swift-abi-core/src/lib.rs` exposes feature-gated `SwiftRetained` from `retained.rs` under `#![no_std]`. Compiler proofs cover ownership, scalar Swift calls, String, Optional, and async lowering. A documented general Swift async task-entry/resume contract is still unavailable; do not claim a production async adapter.
- `platform/ios/ios-activitykit-status/build.rs`, `ios-alarmkit-status/build.rs`, `ios-photogrammetry-status/build.rs` repeat Clang/SDK/archive/link plumbing but vary in targets, deployment floors and frameworks.
- Many `platform/ios/*/check.sh` repeat Cargo/CI commands and also contain important unique negative assertions. `tools/xtask/src/main.rs` already offers audits, inventory, docs, zero-Swift, and archive commands; parity presently reports unavailable.
- `platform/ios/ios-files/src/lib.rs::IosFiles` implements scoped POSIX/native sandbox operations with distinct semantic constraints.
- The inspected repository contains 397 `PLAN*.md` files, including a very large accessibility history. File presence or a partial capability row cannot prove a plan's acceptance criteria completed.

## Active plans and sequencing

0. **P0 immediate planning editing** — [PLAN_REUSE_COMPACTION.md](PLAN_REUSE_COMPACTION.md). Keep an active-only index; evidence-classify legacy plans before deleting them. This stage precedes code work and is not delegated to a coding executor. Historical plans are not routine executor reading material.
1. **R1 shared native build support** — [PLAN_REUSE_BUILD.md](PLAN_REUSE_BUILD.md). Factor build machinery, never guessed ABI signatures.
2. **R2 shared validation** — [PLAN_REUSE_VALIDATION.md](PLAN_REUSE_VALIDATION.md). Factor standard checks while retaining feature-specific public-API, import, symbol, availability, and ownership guards.
3. **R3 reusable verified interop primitives** — [PLAN_REUSE_INTEROP.md](PLAN_REUSE_INTEROP.md). Only after R1 interface and R2 evidence gates are settled.
4. **Stop after infrastructure integration** — Validate and hand off R1–R3; do not begin unfinished API/capability work. All deferred contractual functionality remains preserved for a separate user-authorized execution.

## Current execution boundary — infrastructure only

**This execution is limited to R1, R2 and R3 shared infrastructure.** P0 reconciliation is allowed only where required for their safe integration; no general backlog cleanup or new capability implementation is authorized. The previously identified D/B/F/G/API work remains deferred, not cancelled.

- Codex **may spawn bounded, independently implementable infrastructure sub-workstreams** under R1/R2/R3, with existing ownership, dependency ordering, isolated worktrees where parallel-safe, explicit interface contracts and integration testing. Do not invent or renumber workstream IDs just for delegation.
- Codex **must not spawn API, capability, application-feature or backend-expansion workstreams**, nor use infrastructure migration as a reason to complete unrelated APIs. Existing pilot code may be adapted only as necessary for infrastructure refactoring, compiler/link correctness or regression validation, preserving its public contract and observable behavior. R3 is restricted to reusing verified existing interop primitives; compiler-only proofs do not authorize production Swift async, StoreKit, Translation or App Intents APIs.
- Complete the R1/R2/R3 scope and tests, integrate in the prescribed order, review the final diff, report exact validation, skips and blockers, perform the ordinary implementation handoff and commit without temporary planning files. **Then STOP.** Do not select another backlog item, create follow-up implementation workstreams or resume API development without a new explicit user request.
- If a required infrastructure task is blocked, record its blocker and remaining owner rather than expanding scope into API work. Maintain all outstanding capability contracts for a later separately authorized cycle.

## Workstream identity and consolidation contract

- **Existing IDs are permanent.** Keep all assigned P/R/A/B/C/D/F/G workstream IDs, filenames and interplan dependency references stable. Never renumber, recycle a retired ID, or create a replacement series (including W00–W03). The canonical capability IDs in `docs/capabilities/capability-status.json` are independently immutable.
- **New workstreams continue the appropriate existing series**, using an ID greater than every previously issued ID in that series, including retired and Git-only historical plans. Check the completion ledger and Git history before allocating. A gap does not imply an available ID.
- **Consolidation is transparent to execution.** A consolidated plan is an ordinary authoritative contract for the workstreams it owns. Keep each workstream's original ID and all necessary interfaces, invariants, write ownership, prerequisites, failure cases, non-goals, tests, evidence limits and unresolved work. Do not assign new IDs solely because text moved into another plan.
- **Separate execution and audit views.** Executors read this plan, their assigned active plan and relevant source/doc entrypoints; they do not need to distinguish a legacy plan from a consolidated one. Preserve immutable original-plan refs, status classifications and relocation mappings in the completion ledger / durable audit documentation, not as routine extra reading. Use historical details when resolving a disputed or missing contract.
- Retiring a plan file is not retiring its still-open obligations. Preserve each such obligation under an explicitly named existing workstream owner and validate identifiers and links before committing planning-only changes.

## Legacy scope migration and completion rules

Existing foundation, capabilities, native iOS, Swift ABI, bindings, replacements and validation plans are historical input to classification, not default reading. Consolidate by independently executable ownership and exact symbols, not by individual scalar property. Keep `docs/capabilities/capability-status.json` as canonical capability coverage. For every old plan assign exactly one evidence-backed status: `complete`, `partial`, `blocked`, `superseded`, `not_started`, or `research_closed`. Capture original criteria, code/tests/verification evidence, unresolved acceptance criteria and new owner. Unknown status defaults to partial, not complete. An implemented status getter does not complete a full framework capability. Never run `rm PLAN_*.md` indiscriminately.

Residual ownership: Swift runtime compiler proofs and genuinely open async/App Intents interfaces -> R3 and narrow follow-up; build scripts -> R1; validation/CI -> R2; filesystem -> `IosFiles` owner; accessibility -> `ios-accessibility` owner; C/Python bindings -> bindings owner; performance replacements -> evidence-backed replacement owner; other native capabilities -> manifest-assigned owner. Retire a file only once all remaining obligations are transferred, implemented or explicitly blocked and all links are updated.

## Shared interfaces, ownership and parallel safety

| Path / symbol family | Single writer | Consumers |
|---|---|---|
| Proposed `tools/native-build-support/**`, pilot `build.rs` migrations | R1 | R3, capability crates |
| `tools/xtask/**`, proposed validation harness, CI | R2 | R1, R3 |
| `interop/swift-abi-core/**`, `interop/swift-abi-generated/**` | R3 | capability crates |
| `PLAN*.md`, proposed completion ledger / planning index, `AGENTS.md` amendment proposal | P0 planning editor | all |
| Capability-status manifest | serialized capability integration owner | all read-only |
| `platform/ios/ios-files/**` | existing filesystem owner | others read-only |

Do not add mandatory runtime dependencies, dynamic registries, a centralized mega-crate, or an arbitrary Swift ABI invoker. Preserve static dispatch, feature isolation, deployment floors, public ABI surface and no_std constraints.

R1 and R2 may work concurrently only in isolated worktrees; integrate R1 then R2. Start R3 after shared interface review. Each executor reports changed paths, commit SHA, tests, skipped checks, differences, remaining risks and open assumptions. Resolve contradictions centrally. Recheck main before work, rerun relevant host/iOS cross-target, Clippy, docs, link/import, zero-Swift, feature-isolation and example validations. No missing SDK or device test may be labelled passed.

## Final diff checklist

- No production behavior change from build/test refactoring; preserve Apple API and OS version guards.
- No unsupported Swift async promise, private symbol, guessed ABI signature, duplicated shared helper or mandatory capability linkage.
- Every retired plan has acceptance-criteria and validation evidence; every residual requirement has an explicit active owner; docs/CI links stay valid.
- Review resulting file and linked-binary diffs; preserve smaller active Codex context.

## Execution handoff

Verify latest main, read this plan and assigned R1/R2/R3 infrastructure workstream only, implement within named ownership, integrate dependencies in order, report tests/SHA, independently validate the final diff, delete temporary PLAN*.md, and commit implementation without them. Stop after this handoff; do not initiate capability/API work. All changes to the planning set require approval before GitHub writes; `AGENTS.md` is outside a plan-only commit.
