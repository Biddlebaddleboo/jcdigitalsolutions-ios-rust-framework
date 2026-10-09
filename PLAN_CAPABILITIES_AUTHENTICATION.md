# PLAN_CAPABILITIES_AUTHENTICATION.md — Workstream D12: Local Authentication

## Status

D12's portable `framework-auth` contract and guide are integrated. B16's separate `ios-auth`
backend is integrated with target compile/lint and link-import evidence; no tests or live prompt were
run.

## Objective

Add a small portable contract for one-shot, system-mediated local user-presence authentication.
Keep this distinct from credential storage, remote identity, passkeys, biometric-data access, and
general privacy authorization status.

## Dependencies

- Foundation A and `framework-core` are integrated
- D2 secure storage is not a dependency; authentication does not store or retrieve credentials
- B16 implements the iOS backend with the public Objective-C `LAContext` API through generated bindings; see `PLAN_IOS_AUTHENTICATION.md`

## Write scope

- `PLAN_CAPABILITIES_AUTHENTICATION.md`
- `crates/framework-auth/Cargo.toml`
- `crates/framework-auth/src/lib.rs`
- `docs/capabilities/authentication.md`

Do not edit `PLAN_CAPABILITIES.md`, root workspace configuration, `Cargo.lock`, capability
manifest, shared capability summary, iOS backend, Swift ABI, C bindings, or other capability
contracts. The orchestrator owns shared plan links and manifest/lockfile reconciliation. Do not
add platform types or Swift source.

## Contract requirements

- Add an independently usable `#![no_std]` crate named `framework-auth` with generic static
  backend selection, caller-owned state, no global registry, and no hidden initialization
- Define `AuthenticationPolicy::BiometricsOnly` as requiring biometrics without local-device-
  credential fallback, and `AuthenticationPolicy::DeviceOwner` as allowing the platform's
  biometrics or local-device-credential policy; do not expose biometric modality or data
- Define a borrowed `AuthenticationRequest` that carries one policy and non-empty, non-whitespace
  caller-provided reason text without normalization
- Provide policy-specific availability queries that do not prompt
- Provide one asynchronous operation whose native work starts no earlier than the returned future's
  first poll; no executor or `Send` requirement is imposed
- Define `Ok(())` only as the selected platform policy reporting success at that operation's
  completion; do not claim identity proof, a reusable credential, or protection of later work
- Preserve backend `ErrorKind` and optional platform error code; map invalid reason text to
  `InvalidInput`
- If the returned future is dropped while pending, require the backend to request native
  cancellation where supported, suppress the Rust result, and keep callback state safe until
  released exactly once; do not promise that already-presented system UI is dismissed
- Document reason-text ownership, prompt timing, blocking/async behavior, errors, cancellation,
  and the confidentiality boundary
- Do not add passkeys, Sign in with Apple, Keychain policy, privacy-authorization queries, or
  biometric values

## Validation and handoff

- Run `cargo check -p framework-auth --no-default-features`
- Run strict Clippy with `cargo clippy -p framework-auth --no-default-features -- -D warnings`
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`,
  `cargo xtask zero-swift-source`, and `git diff --check`
- Do not add or run tests in this slice
- Report public symbols, policy semantics, prompt/cancellation behavior, exact commands/results,
  changed files, deviations, and unresolved assumptions; do not edit shared capability totals
