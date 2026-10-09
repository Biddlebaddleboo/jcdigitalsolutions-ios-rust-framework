# PLAN_CAPABILITIES.md — active portable capabilities and iOS coverage

## Canonical source and historical reference
`docs/capabilities/capability-status.json` is authoritative for all 114 capability rows, crate ownership, min iOS versions, platform classes, permissions, lifecycle and scope. Current inspected baseline: 98 partially supported B rows, 16 X. **No family is wholly complete.** Prior detailed D1–D100+ matrix / accumulated status history is in Git at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_CAPABILITIES.md`; ordinary executors must not load it. Closed X-row feasibility decisions: `PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md`. Active residuals are represented by the manifest and surviving individual capability plans until individually retired.

## Implementation scope
Start with named row in the manifest, `crates/framework-*/src/lib.rs`, selected `platform/ios/ios-*/src/lib.rs`, `docs/capabilities/*`, `PLAN_IOS_NATIVE.md` and only assigned targeted plan. Portable semantic types, error/availability, no_std where required, no platform-native types in portable contracts. Statically selected native backends; public Apple interfaces through minimal C/Darwin, `objc2` or proven Swift ABI. Rust caller never routes through C ABI. Preserve optional native escape.

## Required behavior and invariants
Do not elevate a support Boolean to an end-to-end capability. Document host configuration, entitlements, permissions, unsupported devices, app lifecycle, callback delivery and actual operation reachability. Async contracts must distinguish operation start, exactly-once completion, cancellation request versus cancellation guarantee, drop/detach semantics, caller-thread/actor restrictions and error detail. Do not hide kernel/platform background restrictions. Platform-exclusive semantics stay exclusive.

## Active ownership / next work
- Native integration: `PLAN_IOS_NATIVE.md`; Swift-only ABI gaps: `PLAN_SWIFT_ABI.md` and `PLAN_REUSE_INTEROP.md`.
- Reusable build/test machinery: `PLAN_REUSE_BUILD.md` and `PLAN_REUSE_VALIDATION.md`.
- Existing capability-specific plans remain authoritative **only for unclosed acceptance criteria**. Do not repeat isolated scalar-query additions solely to increase coverage counts. Prefer complete useful workflows when backed by a public API.
- Preserve every `X` row and detailed `status_reason`; change its class only after verified implementation and targeted validation.

## Tests / handoff
Portable unit/property tests, static boundary checks, no_std, locked cross-target check/lint, import/entitlement evidence, simulator or device runs when claims require it, permissions/fake-clock/state-machine tests where feasible, parity/performance only with actual evidence. Update manifest, rustdoc, feature graph and guides in the same implementation commit. Record deviations and SHA.
