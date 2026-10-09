# PLAN_CAPABILITIES_ACCESSORY.md — Workstream D39: ExternalAccessory Presence

## Objective

Define a portable scalar snapshot for whether the current platform list of connected-and-available external accessories contains an entry.

## Required contract

- Keep the contract `no_std`, allocation-free, and free of native handles or accessory identity
- Treat each result as a transient point-in-time list snapshot
- State that an empty list does not prove physical absence, general platform support, protocol eligibility, or communication readiness
- Exclude picker presentation, notifications, protocol inspection, sessions, streams, and data transfer
- Make no MFi, entitlement, privacy, or provisioning claim

## Validation

Run the portable no-default check, strict Clippy, and rustdoc through `platform/ios/ios-accessory/scripts/check.sh`. Do not claim live hardware behavior.

## Write scope

- `crates/framework-accessory/**`
- `docs/capabilities/accessory.md`

The orchestrator owns workspace integration, the canonical matrix, CI, and aggregate-plan updates.
