# PLAN_CAPABILITIES_WEB.md — Workstream D28: Platform-Exclusive Web View Contract

## Objective

Define a minimal no_std contract for a native HTTPS web view and basic navigation controls. Treat
WebKit/browser behavior as a platform-exclusive capability; do not promise a portable browser
engine or parity.

## Dependencies

- Foundation A is integrated
- D8 `framework-format::Uri` is integrated for borrowed absolute-URI syntax

## Write scope

- `crates/framework-web/**`
- `docs/capabilities/web.md`

Do not edit the iOS backend, root workspace configuration, `Cargo.lock`, capability manifest,
aggregate plans/indexes, CI, C bindings, or other portable contracts. The orchestrator owns
lockfile and canonical capability-matrix integration. Do not add Swift source or platform types.

## Contract requirements

- Add an independently usable `#![no_std]` crate named `framework-web` with no unsafe code.
- Add a borrowed `HttpsUrl<'a>` that accepts exact absolute RFC 3986 HTTPS URI text with a
  nonempty authority host and does not normalize, resolve, or copy caller text.
- Define a small `NavigationState` snapshot and a static `WebView` trait for state, back, forward,
  reload, and stop-loading operations.
- Preserve backend error categories and optional native codes for construction failures.
- Keep the contract free of Apple, Android, DOM, JavaScript, file URL, and browser-engine types.
- Document that the capability is platform-exclusive, navigation-state is a snapshot, commands do
  not prove page load, and this surface is not full browser parity.

## Validation and handoff

- Run `cargo check --no-default-features -p framework-web`, focused crate tests, and strict Clippy.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and
  `git diff --check`.
- Use the package-local `crates/framework-web/check.sh` gate after root lockfile reconciliation.
- Report public symbols, ownership/text-preservation behavior, exact checks, deviations, and
  unresolved assumptions. Do not edit shared manifest totals; the orchestrator integrates D28
  centrally.

## Status

Implemented in isolated worktree `/Users/john/Projects/.worktrees/jcdig-webkit-d28` on
`workstream/capabilities-webkit-d28`, based on `main` HEAD `7b5513fa2a4864d21a594cbf1fbd43951427155d`.
The public surface is `HttpsUrl`, `HttpsUrlError`, `WebViewError`, `NavigationState`, and `WebView`.

Passed: `cargo check --locked --no-default-features -p framework-web`; `cargo test --locked -p
framework-web` (3 passed); `cargo clippy --locked --all-targets -p framework-web -- -D warnings`;
`cargo doc --locked --no-deps -p framework-web`; `cargo fmt --all -- --check`; `cargo xtask
docs-check`; `cargo xtask zero-swift-source`; and `git diff --check`.

The crate and docs are complete for the bounded contract. Root lockfile and canonical capability
matrix integration remain orchestrator-owned. The worktree lock was temporarily resolved for the
package gates and will be restored before handoff; locked gates need root lock reconciliation to
run on the integrated tree.
