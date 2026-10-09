# B96: iOS Entry Modification-Time Snapshot

## Status

`IosFiles::entry_modification_time` returns the POSIX seconds/nanoseconds pair from one
no-follow metadata lookup. It reports the final entry itself and makes no content-version,
reliable-change-token, or durability claim

## Objective

Add a read-only, prompt-free iOS query of the data-modification timestamp for one validated sandbox
entry, without opening or reading its contents

## Contract

- `entry_modification_time(&self, path: AppPath<'_>) -> Result<IosFileModificationTime, FileError>`
  uses the existing semantic `AppDirectory`, `path_parts`, root descriptor, and `open_parent`
  traversal.
- The final entry is inspected once by `fstatat(..., AT_SYMLINK_NOFOLLOW)`. It returns the
  `st_mtime` seconds and `st_mtime_nsec` fields without local-time conversion. A symlink reports
  the symlink's own fields, not its target's.
- The result type holds signed `i64` seconds since the Unix epoch and `u32` nanoseconds within that
  POSIX second. A nanosecond field outside `0..1_000_000_000` maps to `InvalidInput`.
- Filesystems may store coarser precision than the returned nanosecond field suggests. A caller may
  set modification timestamps. The value is not a content version, reliable change token, or proof
  of durability.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors preserve the existing mapped category and native code.
- The query accepts regular files, directories, symlinks, and special entries. It is synchronous,
  reads no content, allocates no payload-sized buffer, follows no final symlink, accepts no absolute
  URL, starts no security scope, and grants no access. It has the existing concurrent native
  directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It does not add a timestamp concept to the portable
  `FileBackend`/`Files` contract or change `AppPath` semantics.

## Static evidence

- The installed iOS SDK `sys/stat.h` defines `st_mtimespec` as the time of last data modification
  and declares `fstatat` available from iOS 8.0. The existing `ios-files` package floor is iOS 10.0,
  so B96 adds no deployment-floor requirement.
- The locked `libc` Apple binding exposes `stat.st_mtime` and `stat.st_mtime_nsec`; B96 reads those
  fields from the structure already returned by `fstatat`.
- `ios-files` already uses `fstatat` with `AT_SYMLINK_NOFOLLOW` for file size, entry kind, existence,
  directory classification, and replacement checks. B96 adds no dependency, framework, or new
  native symbol.
- No permission prompt, Info.plist key, entitlement, provider API, bookmark, coordination, or
  persistence behavior is involved.

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
