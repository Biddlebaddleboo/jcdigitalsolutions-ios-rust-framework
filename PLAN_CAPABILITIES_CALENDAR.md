# PLAN_CAPABILITIES_CALENDAR.md — Workstream D25: Calendar Authorization Contract

## Objective

Add a portable no_std contract for Calendar event-authorization status and an explicit full-access request, without native EventKit types or an iOS backend dependency.

## Dependencies

- `PLAN_FOUNDATION.md`
- The integrated framework-core error and platform-code contract

## Write scope

- `crates/framework-calendar/**`
- `docs/capabilities/calendar.md`
- `PLAN_CAPABILITIES_CALENDAR.md`

Do not edit the canonical capability manifest, root workspace manifests or lockfile, aggregate plans, CI, `tools/xtask`, indexes, iOS backends, C/C++ bindings, or Swift ABI files.

## Required contract

- Define framework-owned statuses for unknown, not-determined, restricted, denied, write-only, and full access.
- Keep `WriteOnly` distinct from `FullAccess`; this slice reports write-only status but has no write-only request operation.
- Expose static `CalendarAuthorizationBackend` and a thin `Calendar<B>` client for non-prompting status and explicit full-access request.
- Use no_std plus core only in the portable crate; impose no executor, boxed future, global registry, or `Send` requirement.
- Specify that prompt-capable work starts on first future poll, request results come from post-completion status, and dropping a pending future detaches result interest without promising prompt dismissal.
- Preserve portable error category and optional native code through `CalendarError`.
- Add deterministic fake-backend tests for status forwarding, write-only/full distinction, and explicit lazy request behavior.

## Exclusions

No event/reminder fetch or enumeration, event creation or edits, write-only request, reminder permission, EventKitUI or calendar UI, entitlements, permission prompt implementation, native types in the portable crate, or C/C++/Swift API.

## Validation

~~~sh
cargo +1.94.1 fmt --all -- --check
cargo +1.94.1 test --locked --offline -p framework-calendar
cargo +1.94.1 check --locked --offline -p framework-calendar --no-default-features
git diff --check
~~~

## Status

Reconciled against current base `95f201c0ecccf624ea78f726e83d5e340bea4df1`; the existing portable API already met the required static/no_std contract. Clarified that `Unknown` differs from `NotDetermined`, made unpolled-drop semantics explicit, and strengthened deterministic tests to observe backend method invocation separately from future polling. Added error kind/native-code preservation coverage.

Current worktree evidence with Rust `1.94.1`: workspace formatting passed; the package test suite passed all 5 tests; the package no-default-features check passed; `git diff --check` passed. The locked offline dependency tree contains only `framework-core` below `framework-calendar`. Production code uses `core` plus framework-owned `framework-core` types; `std` is test-only under `cfg(test)`, and the crate has no allocator, unsafe code, dynamic dispatch, or native EventKit dependency. No iOS prompt, EventKit runtime, or event-data behavior was exercised or asserted.
