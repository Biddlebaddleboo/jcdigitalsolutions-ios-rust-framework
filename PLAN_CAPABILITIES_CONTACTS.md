# PLAN_CAPABILITIES_CONTACTS.md — Workstream D24: Portable Contacts Authorization

## Objective

Add a portable contract for querying Contacts authorization and explicitly requesting access. This slice does not expose contacts or claim that contact data can be fetched.

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 portable API conventions are integrated

## Write scope

- `crates/framework-contacts/**`
- `docs/capabilities/contacts.md`

Do not edit the root workspace or lockfile, capability manifest, aggregate plans or indexes, CI, iOS backend crates, Swift ABI, C bindings, or unrelated capability families.

## Required contract

- Provide fixed, portable `ContactsAuthorization` values for unknown, not determined, restricted, denied, authorized, and limited status
- Keep `Limited` semantically distinct from full `Authorized`; allow callers to test whether any access is allowed without erasing that distinction
- Provide `ContactsError`, a static `ContactsBackend`, and a thin caller-owned `Contacts<B>` facade for availability, non-prompting status query, and explicit authorization request
- Use `#![no_std]`, no allocator, no platform types, no global registry, no dynamic dispatch, and no required executor
- Specify lazy request start, future drop, callback lifetime, and exactly-once result semantics
- Keep enumeration, fetch, contact values/identifiers, writes, ContactsUI, and picker behavior out of scope

## Validation and handoff

- Add deterministic portable tests for authorization semantics, lazy explicit request behavior, and native-code preservation
- Run `cargo fmt --all -- --check`, `cargo test -p framework-contacts`, `cargo check -p framework-contacts --no-default-features`, and `git diff --check`
- Audit the portable public API for platform types, `std`, allocation, hidden initialization, and dynamic dispatch
- Report changed files, exact checks, runtime limits, deviations, and unresolved assumptions
