# PLAN_CAPABILITIES_RESOURCES.md — Workstream D7: Packaged Resources Contract

## Status

D7 is integrated as `framework-resources`. A follow-up audit corrected path validation to reject
Windows drive-prefixed paths and expanded the guide's public API inventory. No tests were added or
run. The portable no-default-features check, strict Clippy, workspace formatting, docs check, and
diff check pass. No unresolved D7 assumptions were found.

## Objective

Add a small portable, read-only contract for looking up resources packaged with an application. Keep packaged resources distinct from writable sandbox files and avoid claiming support for asset catalogs, localization policy, or resource discovery beyond exact relative paths.

## Dependencies

- Foundation A is integrated
- D1 `framework-files` is integrated as a reference for portable path and error conventions

## Write scope

- `crates/framework-resources/**`
- `docs/capabilities/resources.md`

Do not edit the iOS backend, root workspace configuration, `Cargo.lock`, capability manifest, shared capability summary, other portable contracts, or validation tooling. The orchestrator owns dependency/lockfile and shared capability-index reconciliation. Do not add Swift source or platform types.

## Contract requirements

- Add an independently usable `#![no_std]` crate named `framework-resources` with static backend selection and no global registry or initialization.
- Represent one resource as a borrowed UTF-8 slash-separated relative path. Reject empty, absolute, empty, `.` and `..` components, backslashes, and NUL bytes; do not normalize names or promise Unicode normalization or case folding.
- Provide synchronous lookup that returns caller-owned bytes and an explicit UTF-8 string conversion that moves the returned bytes without a second facade-level byte copy.
- Preserve backend `ErrorKind` and optional platform error code; an absent resource is reported by the backend as `NotFound`.
- State that methods may block and define no cancellation, directory enumeration, arbitrary URL lookup, localized-resource selection, asset-catalog access, or resource format decoding.
- Keep platform availability, bundle location, file-system security, and copy costs as backend responsibilities.

## Validation and handoff

- Check the crate with `--no-default-features` and strict Clippy.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, and `git diff --check`.
- Do not add or run tests in this slice.
- Report public symbols, ownership/copy behavior, changed files, exact check results, deviations, and unresolved assumptions. Do not edit shared manifest totals; the orchestrator reconciles D7 and its later iOS backend together.
