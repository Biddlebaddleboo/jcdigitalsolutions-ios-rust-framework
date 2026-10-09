# PLAN_CAPABILITIES_DEVICE_INTEGRITY.md — Workstream D37: DeviceCheck and App Attest Support Values

## Objective

Add a portable `no_std` value for the platform's DeviceCheck and App Attest API support booleans. This is a transient capability hint, not device integrity, identity, enrollment, or trust.

## Required contract

- Use framework-owned booleans in `AvailabilitySnapshot`; expose no Apple types
- Keep the value copied, fixed-size, and free of allocation or global state
- State that positive support does not guarantee token/key generation, attestation, assertion, entitlement setup, or server verification
- Do not expose operations, prompts, network calls, or persistent security decisions

## Validation

Run the portable no-default check and strict Clippy gate. Do not claim a runtime security result.

## Write scope

- `crates/framework-device-integrity/**`
- `docs/capabilities/device-integrity.md`

The orchestrator owns workspace integration, the canonical matrix, CI, and aggregate-plan updates.
