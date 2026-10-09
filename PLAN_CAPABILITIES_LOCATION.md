# PLAN_CAPABILITIES_LOCATION.md — Workstream D4: Portable Location Contract

## Status

D4's one-shot portable location contract is implemented. Numeric tests cover inclusive coordinate
bounds, both out-of-range directions for each component, NaN and both infinities for coordinates,
accuracy targets, and fix accuracy, plus negative finite accuracy values. Authorization mapping and
the fake backend are also covered. `cargo fmt --all -- --check`, `cargo test -p framework-location`
(7 tests), `cargo check -p framework-location --no-default-features`, and `git diff --check` pass.
The contract defines no timeout; callers impose deadlines outside the request and drop pending
futures under backend cancellation rules.

## Objective

Add a small portable Rust contract for one-shot current-location requests. Do not add an iOS backend in this workstream.

## Dependencies

- Foundation A is integrated
- D1 portable API conventions and `framework-core` are integrated
- iOS backend work waits for this contract and a separate named subplan

## Write scope

- `crates/framework-location/**`
- `docs/capabilities/location.md`
- `Cargo.lock` only for central workspace resolution

Do not edit root workspace configuration, D1-owned crates, the capability status manifest, iOS backend crates, Swift ABI, C bindings, or unrelated capability families. The orchestrator updates the shared manifest after integration.

## Required contract

- Build as a portable `#![no_std]` crate with no third-party dependencies
- Use framework-owned coordinate, fix, request, authorization, and error values; expose no Apple types
- Validate latitude and longitude ranges and reject NaN or infinite coordinates
- Make the coordinate reference, accuracy unit, timestamp epoch/unit, ownership, and returned-fix cost explicit
- Model one-shot current-location requests with a caller-selected accuracy target; do not claim that a backend must meet that target
- Expose static `LocationBackend` selection, availability, non-prompting authorization query, explicit authorization request, and a Rust `Future` for one fix; require no global registry or executor
- Define `Availability` as a non-prompting backend service/context signal separate from authorization; it does not imply that permission is granted or a request will succeed
- Define no portable timeout field or completion deadline; caller-imposed deadlines are external and follow the future-drop/cancellation contract, while backend timeout errors preserve their category and optional native code
- Preserve backend error category and optional native code
- State future-drop, native request cancellation, permission, accuracy, and exactly-once completion semantics
- Keep continuous updates, geofencing, visit monitoring, heading, speed, background location, and location history out of scope

## Documentation

Record coordinate validation, accuracy and timestamp semantics, permission scope, future ownership, native escape boundary, data freshness limits, and all unimplemented location features.

## Validation and handoff

- Add deterministic tests for all inclusive coordinate endpoints and both out-of-range directions per component; NaN and both infinities for coordinate components, accuracy targets, and fix accuracy; negative finite accuracy targets and fix accuracy; authorization mapping; and a fake backend
- Run `cargo fmt --all -- --check`, `cargo test -p framework-location`, `cargo check -p framework-location --no-default-features`, and `git diff --check`
- Inspect the public API for platform types, `std`, dynamic dispatch, hidden initialization, and unrelated dependencies
- Report changed files, commit SHA, exact checks, deviations, and unresolved assumptions
