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
cargo fmt --all -- --check
cargo test -p framework-calendar
cargo check -p framework-calendar --no-default-features
git diff --check
~~~

## Status

Complete. `framework-calendar` is `no_std`, forbids unsafe code, and has no Apple dependency. The fake-backend tests cover status forwarding, `WriteOnly` versus `FullAccess`, and lazy explicit request start. `cargo test -p framework-calendar` passed with 3 tests; `cargo check -p framework-calendar --no-default-features`, workspace format, and `git diff --check` passed in the package gate.
