# PLAN_CAPABILITIES_DATA.md — Workstream D13: Portable Byte and UTF-8 Data

## Status

D13's standalone `framework-data` contract, guide, and focused unit tests are present. Direct no-std
`rustc` compile, strict `clippy-driver`, four unit tests, rustdoc build/example, formatting, and diff
checks pass. Root integration now passes `cargo +1.94.1 check --workspace`, the 20-crate host
no-std link probe, `cargo xtask docs-check`, and workspace format checks; the root lockfile is
reconciled. No iOS adapter or live platform behavior is part of D13

The 2026-10-08 closeout audit found no contract gap. Re-run package checks passed: no-default-features
check, strict all-target Clippy, all four unit tests, the crate doctest, warning-denied package
rustdoc, workspace formatting, documentation-index validation, and `git diff --check`. No crate or
guide edits were needed

## Objective

Implement the portable data/byte/string conversion row with exact borrowed byte and UTF-8 views,
explicit allocating copies, and explicit ownership-transfer conversions. Keep this utility separate
from D1-owned file, preference, and network crates.

## Dependencies

- Rust `core` and `alloc` only; no third-party or workspace crate dependency
- D1 data/string conversions remain unchanged and do not depend on this crate

## Write scope

- `PLAN_CAPABILITIES_DATA.md`
- `crates/framework-data/Cargo.toml`
- `crates/framework-data/src/lib.rs`
- `docs/capabilities/data.md`

Do not edit root `Cargo.toml`, `Cargo.lock`, `PLAN_CAPABILITIES.md`, the capability README or global
status manifest, D1-owned crates, platform backends, bindings, or validation tooling. The
orchestrator owns workspace-lock and capability-index integration.

## Contract requirements

- Add an independently usable `#![no_std]` crate named `framework-data`; use `alloc` only for
  owned `Vec<u8>` and `String` values.
- Expose `ByteView<'a>` over exact caller-borrowed bytes and `Utf8View<'a>` over exact valid UTF-8
  text. Constructing, viewing, or converting between these borrowed forms must not allocate, copy,
  normalize, transcode, or retain the borrow beyond its type lifetime.
- Validate byte-to-UTF-8 views with `core::str::from_utf8`; map invalid input to
  `DataError::InvalidUtf8` and preserve the valid-prefix offset for failed owned conversion.
- Make copies explicit: `ByteView::copy_to_owned` and `Utf8View::copy_to_owned` copy the source
  bytes into owned storage, allocating as needed.
- Make allocation transfer explicit: `OwnedBytes::from_vec` / `into_vec` and
  `OwnedText::from_string` / `into_string` move ownership without copying. `OwnedText::into_bytes`
  moves its backing buffer. `OwnedBytes::try_into_text` uses checked UTF-8 conversion without a
  successful-path copy; on failure return the original owned bytes and valid-prefix offset.
- Do not implement `Clone` for owned wrappers, so an accidental implicit full-data copy is not part
  of the API. Borrowed views may be `Copy` because that copies only a reference.
- Add focused tests for exact borrowing, invalid UTF-8, owned-copy independence, and pointer/capacity
  preservation across allocation-transfer conversions.
- Use no unsafe code, platform types, D1 dependencies, or external dependencies.
- Document allocator needs for copy operations, ownership lifetimes, invalid input, error offsets,
  and the difference between copying and moving an allocation. Do not claim FFI, native-buffer,
  in-place mutation, encoding-detection, or normalization support.

## Validation and handoff

- Run formatting, `cargo check -p framework-data --no-default-features`, strict Clippy,
  `cargo test -p framework-data`, package rustdoc, `cargo xtask docs-check`, and `git diff --check`
  after workspace lock integration.
- During isolated work, do not mutate the root lockfile; the orchestrator runs the Cargo gates after
  integrating the workspace package.
- Report public symbols, borrow/copy/transfer semantics, exact checks, deviations, and unresolved
  assumptions. Do not edit shared capability totals.
