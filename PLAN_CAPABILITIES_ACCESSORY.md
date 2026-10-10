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

## D39 reconciliation evidence (2026-10-10)

The existing `framework-accessory` contract and `docs/capabilities/accessory.md` meet this
workstream's portable scope. The crate is `no_std`, has no dependencies or allocation, and returns
only the scalar `AccessoryPresenceSnapshot`; it exposes no native handle or accessory identity.
Each value describes the current connected-and-available list at query time. An empty list does not
prove physical absence, general platform support, protocol eligibility, or communication readiness.
The contract and guide exclude picker presentation, notifications, protocol inspection, sessions,
streams, data transfer, and any MFi, entitlement, privacy, or provisioning claim. No product/API
change was needed.

The read-only row `043-sensors-connectivity-externalaccessory` remains marked as a portable
contract with partial iOS implementation. It describes only the current list snapshot; this D39
reconciliation does not change the canonical matrix or B44/iOS implementation.

`sh platform/ios/ios-accessory/scripts/check.sh` passed with exit code 0 on Xcode 26.6
(17F113), iOS SDK 26.5, and Rust/Cargo 1.94.1. This gate covered formatting, the portable
`--no-default-features` check, strict Clippy and rustdoc, device and Simulator `cargo check`, strict
Clippy for both targets, device-target rustdoc, the link-import check, `cargo xtask docs-check`,
`cargo xtask zero-swift-source`, and `git diff --check`. The pinned validator list has no
ExternalAccessory pilot; the prescribed package gate is the check above. The gate runs no tests and
does not prove live hardware behavior; Xcode 27 qualification and device runtime remain unverified.
