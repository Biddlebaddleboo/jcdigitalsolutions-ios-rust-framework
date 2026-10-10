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

Reconciled and rechecked in isolated worktree `/private/tmp/jc-d28-web` on
`workstream/d28-web-contract`, based on current root `main` HEAD
`3c92157aeae479ccc482cfb3d889df7271d15cfe`. Current root `Cargo.lock` resolves the package and its
dependency tree without edits. The public surface is `HttpsUrl`, `HttpsUrlError`, `WebViewError`,
`NavigationState`, and `WebView`. The crate borrows exact URI text; the focused tests now assert
pointer/length identity and preservation of backend error category plus optional native code.

Passed on Rust/Cargo `1.94.1`: `cargo +1.94.1 check --locked --offline --no-default-features -p
framework-web`; `cargo +1.94.1 test --locked --offline -p framework-web` (4 passed);
`cargo +1.94.1 clippy --locked --offline --all-targets -p framework-web -- -D warnings`;
`cargo +1.94.1 fmt --all -- --check`; `cargo +1.94.1 --locked --offline xtask docs-check`;
`cargo +1.94.1 --locked --offline xtask zero-swift-source`; `git diff --check`; and
`RUSTUP_TOOLCHAIN=1.94.1 crates/framework-web/check.sh`. `cargo +1.94.1 tree --locked --offline
-p framework-web` showed only `framework-core`, `framework-format`, and `iri-string` below the
package.

No iOS backend, workspace, lockfile, capability-manifest, CI, C binding, or aggregate-plan edits;
no Swift source. This validates only the portable contract and its crate gates. It makes no claim
about native view creation, app/page behavior, network access, or platform runtime behavior.
