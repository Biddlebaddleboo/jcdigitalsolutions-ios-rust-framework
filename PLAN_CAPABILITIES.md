# PLAN_CAPABILITIES.md — Active portable capability coverage

## Canonical source, baseline and work selection

`docs/capabilities/capability-status.json` is authoritative for capability identities, ownership, platform classifications, supported/partial/unsupported status, availability, permissions, lifecycle and scope. The inspected repository has a previously recorded 98 partial B rows and 16 X rows, **not 98 completed framework capabilities**; re-evaluate the current manifest at latest `main` rather than carrying the number forward as a live count. The original D-family matrix and accumulated history remain recoverable at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_CAPABILITIES.md`. Never recycle D or B identifiers; retain closed no-go outcomes in `PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md`.

For a selected user-authorized capability, start with its manifest row, the targeted `PLAN_CAPABILITIES_*.md`, `PLAN_IOS_*.md` if present, and exact portable/backend symbols named there. Do not read all capability plans. Confirm which acceptance criteria are still open in current code, tests and docs before generating changes.

## Required architecture

Expose high-level platform-neutral semantics in `crates/framework-*/`, with stable errors, ownership, OS capability/availability reporting and `no_std` where required. Do not expose native ObjC/Swift/Java/Windows objects in portable types. Use target-static backend selection; ordinary Rust callers must use Rust directly rather than round-tripping through the exported C ABI. Keep Apple public API calls bounded to minimum native interfaces, typed `objc2` where available, or verified compiler-derived Swift ABI bridges; no guessed mangled symbols, private APIs, arbitrary Swift task invoker or shipping Swift source. Keep each capability's required framework dependencies isolated.

A snapshot getter is not an end-to-end operation: availability, consent, entitlements, host configuration, lifecycle and hardware may prevent real functionality. Distinguish read-only reports from mutations, prompts, background execution or cross-app access. Never invent a fallback that contradicts a prior no-go, policy limitation or Apple API entitlement.

D47 defines an owned, one-shot CloudKit account-status snapshot in [`framework-cloud`](crates/framework-cloud/), with no database, data-access or account-change behavior; see [PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md](PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md)

## Async and safety contract

Async-capable plans must specify start failure vs accepted operation, caller/actor requirements, exactly-once completion and callback ownership, cancellation **request** vs guarantee, drop/detach, multiple completion/callback safety, error propagation, user revocation, stale handles and shutdown. For durable/persistent operations specify authoritative state, duplicate-prevention, retry/timeout class, reconciliation and crash recovery; omit these only when not relevant. Preserve privacy, permissions, transport and application sandbox limits.

## Validation using the shared tools

Use `docs/SHARED_TOOLING.md` and the installed `ios-rust-build` / `ios-rust-validate`, not shared Rust engine source. Begin with `ios-rust-validate --list`; only four production pilots are currently configured. A new capability requires a reviewed `tools/validation/specs/validation-v1.json` declaration, optional Python assertions, and/or preserved native ABI/device-specific fixtures. Do not drop current `check.sh`/compiler-oracle or CI behavior without demonstrated parity, including negative checks and reverse dependency selection. Report static compile/link/import separately from iOS simulator and physical-device execution. Do not claim Xcode 27.x verification from earlier 26.6/26.5 evidence.

## Integration handoff

The selected capability owner owns its types/backend and tests, binding F owners consume frozen C ABI contracts, G owns cross-cutting CI, and R3 owns only separately justified verified shared ABI improvements. Respect overlapping write hotspots and integrate prerequisite changes first. Report changed files/symbols, exact test commands and status, newly declared gates, scoped public functionality vs unimplemented behavior, uncovered platform/device scenarios and the final commit SHA. Update the canonical manifest **only** with evidence-backed status changes.

## Completed infrastructure boundary

R1/R2 build and validation tooling is installed and complete; `PLAN_REUSE_BUILD.md`, `PLAN_REUSE_VALIDATION.md` and `PLAN_TOOLING_FINALIZATION.md` have been retired. The capability owner may register targeted checks in `tools/validation/specs/validation-v1.json`, but must not resurrect engine implementation work. Use `docs/SHARED_TOOLING.md` as the operational source of truth.