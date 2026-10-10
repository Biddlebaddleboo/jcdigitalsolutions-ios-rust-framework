# PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md — Workstream D30: iCloud Drive Identity-Presence Contract

## Objective

Add a portable `no_std` contract that reports only whether a backend observed a non-`nil` iCloud
Drive Documents identity token. Do not expose or classify the opaque token.

## Write scope

- `crates/framework-cloud/**` for this bounded contract only
- `docs/capabilities/icloud-drive-identity.md`

Do not edit root workspace configuration or lockfile, capability status JSON/counts, aggregate
plans, shared documentation indexes, CI, `tools/xtask`, CloudKit work, or other capability slices.

## Required contract

- Use an allocator-free `#![no_std]` portable value with no third-party dependency or Apple type.
- Provide only token-present and token-absent outcomes plus a synchronous static backend trait.
- Map one nullable token observation to presence only; do not infer why it was absent.
- State that presence does not prove ubiquity-container access, sync, or CloudKit account status.
- Do not return, store, compare, serialize, stringify, format, or log the token.
- Do not add CloudKit calls, account names or identifiers, file operations, notifications, or sync.

## Documentation and evidence

- Cite Apple's `FileManager.ubiquityIdentityToken` documentation and Technical Q&A QA1935.
- Record absent-token ambiguity, token-present limits, host iCloud capability/service, signing,
  per-app iCloud Drive setting, no `Info.plist` key, and no entitlement inspection/inference.
- Record that snapshots may become stale immediately and do not observe identity changes.

## Validation

- Add deterministic tests for the two token-presence outcomes and a portable fake backend.
- Run `cargo test -p framework-cloud`,
  `cargo check -p framework-cloud --no-default-features`, `cargo fmt --all -- --check`, and
  `git diff --check`.
- Inspect the public API for Apple types, `std`, allocator, native-token escape, and unrelated
  dependencies.
- Report changed files and limits. Do not edit or claim an aggregate capability-matrix update.

## Completion status

Implemented the D30 presence slice in `crates/framework-cloud` and
`docs/capabilities/icloud-drive-identity.md`. `UbiquityIdentityBackend: Sized` enforces a
synchronous concrete backend call path; deterministic tests cover both nullable-observation
outcomes and a generic fake backend. The separate CloudKit account-status API and its existing
`framework-core` dependency remain unchanged.

Rust 1.94.1 evidence:

- `cargo +1.94.1 test -p framework-cloud` — passed, 2 tests.
- `cargo +1.94.1 check -p framework-cloud --no-default-features` — passed.
- `cargo +1.94.1 fmt --all -- --check` — passed.
- `git diff --check` — passed.
- API audit confirmed the D30 value uses only `bool`/enum state, with no Apple type, `std`,
  allocator, native-token return or storage path, or added dependency.

`Cargo.lock` already records the package in the integration baseline and remained unchanged. These
checks cover the portable contract only; they do not prove iOS runtime, iCloud service, signing,
container access, synchronization, or CloudKit account behavior. A snapshot may become stale
immediately and does not observe identity changes.
