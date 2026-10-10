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

## D37 reconciliation evidence — 2026-10-10

Reconciled the portable contract at base `5fbbe26cfedb92272a3f3a80ef71d76ff7e7b518`. No crate or capability-guide edit was required:

- `framework-device-integrity` is `#![no_std]`, forbids unsafe code, and has no dependencies. `AvailabilitySnapshot` owns two `bool` fields and derives `Copy`; it uses no allocation, global state, or Apple type.
- The public surface contains only the snapshot constructor and support-value accessors. It has no prompt, network call, key/token operation, attestation/assertion operation, or persistent security decision.
- Crate docs and `docs/capabilities/device-integrity.md` describe support as transient and state that positive values do not establish integrity, identity, enrollment, entitlement configuration, server trust, or success of later key/token generation, attestation, or assertions. The iOS guide records that no prompt or network request occurs.
- Canonical row `022-security-auth-app-attest-devicecheck` marks the portable contract implemented while the overall capability remains partial; B42 reports support values and makes no key, token, attestation, service-contact, or trust claim.

Pinned shared tools installed and version-checked: `ios-rust-build` and `ios-rust-validate` 0.1.0, source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. The validator lists four unrelated pilot profiles; no D37 profile applies.

Checks on 2026-10-10:

```text
cargo +1.94.1 fmt --all -- --check — PASS
cargo +1.94.1 check --locked --offline -p framework-device-integrity --no-default-features — PASS
cargo +1.94.1 clippy --locked --offline -p framework-device-integrity --all-targets -- -D warnings — PASS
git diff --check — PASS
```

No native support query, key/token generation, attestation/assertion, prompt, network call, entitlement setup, server verification, or runtime security result was tested or claimed. No test was added or run. Row 022 remains partial.
