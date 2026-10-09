# B105: iOS Regular-File Hard-Link Count

## Status

`IosFiles::regular_file_hard_link_count` returns the raw `st_nlink` value for a regular file from
one no-follow metadata lookup. It is a point-in-time diagnostic, not an alias list or write-safety
guarantee

## Objective

Expose a narrowly scoped iOS query that reports whether a regular file has more than one hard link
at the time of observation, without reading or opening file contents

## Contract

- `regular_file_hard_link_count(&self, path: AppPath<'_>) -> Result<u64, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. It must be a regular
  file; a final symlink, directory, or special entry returns `InvalidInput`, not a result for its
  referent or a different entry class.
- The returned `u64` is the unsigned `st_nlink` count at that observation. A count greater than one
  means the regular-file inode had multiple hard links then. The call does not list link names,
  prove that all links are in this sandbox, or guarantee that a later operation sees the same
  count or inode.
- The count can change after link/unlink or path replacement. It is not a stable content identity,
  exclusive-ownership proof, or guard against a concurrent hard-link mutation.
- Missing final entries map through the existing POSIX mapper to `NotFound`; parent, permission,
  and metadata failures retain the existing mapped categories and native codes.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It does not add hard-link support or a hard-link-count concept
  to the portable `FileBackend`/`Files` contract.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/lstat.2.html)
  defines `st_nlink` as the number of hard links. It specifies that `lstat` reports a symlink's own
  link count as one; B105 rejects symlinks to avoid implying a count for their referents.
- The installed iPhoneOS 26.5 SDK and locked `libc` Apple binding expose `st_nlink` as `nlink_t`
  (`u16`). The API widens this raw nonnegative field to `u64`.
- Darwin `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0.
  This method adds no newer API, framework, dependency, or native symbol.
- The adapter already uses one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup for other metadata
  snapshots. B105 adds no permissions, prompts, provider behavior, bookmarks, coordination, or
  persistence.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run. These
  checks used Rust/Cargo 1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5;
  local Xcode is below the repo's required Xcode 27.x baseline.
