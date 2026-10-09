# PLAN_CAPABILITIES_NEARBY_INTERACTION.md — Workstream D34: Nearby Interaction Capability Snapshot

## Objective

Add a no_std portable value and backend contract for one non-prompting Nearby Interaction device-capability field. Do not implement session, peer, token, permission, or ranging behavior.

## Dependencies

- D1 portable API conventions are available
- No third-party dependency is needed by the portable contract
- B39 supplies the iOS adapter in a separate workstream

## Write scope

- `crates/framework-nearby/**`
- `docs/capabilities/nearby-interaction.md`

Do not edit the root workspace configuration, root lockfile, capability status manifest, iOS adapter, shared docs indexes, CI, or `tools/xtask`. The integrator owns workspace and manifest reconciliation.

## Required contract

- Build as a portable `#![no_std]` crate with no third-party dependencies and no unsafe code
- Expose an owned scalar snapshot with only `supports_precise_distance_measurement: bool`
- Expose a synchronous, statically selected backend trait
- Define the value only as the backend-reported precise-distance capability; do not call it general Nearby Interaction or session-operation support
- Require the backend query to be non-prompting and exclude session creation/run, permission requests, discovery-token exchange, peer discovery, and ranging
- Preserve no Apple type in the portable API

## Documentation

Record that the snapshot does not report permission, peer compatibility, session readiness, runtime success, ranging accuracy, or background-operation support. State that `false` only describes this one feature field and that `true` does not guarantee an interaction can run.

## Validation and handoff

- Add deterministic tests for `true` and `false` snapshots and a fixed fake backend
- Run `cargo fmt --all -- --check`
- Run `cargo test --locked -p framework-nearby`
- Run `cargo check --locked -p framework-nearby --no-default-features`
- Run strict Clippy for `framework-nearby`
- Inspect the public API for Apple types, `std`, unsafe code, dynamic dispatch, initialization, and unrelated dependencies
- Report exact checks, dependency metadata, metadata caveats, and remaining work
