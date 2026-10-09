# B103: iOS POSIX Permission-Bit Snapshot

## Status

`IosFiles::entry_posix_permission_bits` returns the raw `st_mode & 0o7777` mask from one no-follow
metadata lookup. It reports stored POSIX bits only; it does not report effective access or predict
whether a later file operation will succeed

## Objective

Expose one bounded, read-only iOS query of the permission and special-mode bits recorded for a
sandbox entry, without reading file contents. The intended use is diagnostics of stored mode bits,
including the creation modes used by the adapter; this is not an access check

## Contract

- `entry_posix_permission_bits(&self, path: AppPath<'_>) -> Result<u16, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. The returned `u16` is
  exactly `st_mode & 0o7777`, including owner/group/other read, write, and execute bits, set-user-ID,
  set-group-ID, and sticky bits. A symlink returns its own raw bits, not its target's.
- Missing final entries map through the existing POSIX mapper to `NotFound`; parent, permission, and
  metadata failures keep the existing mapped categories and native codes.
- The snapshot does not report effective access, account for every system policy, or guarantee that
  a later operation will succeed. It is diagnostic metadata only; callers must handle actual read
  and write errors. Concurrent path changes may make later operations refer to a different entry.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It does not add a permission concept to the portable
  `FileBackend`/`Files` contract or change `AppPath` semantics.

## Static evidence

- The installed iPhoneOS 26.5 SDK declares `stat.st_mode` as the inode's mode and defines the
  standard permission and special-mode masks. The checked-in `libc` Apple binding types `mode_t`
  as `u16`; the API returns only the `0o7777` mask.
- `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0. This
  method adds no newer API, framework, dependency, or native symbol.
- `ios-files` already uses `fstatat(..., AT_SYMLINK_NOFOLLOW)` for file size, entry kind, existence,
  modification time, identity, directory classification, and replacement checks.
- The adapter creates new files with `0600` and directories with `0700`; the documented process
  `umask` may remove bits. This query can report the stored mask for diagnostics without changing
  those write semantics.
- Apple documents the `st_mode` permission bits in its iOS [`chmod(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/chmod.2.html).
  This slice does not use them as an authorization decision.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and
  `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run. These
  checks used Rust/Cargo 1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5;
  local Xcode is below the repo's required Xcode 27.x baseline.
