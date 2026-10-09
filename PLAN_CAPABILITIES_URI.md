# PLAN_CAPABILITIES_URI.md — Workstream D8: Portable URI Values

## Status

D8 matches its contract: `Uri` and `UriReference` borrow exact text and component slices, while
`iri-string` 0.7.14 stays behind the facade with default and optional features disabled. Locked
no-default-features check, strict Clippy, workspace formatting, docs check, feature-tree review,
and diff check pass. No tests were added or run; no D8 gap was found.

## Objective

Add borrowed, validated URI and URI-reference values for the URL/URI capability row. Preserve the caller's exact text; do not add URL normalization, host resolution, dereference, or platform behavior.

## Dependency rationale

Use `iri-string` 0.7.14 with default features disabled to validate borrowed RFC 3986 URI forms without adding a custom parser or requiring `std`/`alloc`. Its no_std and borrowed URI types are documented in the [crate API](https://docs.rs/crate/iri-string/0.7.14); the syntax source is [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986.html). Do not enable `memchr`, `serde`, `alloc`, or `std` for this slice.

## Dependencies

- Foundation A is integrated
- D1 `framework-network` is integrated as an example of a deliberately bounded borrowed network value

## Write scope

- `Cargo.toml` workspace dependency entry for `iri-string`
- `crates/framework-format/**`
- `docs/capabilities/uri.md`

Do not edit iOS backends, `Cargo.lock`, the capability manifest, shared capability summary, other portable contracts, or validation tooling. The orchestrator owns lockfile, no_std list, and shared capability-index reconciliation. Do not add Swift source or platform types.

## Contract requirements

- Add an independently usable `#![no_std]` crate named `framework-format`.
- Expose borrowed `Uri<'a>` and `UriReference<'a>` wrappers for RFC 3986 absolute URI and URI-reference syntax using `iri-string` validation.
- Preserve the exact borrowed input; parsing and component access must not allocate, normalize, percent-decode, resolve, or perform DNS.
- Expose the raw text and borrowed component views supported by the dependency, with clear `Option` semantics for absent components.
- Map invalid syntax to a stable `InvalidInput` error. Keep IRI input with unescaped non-ASCII text outside this URI-only contract.
- State that syntactic validity does not prove scheme-specific validity, web safety, network reachability, or equivalence after normalization.
- Keep the crate portable and independent of Foundation/NSURL; do not modify the existing HTTP-only `HttpUrl` semantics.

## Validation and handoff

- Check `framework-format` with `--no-default-features` and strict Clippy.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, and `git diff --check`.
- Do not add or run tests in this slice.
- Report public symbols, dependency feature state, ownership/copy behavior, exact checks, deviations, and unresolved assumptions. Do not edit shared manifest totals; the orchestrator reconciles D8 centrally.
