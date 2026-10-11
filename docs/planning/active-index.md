# Active Plan Index

Historical review basis: `7c702b5b659941d47f3d0ca1af3e29cfb4c2df33`

## Current execution

The current execution follows the user’s standing objective to implement feasible public Apple APIs autonomously, using latest `main`, [PLAN.md](../../PLAN.md), and the canonical capability registry. The P0/R1/R2/R3 entries below preserve the former infrastructure sequence; they do not limit API implementation.

1. **P0** — [PLAN_REUSE_COMPACTION.md](../../PLAN_REUSE_COMPACTION.md): preserve plan IDs, acceptance criteria, owners, and evidence refs
2. **R1** — [PLAN_REUSE_BUILD.md](../../PLAN_REUSE_BUILD.md): factor native bridge build support
3. **R2** — [PLAN_REUSE_VALIDATION.md](../../PLAN_REUSE_VALIDATION.md): factor capability validation while retaining all focused guards
4. **R3** — [PLAN_REUSE_INTEROP.md](../../PLAN_REUSE_INTEROP.md): reuse only compiler-verified Swift/native interop primitives

The former R1–R3 stop boundary is superseded; continue capability implementation after infrastructure work.

## Execution boundary

API and capability implementation remains active beyond shared build, validation, and verified interop infrastructure. Keep shared engine/release maintenance separate unless separately authorized.

## Other plan records

The [completion ledger](completion-ledger.json) is a conservative historical plan-path inventory at its recorded review SHA, not current capability status or proof of completion. Use latest `main`, `docs/capabilities/capability-status.json`, and relevant targeted plans to determine current implementation state.
