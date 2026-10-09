# PLAN_CAPABILITIES_SHARE.md — Workstream D6: Portable Share Contract

## Status

D6's portable share contract and guide pass acceptance review. Fake-backend tests cover request
values and order, pass-through URL text, both terminal outcomes, backend error preservation,
first-poll start, and pre-/post-presentation drop behavior. The portable surface uses only `alloc`,
`core`, and `framework-core`; D5 clipboard behavior remains unchanged. Validation passes:
`cargo fmt --all -- --check`, `cargo test --locked --offline -p framework-sharing` (10 passed),
`cargo check --locked --offline -p framework-sharing --no-default-features`, and `git diff --check`.

## Objective

Add a small `no_std` portable contract for caller-provided outgoing share content and the result of a system-share operation. This contract does not present UI or implement an iOS backend.

## Dependencies

- Foundation A and the D5 `framework-sharing` crate are integrated.
- Keep D5 clipboard behavior and documentation unchanged.

## Read first

- `PLAN.md`
- `PLAN_CAPABILITIES.md`
- `PLAN_CAPABILITIES_CLIPBOARD.md`
- `crates/framework-core/src/lib.rs`
- `crates/framework-sharing/src/lib.rs`
- `docs/capabilities/sharing.md`

## Write scope

- `crates/framework-sharing/src/share.rs`
- `crates/framework-sharing/src/lib.rs` only for the public module/re-export
- `docs/capabilities/share.md`

Do not edit D5 clipboard symbols or docs, root workspace files, `Cargo.lock`, the shared capability manifest or index, iOS backend crates, CI, C bindings, Swift ABI, or native UI APIs. The orchestrator owns shared manifest and index updates.

## Required contract

- Define owned portable share content and a request that can contain one or more supported items. Keep V1 scope to UTF-8 text and URL text; do not expose `NSURL`, UIKit, file URLs, platform item providers, or third-party URL types.
- State whether URL text is validated or passed through verbatim. Do not claim that a platform backend preserves every URL string exactly.
- Define a portable terminal result for user completion versus dismissal without leaking activity-controller or platform activity identifiers.
- Expose a statically selected `ShareBackend` and thin client with `Availability`, `ErrorKind`, and native error-code preservation consistent with D5. No boxed trait object, executor, `Send` bound, global registry, or hidden initialization.
- Operations start on first future poll. Document drop before first poll, drop after native presentation begins, result suppression, callback detachment, exactly-once completion, and the fact that a system share UI may remain visible after Rust drops its future.
- Keep permission, presentation context, lifecycle, privacy, and native UI details in the platform backend guide.
- Explicitly defer file/image/rich payloads, share extensions, recipients, activity selection, previews, custom activities, and clipboard behavior.

## Documentation

Document supported items, ownership/copy costs, URL treatment, result semantics, cancellation limits, errors, static backend substitution, and all non-goals. Keep native presentation and availability claims out of the portable guide.

## Validation and handoff

- Add deterministic fake-backend tests for request values, result states, errors, first-poll start, and drop behavior.
- Run `cargo fmt --all -- --check`, `cargo test -p framework-sharing`, `cargo check -p framework-sharing --no-default-features`, and `git diff --check`.
- Audit the portable API for `std`, platform types, dynamic dispatch, hidden initialization, unrelated dependencies, and changes to D5 behavior.
- Report changed files, commit SHA, exact checks, deviations, and unresolved assumptions. Do not push.
