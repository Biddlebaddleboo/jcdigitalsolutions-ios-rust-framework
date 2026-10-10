# PLAN_REUSE_COMPACTION.md — P0: evidence-backed plan consolidation and repository rules

## Immediate planning action

## Completed tooling and current rewrite boundary

R1/R2 shared native-build and validation infrastructure, pinned installation, and source isolation are completed; the former `PLAN_REUSE_BUILD.md`, `PLAN_REUSE_VALIDATION.md`, and `PLAN_TOOLING_FINALIZATION.md` were retired in commit `f3b14892a560da05a086caf9e6d30a0c9beda25e`. Do not reopen them as outstanding work. Use `docs/SHARED_TOOLING.md` for tool commands and interface details. P0 remains strictly an evidence-backed planning ownership/retirement task; it must not implement API code or tooling engines. Replace obsolete test-execution references in surviving plans with installed-tool guidance only when equivalent coverage is expressible and proved; otherwise retain unique focused gates. At the post-deletion audit there were 326 `PLAN*.md` files; historical totals are snapshots, not current counts.


This is a planning-edit step, **not** a delayed coding executor task. The master PLAN.md is already compacted. Individually retire redundant legacy plans only after checking all acceptance conditions and transporting every unfinished obligation. Do not confuse a working status getter with an entire completed Apple capability.

## Scope

Audit `PLAN.md`, `PLAN_FOUNDATION.md`, `PLAN_SWIFT_ABI.md`, `PLAN_CAPABILITIES.md`, `PLAN_IOS_NATIVE.md`, `PLAN_VALIDATION.md`, `PLAN_BINDINGS.md`, `PLAN_REPLACEMENTS.md`, and referenced slice plans; inspect `AGENTS.md`, `docs/capabilities/capability-status.json`, corresponding code, tests and documentation. Proposed lasting ledger/index: `docs/planning/completion-ledger.json`, `docs/planning/active-index.md`. Preserve historical evidence in durable docs or git history, not the active execution context.

## Verified caveat

An earlier audit inspected 397 `PLAN*.md` files (historical snapshot, not current inventory). The manifest records 98/114 capability rows as **partial** and 16 as X. Old plan titles, existing crates, and compiler link checks alone cannot establish completion of the original requirements.

## Identifier and presentation policy

- Preserve every already-issued workstream identifier across consolidation, retirement and follow-up. Do not renumber existing P/R/A/B/C/D/F/G workstreams, create a W-series substitute, or reuse gaps. Allocate future IDs monotonically within the appropriate original series after consulting both live files and historical Git/ledger records.
- Keep capability-manifest IDs unchanged; they are capability identity, not workstream order.
- A merged plan must provide a self-contained executable contract for every retained workstream ID, without requiring executors to compare old and consolidated versions. Merge by coherent ownership, never by erasing distinct interfaces, invariants, non-goals, evidence boundaries or tests.
- Keep a durable historical mapping of original ID, former path, surviving authoritative location, reviewed commit, completion status and remaining owner. This mapping is audit data and should not be mandatory Codex reading for routine execution.
- Rewrite only references to retired **paths** where necessary; preserve ID citations as historical and dependency identities. Validate referential integrity and uniqueness, including historical IDs, before proposing any deletion.

## Classification

Record each existing plan once, with original path, objective and acceptance criteria, evidence (exact source symbols, tests, CI, verified commit), residual requirements, new owner, reviewed SHA and one status:
- `complete`: all original bounded criteria implemented and verified.
- `partial`: some criteria still open; keep exact residual tests and owner.
- `blocked`: cite public API/SDK/toolchain blocker and reevaluation trigger.
- `superseded`: an active plan explicitly preserves all open criteria.
- `not_started`: entire objective remains open.
- `research_closed`: bounded feasibility experiment ended; don't imply production implementation.

Unknown evidence -> partial. A proof may be complete while a related implementation remains open.

## Required retirement procedure

1. Audit top-level plans first, follow references only where needed, remove completed historical status prose from active instructions.
2. Group residual requirements by independent interfaces/owners, retaining specific first files/symbols, scope, invariants, tests, non-goals and handoff.
3. Make a deletion manifest of **individually verified** fully complete/superseded paths; never bulk-delete `PLAN_*.md`.
4. Update all docs, tests, CI and interplan links to stable docs or new plans; perform referential-integrity checks. Keep capability manifest canonical, without duplicating status history.
5. Maintain an active index; target `PLAN.md` under 250 lines. A Codex executor should need only `PLAN.md` plus assigned plan and source entrypoints.
6. Prepare `AGENTS.md` reuse amendment: search exact existing symbols/helpers before inventing, preserve typed/static crate boundaries and negative checks, avoid reading closed plans by default, ensure completed evidence is retained. **An edit to AGENTS.md is separately authorized and is excluded from a plan-only commit.**
7. Coordinate schema/CI verification with R2; test no orphaned requirements, missing links or stale active references.

## Validation/handoff

Dry-run audit and deletion report before changes; every old plan classified with evidence-backed residuals; no dropped safety/availability/ownership/async behavior; commands for docs-check, zero-swift-source and offline link-integrity/ledger tests; explicit retained/replaced/deleted filename inventory, evidence limits, validation and SHA. Reconcile advancing main before changing plans or rules.
