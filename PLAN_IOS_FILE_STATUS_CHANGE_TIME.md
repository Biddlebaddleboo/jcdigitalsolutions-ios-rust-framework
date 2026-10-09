# B107: iOS Entry Status-Change Time

## Status

`IosFiles::entry_status_change_time` returns the POSIX status-change seconds/nanoseconds pair from
one no-follow metadata lookup. It is distinct from B96 data-modification time, rejects symlinks,
and makes no content-version or reliable-change-token claim

## Objective

Expose a read-only iOS query for the last POSIX file-status change time of one validated sandbox
entry, including regular files, directories, and special entries, without reading contents

## Contract

- `entry_status_change_time(&self, path: AppPath<'_>) -> Result<IosFileStatusChangeTime, FileError>`
  uses the existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. Final symlinks return
  `InvalidInput`; all other entry kinds use that entry's `st_ctime` and `st_ctime_nsec` fields.
- The result type holds signed `i64` seconds relative to the Unix epoch and `u32` nanoseconds within
  that POSIX second. A nanosecond field outside `0..1_000_000_000` maps to `InvalidInput`.
- Apple's iOS `stat(2)` documentation defines `st_ctime` as the last file-status change time and
  lists `chmod`, `chown`, `link`, `mknod`, `rename`, `unlink`, `utimes`, and `write` as operations
  that change it. It therefore covers status metadata changes that B96's data-modification time
  does not necessarily report.
- Apple's iOS `stat(2)` documentation says symbolic links have no own times and that `lstat` only
  reports a limited set of link-owned fields. B107 rejects a final symlink instead of exposing a
  timestamp that could be attributed to the containing directory.
- Filesystems may store or report coarser precision. The result is one point-in-time observation;
  path changes and metadata operations can race. It is not a content version, reliable change
  token, durability proof, or reservation for a later operation.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors preserve the existing mapped category and native code.
- The query is synchronous, reads no content, follows no final symlink, accepts no absolute URL,
  starts no security scope, and grants no access. It has the existing concurrent native
  directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It adds no timestamp concept to the portable
  `FileBackend`/`Files` contract and adds no dependency, framework, or native symbol.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_ctimespec` as last file-status change time, distinguishes it from `st_mtimespec` data
  modification time, lists status-changing operations, and documents the symlink `lstat` limit.
- The installed iPhoneOS 26.5 SDK `sys/stat.h` declares `st_ctimespec` as the time of last status
  change and makes `st_ctime`/`st_ctimensec` aliases in `struct stat`.
- The locked `libc` Apple binding exposes `stat.st_ctime` and `stat.st_ctime_nsec`. The existing
  `ios-files` floor is iOS 10.0; `fstatat` is available from iOS 8.0, so B107 adds no deployment
  floor requirement.
- `ios-files` already uses `fstatat` with `AT_SYMLINK_NOFOLLOW` for metadata snapshots. B107 adds
  no permission prompt, Info.plist key, entitlement, provider behavior, bookmark, coordination, or
  persistence behavior.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run. These
  checks used the repository's current Rust/Cargo 1.94.1 and iOS target setup; the shared checkout
  also contains concurrent unrelated work, which this B107 change did not modify.
