# PLAN_CAPABILITIES_SECURE_STORAGE.md — Workstream D2: Secure Storage Contract

## Status

D2 remains the portable `no_std` facade and contract; it does not select or implement a storage
backend. B2 is separately integrated as an iOS Keychain backend. The facade alone provides no
secrecy; see `docs/capabilities/secure-storage.md` and `docs/ios/secure-storage.md`. B2 adds items
in the app's default Keychain group, but unfiltered reads, updates, and deletes search all groups
available to the app; strict default-group isolation is not provided. This backend-specific
behavior does not change D2's portable identity or policy semantics.

## Objective

Add a portable, statically selected secure-storage facade for opaque secret bytes. This slice defines semantics only; it does not implement Keychain, cryptography, biometric access, key generation, or remote sync.

## Dependencies

- Foundation A and D1 app-data contracts are integrated
- Root workspace glob already matches `crates/*`

## Write scope

- `crates/framework-secure-storage/**`
- `docs/capabilities/secure-storage.md`

Do not edit root workspace configuration, the capability status manifest, iOS backends, Swift ABI, C bindings, or unrelated capability families. The orchestrator updates the shared manifest after integration.

## Required contract

- The crate must compile as portable `#![no_std]` with default and no-default features
- Use caller-owned backend state and static dispatch; no global lookup, hidden initialization, boxed trait objects, or executor
- Model opaque secret bytes only; never implement encryption or imply that memory returned to the caller stays protected after the call
- Validate borrowed service and item identifiers without normalization; reject empty text and NUL bytes
- Make read ownership and write copy cost explicit; never retain a caller borrow
- Define a small portable access policy that can state whether device unlock is required and whether a stored item must remain on one device
- Allow a caller to require a policy; a backend must reject an unsupported requirement before mutation and report the effective policy on success
- Preserve stable framework error categories and optional native status codes
- Define no Keychain-specific or other platform type in the portable API
- Add deterministic contract tests for validation, policy rejection without mutation, owned reads, and remove semantics

## Documentation

Record confidentiality boundaries, plaintext exposure after read, backend-selected protection limits, copy costs, access-policy semantics, and the absence of cryptographic key APIs in this slice. Distinguish this portable D2 contract from the separately integrated B2 Keychain backend, state that the facade alone does not provide secrecy, and link to the current backend guide.

## Validation and handoff

- Run format, crate tests, Clippy, docs, and default/no-default checks
- Inspect public API for platform/dependency types and dynamic dispatch
- Report exact checks, changed files, commit SHA, deviations, and unsupported Keychain/crypto/biometric behavior
